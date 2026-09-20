//! PyO3 export of [`arena_engine::forced_lethal`].

use arena_engine::{forced_lethal, to_neutral, LethalVerdict};
use pyo3::prelude::*;

use crate::convert::{player_str, value_to_py};
use crate::game::PyGame;

/// Call the exhaustive within-turn solver on a live [`PyGame`].
///
/// Returns a dict with `verdict` (`"lethal"` / `"none"` / `"unknown"`),
/// `nodes`, and — only on a kill — `line` (NeutralAction dicts) and
/// `rng_dependent`. There is no boolean helper.
#[pyfunction]
#[pyo3(name = "forced_lethal", signature = (game, budget))]
pub fn py_forced_lethal<'py>(
    py: Python<'py>,
    game: &PyGame,
    budget: u32,
) -> PyResult<Bound<'py, PyAny>> {
    let state = game.engine_state();
    let verdict = forced_lethal(game.engine_db(), state, budget);
    let body = match verdict {
        LethalVerdict::Lethal {
            line,
            nodes,
            rng_dependent,
        } => {
            let mut walk = state.clone();
            let mut acts = Vec::with_capacity(line.len());
            for a in line {
                let n = to_neutral(&walk, &a);
                let _ = arena_engine::apply(game.engine_db(), &mut walk, a);
                acts.push(serde_json::to_value(n).unwrap_or(serde_json::Value::Null));
            }
            serde_json::json!({
                "verdict": "lethal",
                "nodes": nodes,
                "line": acts,
                "rng_dependent": rng_dependent,
                "perspective": player_str(arena_engine::acting_player(state)),
            })
        }
        LethalVerdict::None { nodes } => serde_json::json!({
            "verdict": "none",
            "nodes": nodes,
            "perspective": player_str(arena_engine::acting_player(state)),
        }),
        LethalVerdict::Unknown { nodes } => serde_json::json!({
            "verdict": "unknown",
            "nodes": nodes,
            "perspective": player_str(arena_engine::acting_player(state)),
        }),
    };
    value_to_py(py, &body)
}
