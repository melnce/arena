//! PyO3 bindings for `arena-engine`. JSON at the boundary; dicts in Python.

use pyo3::prelude::*;

mod convert;
mod game;
mod matchup;
mod play;

use game::{load_cards, play_random, PyCardDb, PyGame};
use matchup::py_matchup;

pyo3::create_exception!(arena, Illegal, pyo3::exceptions::PyException);

pub fn hash_u64(state: &arena_engine::State) -> u64 {
    arena_engine::hash(state)
}

#[pymodule]
fn arena(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("Illegal", m.py().get_type::<Illegal>())?;
    m.add_class::<PyCardDb>()?;
    m.add_class::<PyGame>()?;
    m.add_function(wrap_pyfunction!(load_cards, m)?)?;
    m.add_function(wrap_pyfunction!(play_random, m)?)?;
    m.add_function(wrap_pyfunction!(py_matchup, m)?)?;
    Ok(())
}
