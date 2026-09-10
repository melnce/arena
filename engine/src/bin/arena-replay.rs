//! Replay a trace from either engine with ScriptedRng; diff per line.
//!
//! Exit 0 = green; 2 = divergence; 3 = unsupported / oracle-pick-not-legal.

use std::fs;
use std::process::ExitCode;

use arena_engine::oracle::{replay_trace, ReplayOutcome};
use arena_engine::{CardDb, ReplayError};

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
    let mut db = CardDb::load(".").map_err(|e| ReplayError::Header(e.to_string()))?;
    let _ = db.load_extra_dir("engine/tests/fixtures/cards");
    match replay_trace(&db, &text)? {
        ReplayOutcome::Green => Ok(()),
        ReplayOutcome::Divergence(d) => Err(ReplayError::Diverge {
            i: d.i,
            path: d.path,
            arena: d.arena,
            trace: d.trace,
        }),
    }
}
