//! PyO3 bindings for `arena-engine`. JSON at the boundary; dicts in Python.

use pyo3::prelude::*;

mod convert;
mod game;
mod lethal;
mod matchup;
mod play;

use convert::{deck_counts_from_value, py_to_value};
use game::{load_cards, play_random, PyCardDb, PyGame};
use lethal::py_forced_lethal;
use matchup::py_matchup;

pyo3::create_exception!(arena, Illegal, pyo3::exceptions::PyException);

pub fn hash_u64(state: &arena_engine::State) -> u64 {
    arena_engine::hash(state)
}

#[pyfunction]
fn deck_fingerprint(deck: Bound<'_, PyAny>) -> PyResult<String> {
    let v = py_to_value(&deck)?;
    let counts = deck_counts_from_value(&v)?;
    Ok(arena_engine::deck_fingerprint_counts(&counts))
}

#[pymodule]
fn arena(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("Illegal", m.py().get_type::<Illegal>())?;
    m.add_class::<PyCardDb>()?;
    m.add_class::<PyGame>()?;
    m.add_function(wrap_pyfunction!(load_cards, m)?)?;
    m.add_function(wrap_pyfunction!(play_random, m)?)?;
    m.add_function(wrap_pyfunction!(py_matchup, m)?)?;
    m.add_function(wrap_pyfunction!(py_forced_lethal, m)?)?;
    m.add_function(wrap_pyfunction!(deck_fingerprint, m)?)?;
    Ok(())
}
