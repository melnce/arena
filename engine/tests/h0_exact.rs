//! Exact h0 decisions. Pins were generated once on `main` (before the line-key
//! change) by `regen_h0_exact_pins`. Do not regenerate them.
//!
//! Meta roots: 8 games, both seats `h0:nodes=1500,k=2`, two decisions each from
//! turn 4 with at least 4 legal actions. Action traces are in
//! `fixtures/h0_exact/meta_games.json`.
//!
//! Portal fuse (artifact-token `Fuse` on meta-portal-af; no meta-portal deck
//! lists a card with a `fuse` field — only `meta-rune-test-subject` does):
//! `meta:71001:28` (portal-af vs portal-cutthroat) and `meta:72022:22`
//! (portal-af vs abyss-aggro).

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::{
    apply, apply_neutral, legal_actions, new_game, policy_rng, search_key, AnyPolicy, CardDb,
    ChoiceNode, First, GameConfig, Phase, Policy, H0,
};
use serde_json::{json, Value};

mod common;
use common::*;

const S1: &str = "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=builtin,oorder=1,oopen=1,onext=40";

fn spec_list() -> Vec<(&'static str, String, u32)> {
    let s2 = S1.replace("okill=8", "okill=15");
    let s3 = S1
        .replace("k=8,", "k=16,")
        .replace("threads=1", "threads=4");
    let s4 = S1.replace("k=8,", "k=8,info=all,");
    vec![
        ("S1", S1.to_string(), 1),
        ("S2", s2, 1),
        ("S3", s3, 4),
        ("S4", s4, 1),
    ]
}

fn review_fixture(review: &str, name: &str) -> PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(name)
}

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn deck_from_json(v: &Value) -> Vec<arena_engine::CardId> {
    let map = v.as_object().expect("deck object");
    let mut sorted = std::collections::BTreeMap::new();
    for (id, n) in map {
        sorted.insert(id.clone(), n.as_u64().unwrap_or(0) as usize);
    }
    let mut out = Vec::new();
    for (id, count) in sorted {
        let cid = arena_engine::CardId::parse(&id).unwrap_or_else(|| panic!("bad id {id}"));
        out.extend(std::iter::repeat_n(cid, count));
    }
    out
}

fn replay_capture(db: &CardDb, cap: &Value, n: usize) -> arena_engine::State {
    let seed = cap["seed"].as_u64().expect("seed");
    let deck_a = deck_from_json(&cap["deckA"]);
    let deck_b = deck_from_json(&cap["deckB"]);
    let first = match cap.get("first").and_then(|v| v.as_str()) {
        Some("b") | Some("B") => First::B,
        Some("a") | Some("A") => First::A,
        _ => First::Coin,
    };
    let mut st = new_game(
        db,
        GameConfig {
            seed,
            deck_a,
            deck_b,
            first,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let actions = cap["actions"].as_array().expect("actions");
    for step in actions.iter().take(n) {
        if step.get("reseed").is_some() {
            st.reseed(step["reseed"].as_u64().expect("reseed"));
            continue;
        }
        let mut body = serde_json::Map::new();
        for (k, v) in step.as_object().expect("action object") {
            if k == "value" || k == "bot_value" {
                continue;
            }
            body.insert(k.clone(), v.clone());
        }
        let neu: arena_engine::NeutralAction =
            serde_json::from_value(Value::Object(body)).expect("action");
        apply_neutral(db, &mut st, &neu).expect("apply_neutral");
    }
    st
}

fn policy_seed(cap: &Value, ply: usize) -> u64 {
    let actions = cap["actions"].as_array().expect("actions");
    let bot_steps = actions
        .iter()
        .take(ply)
        .any(|s| s.get("mulligan").is_some() || s.get("bot_value").is_some());
    if bot_steps {
        let base = cap["seed"].as_u64().expect("seed");
        let mut n = 0u64;
        for step in actions.iter().take(ply) {
            if step.get("mulligan").is_some() || step.get("bot_value").is_some() {
                n += 1;
            }
        }
        base + n
    } else {
        cap["seed"].as_u64().expect("seed") + ply as u64
    }
}

struct ReviewMoment {
    review: &'static str,
    game: &'static str,
    ply: usize,
}

const REVIEW_MOMENTS: [ReviewMoment; 12] = [
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 27,
    },
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 45,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 69,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 40,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 27,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 52,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 91,
    },
    ReviewMoment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 15,
    },
    ReviewMoment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 35,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 60,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 55,
    },
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 60,
    },
];

struct MetaMoment {
    seed: u64,
    ply: usize,
}

/// Two decisions per game. Portal fuse: 71001/28 and 72022/22.
const META_MOMENTS: [MetaMoment; 16] = [
    MetaMoment {
        seed: 71001,
        ply: 28,
    },
    MetaMoment {
        seed: 71001,
        ply: 29,
    },
    MetaMoment {
        seed: 71002,
        ply: 23,
    },
    MetaMoment {
        seed: 71002,
        ply: 37,
    },
    MetaMoment {
        seed: 71003,
        ply: 24,
    },
    MetaMoment {
        seed: 71003,
        ply: 27,
    },
    MetaMoment {
        seed: 71004,
        ply: 19,
    },
    MetaMoment {
        seed: 71004,
        ply: 49,
    },
    MetaMoment {
        seed: 71005,
        ply: 21,
    },
    MetaMoment {
        seed: 71005,
        ply: 26,
    },
    MetaMoment {
        seed: 71006,
        ply: 19,
    },
    MetaMoment {
        seed: 71006,
        ply: 54,
    },
    MetaMoment {
        seed: 72022,
        ply: 22,
    },
    MetaMoment {
        seed: 72022,
        ply: 24,
    },
    MetaMoment {
        seed: 71008,
        ply: 30,
    },
    MetaMoment {
        seed: 71008,
        ply: 42,
    },
];

struct Outcome {
    idx: usize,
    value_bits: Option<u32>,
    stats: String,
    explain_len: usize,
    explain_fnv: u64,
    search_key: u64,
}

fn choice_kind(node: &ChoiceNode) -> &'static str {
    match node {
        ChoiceNode::Targets { .. } => "Targets",
        ChoiceNode::Modes { .. } => "Modes",
        ChoiceNode::Cards { .. } => "Cards",
        ChoiceNode::FusePartners { .. } => "FusePartners",
        ChoiceNode::MultiPick { .. } => "MultiPick",
    }
}

fn phase_of(state: &arena_engine::State) -> (String, Option<&'static str>) {
    match &state.phase {
        Phase::Mulligan { .. } => ("Mulligan".into(), None),
        Phase::Main => ("Main".into(), None),
        Phase::Combat => ("Combat".into(), None),
        Phase::Choice { node, .. } => ("Choice".into(), Some(choice_kind(node))),
        Phase::End => ("End".into(), None),
        Phase::Terminal => ("Terminal".into(), None),
    }
}

fn decide(
    spec: &str,
    threads: u32,
    db: &CardDb,
    state: &arena_engine::State,
    seed: u64,
) -> Outcome {
    let legal = legal_actions(db, state);
    let mut h0 = parse_h0(spec);
    h0.threads = threads;
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, state, &legal, &mut rng);
    let explain = h0
        .take_explain()
        .map(|r| serde_json::to_string(&r).expect("explain json"))
        .expect("explain armed");
    Outcome {
        idx,
        value_bits: h0.last_value().map(f32::to_bits),
        stats: serde_json::to_string(&h0.stats).expect("stats"),
        explain_len: explain.len(),
        explain_fnv: arena_engine::trace::fnv1a64(explain.as_bytes()),
        search_key: search_key(state),
    }
}

fn pins_path() -> PathBuf {
    fixtures_dir().join("h0_exact/pins.json")
}

fn meta_games() -> Vec<Value> {
    let p = fixtures_dir().join("h0_exact/meta_games.json");
    let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p:?}: {e}"));
    serde_json::from_str(&text).expect("meta games")
}

fn meta_state(db: &CardDb, games: &[Value], seed: u64, ply: usize) -> arena_engine::State {
    let g = games
        .iter()
        .find(|g| g["seed"].as_u64() == Some(seed))
        .unwrap_or_else(|| panic!("no meta game {seed}"));
    let first = match g["first"].as_str() {
        Some("b") => First::B,
        Some("a") => First::A,
        other => panic!("first {other:?}"),
    };
    let mut st = new_game(
        db,
        GameConfig {
            seed,
            deck_a: load_deck_file(g["deck_a"].as_str().unwrap()),
            deck_b: load_deck_file(g["deck_b"].as_str().unwrap()),
            first,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let actions = g["actions"].as_array().expect("actions");
    for step in actions.iter().take(ply) {
        let neu: arena_engine::NeutralAction =
            serde_json::from_value(step.clone()).expect("neutral");
        apply_neutral(db, &mut st, &neu).expect("apply_neutral");
    }
    st
}

struct Root {
    id: String,
    state: arena_engine::State,
    seed: u64,
}

fn all_roots(recorded: &CardDb, live: &CardDb) -> Vec<Root> {
    let mut roots = Vec::new();
    for m in &REVIEW_MOMENTS {
        let text = fs::read_to_string(review_fixture(m.review, m.game)).expect("fixture");
        let cap: Value = serde_json::from_str(&text).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let state = replay_capture(recorded, &cap, m.ply);
        roots.push(Root {
            id: format!("review:{}:{}:{}", m.review, m.game, m.ply),
            state,
            seed,
        });
    }
    let games = meta_games();
    for m in &META_MOMENTS {
        let state = meta_state(live, &games, m.seed, m.ply);
        roots.push(Root {
            id: format!("meta:{}:{}", m.seed, m.ply),
            state,
            seed: m.seed.wrapping_add(m.ply as u64),
        });
    }
    roots
}

fn check_shape(roots: &[Root], db_for_legal: &CardDb, live: &CardDb) {
    let mut choice = 0usize;
    let mut portal_fuse = 0usize;
    for (i, r) in roots.iter().enumerate() {
        let db = if i < REVIEW_MOMENTS.len() {
            db_for_legal
        } else {
            live
        };
        let legal = legal_actions(db, &r.state);
        let (phase, choice_kind) = phase_of(&r.state);
        if i >= REVIEW_MOMENTS.len() {
            assert!(r.state.turn >= 4, "{} turn {}", r.id, r.state.turn);
            assert!(legal.len() >= 4, "{} legal {}", r.id, legal.len());
        }
        if phase == "Choice" {
            assert!(
                matches!(choice_kind, Some("Targets" | "Modes")),
                "{} choice {choice_kind:?}",
                r.id
            );
            choice += 1;
        }
        let fuse = legal
            .iter()
            .any(|a| matches!(a, arena_engine::Action::Fuse { .. }));
        if fuse && (r.id.starts_with("meta:71001:") || r.id.starts_with("meta:72022:")) {
            portal_fuse += 1;
        }
    }
    assert!(choice >= 4, "choice roots {choice}");
    assert!(portal_fuse >= 2, "portal fuse roots {portal_fuse}");
    assert_eq!(roots.len(), 28);
}

fn outcome_json(o: &Outcome) -> Value {
    json!({
        "idx": o.idx,
        "value_bits": o.value_bits,
        "stats": o.stats,
        "explain_len": o.explain_len,
        "explain_fnv": o.explain_fnv.to_string(),
    })
}

fn collect(roots: &[Root], recorded: &CardDb, live: &CardDb) -> Value {
    let specs = spec_list();
    let mut moments = Vec::new();
    for (mi, r) in roots.iter().enumerate() {
        let db = if mi < REVIEW_MOMENTS.len() {
            recorded
        } else {
            live
        };
        let mut by_spec = serde_json::Map::new();
        let mut sk = None;
        for (name, spec, threads) in &specs {
            let t0 = Instant::now();
            let o = decide(spec, *threads, db, &r.state, r.seed);
            eprintln!(
                "{}/28 {} {} idx={} {:.2}s",
                mi + 1,
                r.id,
                name,
                o.idx,
                t0.elapsed().as_secs_f64()
            );
            if let Some(prev) = sk {
                assert_eq!(
                    prev, o.search_key,
                    "{} search_key drifted across specs",
                    r.id
                );
            }
            sk = Some(o.search_key);
            by_spec.insert(name.to_string(), outcome_json(&o));
        }
        moments.push(json!({
            "id": r.id,
            "search_key": sk.unwrap().to_string(),
            "by_spec": Value::Object(by_spec),
        }));
    }
    let mut spec_map = serde_json::Map::new();
    for (name, spec, _) in &specs {
        spec_map.insert(name.to_string(), Value::String(spec.clone()));
    }
    json!({
        "specs": Value::Object(spec_map),
        "moments": moments,
    })
}

fn first_diff(field: &str, pin: &str, live: &str) -> String {
    if pin == live {
        return String::new();
    }
    let n = pin
        .chars()
        .zip(live.chars())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| pin.len().min(live.len()));
    let pin_snip: String = pin.chars().skip(n.saturating_sub(40)).take(80).collect();
    let live_snip: String = live.chars().skip(n.saturating_sub(40)).take(80).collect();
    format!("{field} at char {n}\n  pin  …{pin_snip}…\n  live …{live_snip}…")
}

fn compare_pins(pin: &Value, live: &Value) {
    let pin_m = pin["moments"].as_array().expect("pin moments");
    let live_m = live["moments"].as_array().expect("live moments");
    assert_eq!(pin_m.len(), live_m.len(), "moment count");
    for (p, l) in pin_m.iter().zip(live_m.iter()) {
        let id = p["id"].as_str().unwrap();
        assert_eq!(id, l["id"].as_str().unwrap(), "moment id order");
        let pk = p["search_key"].as_str().unwrap();
        let lk = l["search_key"].as_str().unwrap();
        assert_eq!(pk, lk, "{id} search_key pin {pk} live {lk}");
        for (name, _, _) in spec_list() {
            let ps = &p["by_spec"][name];
            let ls = &l["by_spec"][name];
            if ps["idx"] != ls["idx"] {
                panic!("{id} {name} idx pin {} live {}", ps["idx"], ls["idx"]);
            }
            if ps["value_bits"] != ls["value_bits"] {
                panic!(
                    "{id} {name} value_bits pin {} live {}",
                    ps["value_bits"], ls["value_bits"]
                );
            }
            if ps["explain_len"] != ls["explain_len"] {
                panic!(
                    "{id} {name} explain_len pin {} live {}",
                    ps["explain_len"], ls["explain_len"]
                );
            }
            if ps["explain_fnv"] != ls["explain_fnv"] {
                panic!(
                    "{id} {name} explain_fnv pin {} live {}",
                    ps["explain_fnv"], ls["explain_fnv"]
                );
            }
            let pd = ps["stats"].as_str().unwrap();
            let ld = ls["stats"].as_str().unwrap();
            if pd != ld {
                panic!("{id} {name} stats {}", first_diff("stats", pd, ld));
            }
        }
    }
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn h0_exact_pins() {
    let recorded = load_recorded_db();
    let live = load_db();
    let roots = all_roots(&recorded, &live);
    check_shape(&roots, &recorded, &live);
    let text = fs::read_to_string(pins_path()).expect("pins.json — run regen_h0_exact_pins once");
    let pin: Value = serde_json::from_str(&text).expect("pins json");
    let live_pins = collect(&roots, &recorded, &live);
    compare_pins(&pin, &live_pins);
}

#[test]
#[ignore]
fn regen_h0_exact_pins() {
    let recorded = load_recorded_db();
    let live = load_db();
    let roots = all_roots(&recorded, &live);
    check_shape(&roots, &recorded, &live);
    let live_pins = collect(&roots, &recorded, &live);
    let path = pins_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, serde_json::to_string_pretty(&live_pins).unwrap()).unwrap();
    eprintln!("wrote {}", path.display());
}

#[test]
#[ignore]
fn bench_h0_exact() {
    let recorded = load_recorded_db();
    let live = load_db();
    let roots = all_roots(&recorded, &live);
    let t0 = Instant::now();
    for (mi, r) in roots.iter().enumerate() {
        let db = if mi < REVIEW_MOMENTS.len() {
            &recorded
        } else {
            &live
        };
        let o = decide(S1, 1, db, &r.state, r.seed);
        eprintln!("{}/28 {} idx={} ", mi + 1, r.id, o.idx);
    }
    eprintln!("bench_h0_exact S1 total {:.3}s", t0.elapsed().as_secs_f64());
}

#[test]
fn apply_bumps_step_unless_choice() {
    let db = load_db();
    let games = [
        (
            "oracle/decks/meta-portal-af.json",
            "oracle/decks/meta-portal-cutthroat.json",
            81001u64,
            First::A,
        ),
        (
            "oracle/decks/meta-rune-test-subject.json",
            "oracle/decks/meta-portal-evo.json",
            81002,
            First::B,
        ),
        (
            "oracle/decks/meta-abyss-aggro.json",
            "oracle/decks/meta-haven-evo.json",
            81003,
            First::A,
        ),
        (
            "oracle/decks/meta-dragon-ramp.json",
            "oracle/decks/meta-forest-combo.json",
            81004,
            First::B,
        ),
        (
            "oracle/decks/meta-sword-loot.json",
            "oracle/decks/meta-rune-spell.json",
            81005,
            First::A,
        ),
        (
            "oracle/decks/meta-haven-amulet.json",
            "oracle/decks/meta-abyss-midrange.json",
            81006,
            First::B,
        ),
    ];
    let mut grew = 0usize;
    let mut choice_ok = 0usize;
    for (da, dbn, seed, first) in games {
        let mut st = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: load_deck_file(da),
                deck_b: load_deck_file(dbn),
                first,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut rng = policy_rng(seed);
        let mut steps = 0u32;
        while st.winner.is_none()
            && !matches!(st.phase, Phase::Terminal)
            && st.turn <= 12
            && steps < 200
        {
            let legal = legal_actions(&db, &st);
            if legal.is_empty() {
                break;
            }
            for a in &legal {
                let mut after = st.clone();
                if apply(&db, &mut after, a.clone()).is_err() {
                    continue;
                }
                if after.winner.is_none()
                    && !matches!(after.phase, Phase::Choice { .. } | Phase::Mulligan { .. })
                {
                    assert!(
                        after.step_counter > st.step_counter,
                        "seed {seed} step {} -> {} phase {:?}",
                        st.step_counter,
                        after.step_counter,
                        after.phase
                    );
                    grew += 1;
                }
                if matches!(after.phase, Phase::Choice { .. }) {
                    assert!(
                        after.step_counter >= st.step_counter,
                        "seed {seed} choice step went backwards"
                    );
                    choice_ok += 1;
                }
            }
            let idx = (rng.next_u64() as usize) % legal.len();
            apply(&db, &mut st, legal[idx].clone()).expect("drive");
            steps += 1;
        }
    }
    assert!(grew > 0, "no strict step bumps");
    assert!(choice_ok > 0, "no choice afterstates");
}
