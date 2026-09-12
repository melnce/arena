//! Ordered-pair matchup matrix. Whole games stay in Rust (rayon).

use std::collections::BTreeMap;
use std::sync::Arc;

use arena_engine::{AnyPolicy, CardDb, CardId, End, First, Outcome, PlayerId};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use rayon::prelude::*;

use crate::convert::{deck_from_value, py_err_msg, py_to_value, value_to_py};
use crate::game::PyCardDb;
use crate::play::{game_seed, play_one};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstMode {
    Alternate,
    Coin,
    A,
    B,
}

impl FirstMode {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "alternate" => Ok(Self::Alternate),
            "coin" => Ok(Self::Coin),
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            other => Err(format!(
                "first must be 'alternate', 'coin', 'a', or 'b' (got {other})"
            )),
        }
    }

    fn for_game(self, g: u32) -> First {
        match self {
            Self::Alternate => {
                if g.is_multiple_of(2) {
                    First::A
                } else {
                    First::B
                }
            }
            Self::Coin => First::Coin,
            Self::A => First::A,
            Self::B => First::B,
        }
    }
}

#[derive(Clone, Default)]
struct PairAgg {
    games: u32,
    a_wins: u32,
    b_wins: u32,
    first_player_wins: u32,
    a_games_as_first: u32,
    a_wins_as_first: u32,
    turns: u64,
    actions: u64,
    lethal: u32,
    deckout: u32,
    turn_cap: u32,
    action_cap: u32,
    no_legal: u32,
    illegal: u32,
}

struct GameRow {
    i: usize,
    j: usize,
    g: u32,
    seed: u64,
    out: Outcome,
}

#[pyfunction]
#[pyo3(
    name = "matchup",
    signature = (
        db,
        decks,
        games,
        seed,
        policy = "random",
        threads = None,
        policy_a = None,
        policy_b = None,
        first = "alternate",
        records = false,
    )
)]
#[allow(clippy::too_many_arguments)]
pub fn py_matchup<'py>(
    py: Python<'py>,
    db: &PyCardDb,
    decks: Bound<'_, PyAny>,
    games: u32,
    seed: u64,
    policy: &str,
    threads: Option<usize>,
    policy_a: Option<&str>,
    policy_b: Option<&str>,
    first: &str,
    records: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let pol_a = policy_a.unwrap_or(policy);
    let pol_b = policy_b.unwrap_or(policy);
    AnyPolicy::parse_spec(pol_a).map_err(py_err_msg)?;
    AnyPolicy::parse_spec(pol_b).map_err(py_err_msg)?;
    let first_mode = FirstMode::parse(first).map_err(py_err_msg)?;
    let decks_v = py_to_value(&decks)?;
    let obj = decks_v
        .as_object()
        .ok_or_else(|| py_err_msg("decks must be a dict of name → deck dict"))?;
    let mut named: BTreeMap<String, Vec<CardId>> = BTreeMap::new();
    for (name, deck) in obj {
        named.insert(name.clone(), deck_from_value(deck)?);
    }
    if named.is_empty() {
        return Err(py_err_msg("decks must contain at least one deck"));
    }
    let names: Vec<String> = named.keys().cloned().collect();
    let lists: Vec<Vec<CardId>> = names.iter().map(|n| named[n].clone()).collect();
    let n = names.len();
    let db = db.inner.clone();
    let n_threads = threads.unwrap_or_else(num_cpus_hint);
    let jobs: Vec<(usize, usize, u32)> = (0..n)
        .flat_map(|i| (0..n).flat_map(move |j| (0..games).map(move |g| (i, j, g))))
        .collect();

    let rows = py.allow_threads(|| {
        run_jobs(JobSet {
            db: &db,
            lists: &lists,
            n,
            seed,
            pol_a,
            pol_b,
            first_mode,
            n_threads,
            jobs: &jobs,
        })
    })?;

    let mut aggs: Vec<Vec<PairAgg>> = vec![vec![PairAgg::default(); n]; n];
    for row in &rows {
        let a = &mut aggs[row.i][row.j];
        a.games += 1;
        a.turns += u64::from(row.out.turns);
        a.actions += u64::from(row.out.actions);
        if row.out.first == PlayerId::A {
            a.a_games_as_first += 1;
        }
        if let Some(w) = row.out.winner {
            if w == PlayerId::A {
                a.a_wins += 1;
                if row.out.first == PlayerId::A {
                    a.a_wins_as_first += 1;
                }
            } else {
                a.b_wins += 1;
            }
            if w == row.out.first {
                a.first_player_wins += 1;
            }
        }
        match row.out.end {
            End::Lethal => a.lethal += 1,
            End::Deckout => a.deckout += 1,
            End::TurnCap => a.turn_cap += 1,
            End::ActionCap => a.action_cap += 1,
            End::NoLegal => a.no_legal += 1,
            End::Illegal => a.illegal += 1,
        }
    }

    let mut matrix = serde_json::Map::new();
    for (i, a_name) in names.iter().enumerate() {
        let mut row = serde_json::Map::new();
        for (j, b_name) in names.iter().enumerate() {
            let a = &aggs[i][j];
            let g = f64::from(a.games.max(1));
            row.insert(
                b_name.clone(),
                serde_json::json!({
                    "games": a.games,
                    "a_wins": a.a_wins,
                    "b_wins": a.b_wins,
                    "draws": a.games.saturating_sub(a.a_wins + a.b_wins),
                    "first_player_wins": a.first_player_wins,
                    "a_games_as_first": a.a_games_as_first,
                    "a_wins_as_first": a.a_wins_as_first,
                    "mean_turns": a.turns as f64 / g,
                    "mean_actions": a.actions as f64 / g,
                    "end": {
                        "lethal": a.lethal,
                        "deckout": a.deckout,
                        "turn_cap": a.turn_cap,
                        "action_cap": a.action_cap,
                        "no_legal": a.no_legal,
                        "illegal": a.illegal,
                    },
                }),
            );
        }
        matrix.insert(a_name.clone(), serde_json::Value::Object(row));
    }

    let mut top = serde_json::json!({
        "seed": seed,
        "games": games,
        "policy_a": pol_a,
        "policy_b": pol_b,
        "first": first,
        "threads": n_threads,
        "matrix": matrix,
    });
    if records {
        let recs: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "a": names[row.i],
                    "b": names[row.j],
                    "g": row.g,
                    "seed": row.seed,
                    "first": row.out.first.as_str(),
                    "winner": row.out.winner.map(|w| serde_json::Value::String(w.as_str().into())).unwrap_or(serde_json::Value::Null),
                    "turns": row.out.turns,
                    "actions": row.out.actions,
                    "end": row.out.end.as_str(),
                })
            })
            .collect();
        top.as_object_mut()
            .expect("object")
            .insert("records".into(), serde_json::Value::Array(recs));
    }

    value_to_py(py, &top)
}

fn num_cpus_hint() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

struct JobSet<'a> {
    db: &'a Arc<CardDb>,
    lists: &'a [Vec<CardId>],
    n: usize,
    seed: u64,
    pol_a: &'a str,
    pol_b: &'a str,
    first_mode: FirstMode,
    n_threads: usize,
    jobs: &'a [(usize, usize, u32)],
}

fn run_jobs(set: JobSet<'_>) -> PyResult<Vec<GameRow>> {
    let work = |i: usize, j: usize, g: u32| {
        let s = game_seed(set.seed, (i * set.n + j) as u64, u64::from(g));
        let first = set.first_mode.for_game(g);
        play_one(
            set.db,
            s,
            &set.lists[i],
            &set.lists[j],
            first,
            set.pol_a,
            set.pol_b,
        )
        .map(|out| GameRow {
            i,
            j,
            g,
            seed: s,
            out,
        })
        .map_err(py_err_msg)
    };

    if set.n_threads <= 1 {
        let mut out = Vec::with_capacity(set.jobs.len());
        for &(i, j, g) in set.jobs {
            out.push(work(i, j, g)?);
        }
        return Ok(out);
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(set.n_threads)
        .build()
        .map_err(|e| py_err_msg(e.to_string()))?;
    pool.install(|| {
        set.jobs
            .par_iter()
            .map(|&(i, j, g)| work(i, j, g))
            .collect::<PyResult<Vec<_>>>()
    })
}
