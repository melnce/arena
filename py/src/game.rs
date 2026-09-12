//! `CardDb` and `Game` pyclasses.

use std::sync::Arc;

use arena_engine::trace::NeutralAction;
use arena_engine::trace::Pick;
use arena_engine::{
    apply_neutral, legal_actions_neutral, new_game, snapshot_json, CardDb, GameConfig, GameRng,
    Phase,
};
use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::convert::{
    deck_from_value, event_to_value, full_state_json, opening_hands_from_value, parse_first,
    phase_str, player_str, py_err_illegal, py_err_load, py_err_msg, py_to_value,
    resolve_cards_root, value_to_py,
};
use crate::play::play_one;

#[pyclass(module = "arena", name = "CardDb")]
#[derive(Clone)]
pub struct PyCardDb {
    pub inner: Arc<CardDb>,
}

#[pyclass(module = "arena", name = "Game")]
#[derive(Clone)]
pub struct PyGame {
    db: Arc<CardDb>,
    state: arena_engine::State,
    seed: u64,
}

#[pyfunction]
#[pyo3(signature = (root = "cards"))]
pub fn load_cards(root: &str) -> PyResult<PyCardDb> {
    crate::convert::assert_send_sync();
    let resolved = resolve_cards_root(root);
    let db = CardDb::load(&resolved).map_err(py_err_load)?;
    // `new_game` calls `require_supported` on every deck id. Walking here
    // surfaces load-time unsupported constructs without failing the whole
    // pool (M2/M3 files may still stub constructs).
    for id in db.cards.keys().copied() {
        let _ = db.require_supported(id);
    }
    Ok(PyCardDb {
        inner: Arc::new(db),
    })
}

#[pymethods]
impl PyGame {
    #[new]
    #[pyo3(signature = (db, seed, deck_a, deck_b, first = "coin", opening_hands = None))]
    fn new(
        db: &PyCardDb,
        seed: u64,
        deck_a: Bound<'_, PyAny>,
        deck_b: Bound<'_, PyAny>,
        first: &str,
        opening_hands: Option<Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let da = deck_from_value(&py_to_value(&deck_a)?)?;
        let dbk = deck_from_value(&py_to_value(&deck_b)?)?;
        let opening = match opening_hands {
            Some(h) if !h.is_none() => Some(opening_hands_from_value(&py_to_value(&h)?)?),
            _ => None,
        };
        let state = new_game(
            &db.inner,
            GameConfig {
                seed,
                deck_a: da,
                deck_b: dbk,
                first: parse_first(first)?,
                opening_hands: opening,
            },
        )
        .map_err(py_err_load)?;
        Ok(Self {
            db: db.inner.clone(),
            state,
            seed,
        })
    }

    /// NeutralAction dicts, engine order (`legal_actions_neutral`).
    fn legal<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let acts = legal_actions_neutral(&self.db, &self.state);
        let vals: Vec<serde_json::Value> = acts
            .iter()
            .map(|a| serde_json::to_value(a).unwrap_or(serde_json::Value::Null))
            .collect();
        value_to_py(py, &serde_json::Value::Array(vals))
    }

    /// Apply a NeutralAction dict. Optional `rng` is that action line's pick
    /// array (`docs/trace-format.md`) — used to replay committed traces.
    #[pyo3(signature = (action, rng = None))]
    fn apply<'py>(
        &mut self,
        py: Python<'py>,
        action: Bound<'_, PyAny>,
        rng: Option<Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let v = py_to_value(&action)?;
        let act: NeutralAction =
            serde_json::from_value(v).map_err(|e| py_err_msg(format!("invalid action: {e}")))?;
        if let Some(r) = rng {
            if !r.is_none() {
                let picks = picks_from_py(&r)?;
                self.state.rng = GameRng::scripted(picks, self.seed);
            }
        }
        let events = apply_neutral(&self.db, &mut self.state, &act).map_err(py_err_illegal)?;
        let vals: Vec<serde_json::Value> = events.iter().map(event_to_value).collect();
        value_to_py(py, &serde_json::Value::Array(vals))
    }

    fn snapshot<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        value_to_py(py, &snapshot_json(&self.state))
    }

    fn full<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        value_to_py(py, &full_state_json(&self.state))
    }

    fn hash(&self) -> u64 {
        arena_engine::hash(&self.state)
    }

    #[getter]
    fn phase(&self) -> &'static str {
        phase_str(&self.state.phase)
    }

    #[getter]
    fn active(&self) -> &'static str {
        player_str(self.state.active)
    }

    #[getter]
    fn turn(&self) -> u32 {
        self.state.turn
    }

    #[getter]
    fn winner(&self) -> Option<&'static str> {
        self.state.winner.map(player_str)
    }

    #[getter]
    fn terminal(&self) -> bool {
        self.state.winner.is_some() || matches!(self.state.phase, Phase::Terminal)
    }

    fn clone(&self) -> Self {
        Clone::clone(self)
    }

    fn __repr__(&self) -> String {
        format!(
            "Game(seed={}, turn={}, phase={}, active={}, winner={:?}, terminal={})",
            self.seed,
            self.state.turn,
            phase_str(&self.state.phase),
            player_str(self.state.active),
            self.state.winner.map(player_str),
            self.terminal()
        )
    }
}

fn picks_from_py(obj: &Bound<'_, PyAny>) -> PyResult<Vec<Pick>> {
    let v = py_to_value(obj)?;
    Ok(arena_engine::picks_from_trace_rng(&v))
}

#[pyfunction]
#[pyo3(signature = (db, seed, deck_a, deck_b, first = "coin"))]
pub fn play_random<'py>(
    py: Python<'py>,
    db: &PyCardDb,
    seed: u64,
    deck_a: Bound<'_, PyAny>,
    deck_b: Bound<'_, PyAny>,
    first: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let da = deck_from_value(&py_to_value(&deck_a)?)?;
    let dbk = deck_from_value(&py_to_value(&deck_b)?)?;
    let result = play_one(
        &db.inner,
        seed,
        &da,
        &dbk,
        parse_first(first)?,
        "random",
        "random",
    )
    .map_err(py_err_msg)?;
    let winner = result
        .winner
        .map(|w| serde_json::Value::String(w.as_str().into()))
        .unwrap_or(serde_json::Value::Null);
    value_to_py(
        py,
        &serde_json::json!({
            "winner": winner,
            "turns": result.turns,
            "actions": result.actions,
            "first": result.first.as_str(),
        }),
    )
}
