//! Replay a trace from either engine with ScriptedRng; diff per line.
//!
//! Exit 0 = green; 2 = divergence; 3 = unsupported / oracle-pick-not-legal.

use std::fs;
use std::process::ExitCode;

use arena_engine::{
    apply_neutral, legal_actions_neutral, legal_divergence_parts, neutral_json, new_game,
    picks_from_trace_rng, snapshot_json, CardDb, CardId, First, GameConfig, GameRng, NeutralAction,
    OpeningHands, ReplayError, TraceHeader,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned().unwrap_or_default();
    if path.is_empty() {
        eprintln!("usage: arena-replay <trace.jsonl> [--from-engine practice-tool|arena]");
        return ExitCode::from(1);
    }
    match run(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

fn run(path: &str) -> Result<(), ReplayError> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| ReplayError::Header("empty".into()))?;
    let header: TraceHeader = serde_json::from_str(header_line)
        .map_err(|source| ReplayError::Parse { line: 1, source })?;
    let mut db = CardDb::load(".").map_err(|e| ReplayError::Header(e.to_string()))?;
    let _ = db.load_extra_dir("engine/tests/fixtures/cards");
    let deck_a = parse_ids(&header.deck_a);
    let deck_b = parse_ids(&header.deck_b);
    let first = if header.first == "b" {
        First::B
    } else {
        First::A
    };
    let opening = OpeningHands {
        a: parse_ids(&header.opening_hands.a),
        b: parse_ids(&header.opening_hands.b),
    };
    let mut state = new_game(
        &db,
        GameConfig {
            seed: header.seed,
            deck_a,
            deck_b,
            first,
            opening_hands: Some(opening),
        },
    )
    .map_err(|e| match e {
        arena_engine::LoadError::Unsupported(u) => ReplayError::Unsupported(u),
        other => ReplayError::Header(other.to_string()),
    })?;
    for (ln, line) in lines.enumerate() {
        let rec: serde_json::Value =
            serde_json::from_str(line).map_err(|source| ReplayError::Parse {
                line: ln + 2,
                source,
            })?;
        let i = rec.get("i").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let action: NeutralAction = serde_json::from_value(
            rec.get("action").cloned().unwrap_or_default(),
        )
        .map_err(|source| ReplayError::Parse {
            line: ln + 2,
            source,
        })?;
        let rng = rec.get("rng").map(picks_from_trace_rng).unwrap_or_default();
        state.rng = GameRng::scripted(rng, header.seed);
        apply_neutral(&db, &mut state, &action).map_err(|e| match e {
            arena_engine::Illegal::Unsupported(u) => ReplayError::Unsupported(u),
            arena_engine::Illegal::OraclePickNotLegal(o) => ReplayError::Oracle(o),
            other => illegal_at(i, &action, &legal_json(&db, &state), other),
        })?;
        let got = snapshot_json(&state);
        let want = rec.get("state").cloned().unwrap_or(serde_json::Value::Null);
        if let Some((path, a, b)) = arena_engine::json_eq_first_diff(&got, &want, "") {
            return Err(ReplayError::Diverge {
                i,
                path,
                arena: a,
                trace: b,
            });
        }
        if let Some(legal) = rec.get("legal") {
            let mut ours: Vec<NeutralAction> = legal_actions_neutral(&db, &state);
            let mut theirs: Vec<NeutralAction> =
                serde_json::from_value(legal.clone()).unwrap_or_default();
            ours.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
            theirs.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
            if ours != theirs {
                let (arena, trace) = legal_divergence_parts(&ours, &theirs);
                return Err(ReplayError::Diverge {
                    i,
                    path: "legal".into(),
                    arena,
                    trace,
                });
            }
        }
    }
    Ok(())
}

fn legal_json(db: &CardDb, state: &arena_engine::State) -> String {
    let acts: Vec<String> = legal_actions_neutral(db, state)
        .iter()
        .map(neutral_json)
        .collect();
    format!("[{}]", acts.join(", "))
}

fn illegal_at(
    i: u32,
    action: &NeutralAction,
    legal: &str,
    source: arena_engine::Illegal,
) -> ReplayError {
    ReplayError::Illegal {
        i,
        action: neutral_json(action),
        legal: legal.to_string(),
        source,
    }
}

fn parse_ids(v: &[String]) -> Vec<CardId> {
    v.iter().filter_map(|s| CardId::parse(s)).collect()
}
