//! Full-game h0 self-play pins (`fixtures/h0_exact/games.json`).
//!
//! Regenerate once on `main` via `regen_h0_exact_games` (release, `--ignored`).

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::{
    apply, legal_actions, new_game, policy_rng, search_key, snapshot::hash as snapshot_hash,
    trace::fnv1a64, AnyPolicy, CardDb, First, GameConfig, Phase, Policy, H0,
};
use serde_json::{json, Value};

mod common;
use common::*;

const S1: &str = "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=builtin,oorder=1,oopen=1,onext=40";

/// Pre–brief-YA `py/serve.py` `DEFAULT_STRONG` (`order=builtin,oorder=1`) with `nodes=16000,threads=4`.
const S2: &str = "h0:nodes=16000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=4,lostrank=4000,order=builtin,oorder=1,oopen=1,onext=40";

/// Pre–brief-YA `py/serve.py` `DEFAULT_CHEAT` (`order=builtin,oorder=1`) with `nodes=16000,threads=4`.
const S3: &str = "h0:nodes=16000,horizon=3,k=16,info=all,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=4,lostrank=4000,order=builtin,oorder=1,oopen=1,onext=40";

const S4: &str = "h0";

const S5: &str = "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,hbk=16";

struct PlannedGame {
    spec_key: &'static str,
    spec: &'static str,
    seed: u64,
}

fn game_plan() -> Vec<PlannedGame> {
    let mut out = Vec::new();
    for seed in [71001u64, 71002, 71003, 71004] {
        out.push(PlannedGame {
            spec_key: "S1",
            spec: S1,
            seed,
        });
    }
    for seed in [71005u64, 71006] {
        out.push(PlannedGame {
            spec_key: "S2",
            spec: S2,
            seed,
        });
    }
    out.push(PlannedGame {
        spec_key: "S3",
        spec: S3,
        seed: 72022,
    });
    for seed in [71001u64, 71002, 71003, 71008] {
        out.push(PlannedGame {
            spec_key: "S4",
            spec: S4,
            seed,
        });
    }
    for seed in [71004u64, 71005] {
        out.push(PlannedGame {
            spec_key: "S5",
            spec: S5,
            seed,
        });
    }
    out
}

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn games_fixture_path() -> PathBuf {
    fixtures_dir().join("h0_exact/games.json")
}

fn meta_games() -> Vec<Value> {
    let p = fixtures_dir().join("h0_exact/meta_games.json");
    let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p:?}: {e}"));
    serde_json::from_str(&text).expect("meta games")
}

fn meta_row(games: &[Value], seed: u64) -> &Value {
    games
        .iter()
        .find(|g| g["seed"].as_u64() == Some(seed))
        .unwrap_or_else(|| panic!("no meta game {seed}"))
}

fn parse_first(v: &Value) -> First {
    match v.as_str() {
        Some("b") | Some("B") => First::B,
        Some("a") | Some("A") => First::A,
        other => panic!("first {other:?}"),
    }
}

#[derive(Clone, Debug)]
struct BotDecision {
    decision_index: u32,
    search_key: u64,
    idx: usize,
    value_bits: Option<u32>,
    stats: String,
    explain: String,
}

#[derive(Clone, Debug)]
struct GameCapture {
    id: String,
    spec_key: String,
    seed: u64,
    deck_a: String,
    deck_b: String,
    first: String,
    n_decisions: u32,
    decisions_fnv: u64,
    root_search_keys_fnv: u64,
    final_search_key: u64,
    final_snapshot_hash: u64,
    winner: Option<String>,
    decisions: Vec<BotDecision>,
}

fn decisions_fnv(decisions: &[BotDecision]) -> u64 {
    let mut bytes = Vec::new();
    for d in decisions {
        bytes.extend_from_slice(&(d.idx as u64).to_le_bytes());
        bytes.extend_from_slice(&d.value_bits.unwrap_or(0).to_le_bytes());
        bytes.extend_from_slice(d.stats.as_bytes());
        bytes.extend_from_slice(d.explain.as_bytes());
    }
    fnv1a64(&bytes)
}

fn root_search_keys_fnv(decisions: &[BotDecision]) -> u64 {
    let mut bytes = Vec::new();
    for d in decisions {
        bytes.extend_from_slice(&d.search_key.to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn run_game(db: &CardDb, plan: &PlannedGame, meta: &Value) -> GameCapture {
    let deck_a = meta["deck_a"].as_str().expect("deck_a").to_string();
    let deck_b = meta["deck_b"].as_str().expect("deck_b").to_string();
    let first_s = meta["first"].as_str().expect("first").to_string();
    let first = parse_first(&json!(first_s));
    let mut st = new_game(
        db,
        GameConfig {
            seed: plan.seed,
            deck_a: load_deck_file(&deck_a),
            deck_b: load_deck_file(&deck_b),
            first,
            opening_hands: None,
        },
    )
    .expect("new_game");

    let mut decisions = Vec::new();
    let mut decision_index = 0u32;
    while st.winner.is_none() && !matches!(st.phase, Phase::Terminal) {
        let legal = legal_actions(db, &st);
        if legal.is_empty() {
            break;
        }
        let action = if legal.len() >= 2 {
            let sk = search_key(&st);
            let mut h0 = parse_h0(plan.spec);
            h0.arm_explain();
            let mut rng = policy_rng(plan.seed ^ u64::from(decision_index));
            let idx = h0.choose(db, &st, &legal, &mut rng);
            let explain = h0
                .take_explain()
                .map(|r| serde_json::to_string(&r).expect("explain json"))
                .expect("explain armed");
            let stats = serde_json::to_string(&h0.stats).expect("stats json");
            decisions.push(BotDecision {
                decision_index,
                search_key: sk,
                idx,
                value_bits: h0.last_value().map(f32::to_bits),
                stats,
                explain,
            });
            decision_index += 1;
            legal[idx].clone()
        } else {
            legal[0].clone()
        };
        if apply(db, &mut st, action).is_err() {
            break;
        }
    }

    let winner = st.winner.map(|w| match w {
        arena_engine::PlayerId::A => "A".to_string(),
        arena_engine::PlayerId::B => "B".to_string(),
    });

    GameCapture {
        id: format!("{}:{}", plan.spec_key, plan.seed),
        spec_key: plan.spec_key.to_string(),
        seed: plan.seed,
        deck_a,
        deck_b,
        first: first_s,
        n_decisions: decisions.len() as u32,
        decisions_fnv: decisions_fnv(&decisions),
        root_search_keys_fnv: root_search_keys_fnv(&decisions),
        final_search_key: search_key(&st),
        final_snapshot_hash: snapshot_hash(&st),
        winner,
        decisions,
    }
}

fn decision_pin_json(d: &BotDecision) -> Value {
    json!({
        "decision_index": d.decision_index,
        "search_key": d.search_key.to_string(),
        "idx": d.idx,
        "value_bits": d.value_bits,
        "stats_fnv": fnv1a64(d.stats.as_bytes()).to_string(),
        "explain_fnv": fnv1a64(d.explain.as_bytes()).to_string(),
    })
}

fn capture_to_json(g: &GameCapture) -> Value {
    json!({
        "id": g.id,
        "spec_key": g.spec_key,
        "seed": g.seed,
        "deck_a": g.deck_a,
        "deck_b": g.deck_b,
        "first": g.first,
        "n_decisions": g.n_decisions,
        "decisions_fnv": g.decisions_fnv.to_string(),
        "root_search_keys_fnv": g.root_search_keys_fnv.to_string(),
        "final_search_key": g.final_search_key.to_string(),
        "final_snapshot_hash": g.final_snapshot_hash.to_string(),
        "winner": g.winner,
        "decision_pins": g.decisions.iter().map(decision_pin_json).collect::<Vec<_>>(),
    })
}

fn collect_all(db: &CardDb) -> Vec<GameCapture> {
    let meta = meta_games();
    game_plan()
        .iter()
        .map(|plan| {
            let row = meta_row(&meta, plan.seed);
            run_game(db, plan, row)
        })
        .collect()
}

fn u64_field(v: &Value, key: &str) -> u64 {
    v[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} string"))
        .parse()
        .unwrap_or_else(|e| panic!("{key}: {e}"))
}

fn compare_game(pin: &Value, live: &GameCapture) {
    let id = pin["id"].as_str().unwrap();
    assert_eq!(id, live.id);
    assert_eq!(pin["seed"].as_u64(), Some(live.seed));
    assert_eq!(pin["n_decisions"], json!(live.n_decisions));
    if u64_field(pin, "decisions_fnv") != live.decisions_fnv
        || u64_field(pin, "root_search_keys_fnv") != live.root_search_keys_fnv
    {
        diff_first_decision(id, pin, live);
    }
    assert_eq!(
        u64_field(pin, "final_search_key"),
        live.final_search_key,
        "{id} final_search_key"
    );
    assert_eq!(
        u64_field(pin, "final_snapshot_hash"),
        live.final_snapshot_hash,
        "{id} final_snapshot_hash"
    );
    assert_eq!(pin["winner"], json!(live.winner));
}

fn pin_decisions(pin: &Value) -> &[Value] {
    if let Some(a) = pin["decision_pins"].as_array() {
        return a;
    }
    pin["decisions"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

fn diff_first_decision(id: &str, pin: &Value, live: &GameCapture) {
    let pin_dec = pin_decisions(pin);
    for (p, l) in pin_dec.iter().zip(live.decisions.iter()) {
        let stats_fnv = fnv1a64(l.stats.as_bytes());
        let explain_fnv = fnv1a64(l.explain.as_bytes());
        let pin_stats_fnv = if p.get("stats_fnv").is_some() {
            u64_field(p, "stats_fnv")
        } else {
            fnv1a64(p["stats"].as_str().unwrap_or("").as_bytes())
        };
        let pin_explain_fnv = if p.get("explain_fnv").is_some() {
            u64_field(p, "explain_fnv")
        } else {
            fnv1a64(p["explain"].as_str().unwrap_or("").as_bytes())
        };
        if p["decision_index"] != json!(l.decision_index)
            || p["idx"] != json!(l.idx)
            || p["value_bits"] != json!(l.value_bits)
            || u64_field(p, "search_key") != l.search_key
            || pin_stats_fnv != stats_fnv
            || pin_explain_fnv != explain_fnv
        {
            panic!(
                "{id} first differing decision index {}\n  pin idx={} live idx={}\n  pin value_bits={:?} live {:?}\n  pin search_key={} live {}\n  pin stats_fnv={} live {stats_fnv}\n  pin explain_fnv={} live {explain_fnv}",
                l.decision_index,
                p["idx"],
                l.idx,
                p["value_bits"],
                l.value_bits,
                p["search_key"],
                l.search_key,
                pin_stats_fnv,
                pin_explain_fnv,
            );
        }
    }
    panic!(
        "{id} aggregate fnv mismatch (decisions_fnv pin {} live {}; root_search_keys_fnv pin {} live {})",
        pin["decisions_fnv"],
        live.decisions_fnv,
        pin["root_search_keys_fnv"],
        live.root_search_keys_fnv
    );
}

#[test]
#[ignore = "slow.yml suite h0_exact_games"]
fn h0_exact_games() {
    let db = with_play_reactions_as_recorded(load_db());
    let text = fs::read_to_string(games_fixture_path())
        .expect("games.json — run regen_h0_exact_games once on main");
    let pin: Value = serde_json::from_str(&text).expect("games json");
    let games = pin["games"].as_array().expect("games array");
    assert_eq!(games.len(), game_plan().len());
    let live = collect_all(&db);
    for (p, l) in games.iter().zip(live.iter()) {
        compare_game(p, l);
    }
}

/// Run once on `main` (`336d625` or earlier engine) with `H0_EXACT_GAMES_REGEN=1`.
#[test]
#[ignore]
fn regen_h0_exact_games() {
    if std::env::var("H0_EXACT_GAMES_REGEN").as_deref() != Ok("1") {
        eprintln!("set H0_EXACT_GAMES_REGEN=1 to rewrite fixtures/h0_exact/games.json");
        return;
    }
    let db = with_play_reactions_as_recorded(load_db());
    let live = collect_all(&db);
    let games: Vec<Value> = live.iter().map(capture_to_json).collect();
    let path = games_fixture_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({ "games": games })).unwrap(),
    )
    .unwrap();
    eprintln!("wrote {} ({} games)", path.display(), games.len());
}

#[test]
#[ignore]
fn bench_h0_exact_games() {
    if std::env::var("H0_EXACT_GAMES_BENCH").as_deref() != Ok("1") {
        eprintln!("set H0_EXACT_GAMES_BENCH=1 to time S1's four games");
        return;
    }
    let db = load_db();
    let meta = meta_games();
    let t0 = Instant::now();
    for (i, seed) in [71001u64, 71002, 71003, 71004].iter().enumerate() {
        let plan = PlannedGame {
            spec_key: "S1",
            spec: S1,
            seed: *seed,
        };
        let row = meta_row(&meta, *seed);
        let g0 = Instant::now();
        let cap = run_game(&db, &plan, row);
        eprintln!(
            "{}/4 S1:{} n_decisions={} winner={:?} {:.2}s",
            i + 1,
            seed,
            cap.n_decisions,
            cap.winner,
            g0.elapsed().as_secs_f64()
        );
    }
    eprintln!(
        "bench_h0_exact_games S1 total {:.3}s",
        t0.elapsed().as_secs_f64()
    );
}
