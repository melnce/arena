//! Ordered-pair matchup matrix. Whole games stay in Rust (rayon).

use std::collections::BTreeMap;
use std::sync::Arc;

use arena_engine::{CardDb, CardId, First};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use rayon::prelude::*;

use crate::convert::{deck_from_value, py_err_msg, py_to_value, value_to_py};
use crate::game::PyCardDb;
use crate::play::{game_seed, play_one, Policy};

#[derive(Clone, Default)]
struct PairAgg {
    games: u32,
    a_wins: u32,
    b_wins: u32,
    first_player_wins: u32,
    turns: u64,
    actions: u64,
}

#[pyfunction]
#[pyo3(name = "matchup", signature = (db, decks, games, seed, policy = "random", threads = None))]
pub fn py_matchup<'py>(
    py: Python<'py>,
    db: &PyCardDb,
    decks: Bound<'_, PyAny>,
    games: u32,
    seed: u64,
    policy: &str,
    threads: Option<usize>,
) -> PyResult<Bound<'py, PyAny>> {
    let policy = Policy::parse(policy).map_err(py_err_msg)?;
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

    let results = py.allow_threads(|| run_jobs(&db, &lists, n, seed, policy, n_threads, &jobs))?;

    let mut aggs: Vec<Vec<PairAgg>> = vec![vec![PairAgg::default(); n]; n];
    for ((i, j), r) in results {
        let a = &mut aggs[i][j];
        a.games += 1;
        a.turns += u64::from(r.turns);
        a.actions += u64::from(r.actions);
        if let Some(w) = r.winner {
            if w == arena_engine::PlayerId::A {
                a.a_wins += 1;
            } else {
                a.b_wins += 1;
            }
            if w == r.first {
                a.first_player_wins += 1;
            }
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
                    "first_player_wins": a.first_player_wins,
                    "mean_turns": a.turns as f64 / g,
                    "mean_actions": a.actions as f64 / g,
                }),
            );
        }
        matrix.insert(a_name.clone(), serde_json::Value::Object(row));
    }

    value_to_py(
        py,
        &serde_json::json!({
            "seed": seed,
            "games": games,
            "policy": match policy {
                Policy::Random => "random",
                Policy::FirstLegal => "first-legal",
            },
            "threads": n_threads,
            "matrix": matrix,
        }),
    )
}

fn num_cpus_hint() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn run_jobs(
    db: &Arc<CardDb>,
    lists: &[Vec<CardId>],
    n: usize,
    seed: u64,
    policy: Policy,
    n_threads: usize,
    jobs: &[(usize, usize, u32)],
) -> PyResult<Vec<((usize, usize), crate::play::PlayResult)>> {
    let work = |i: usize, j: usize, g: u32| {
        let s = game_seed(seed, (i * n + j) as u64, u64::from(g));
        play_one(db, s, &lists[i], &lists[j], First::Coin, policy)
            .map(|r| ((i, j), r))
            .map_err(py_err_msg)
    };

    if n_threads <= 1 {
        let mut out = Vec::with_capacity(jobs.len());
        for &(i, j, g) in jobs {
            out.push(work(i, j, g)?);
        }
        return Ok(out);
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(n_threads)
        .build()
        .map_err(|e| py_err_msg(e.to_string()))?;
    pool.install(|| {
        jobs.par_iter()
            .map(|&(i, j, g)| work(i, j, g))
            .collect::<PyResult<Vec<_>>>()
    })
}
