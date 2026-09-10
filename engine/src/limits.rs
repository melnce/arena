//! Turn / action ceilings shared by bench, soak, and search rollouts.
//! The `py/` crate keeps its own copies until the M5 Python sibling switches.

pub const MAX_TURNS: u32 = 60;
pub const MAX_ACTIONS: u32 = 800;
