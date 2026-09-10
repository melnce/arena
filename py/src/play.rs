//! Shared random-legal / first-legal self-play (same policy as `arena-bench`).

use arena_engine::{
    apply, legal_actions, new_game, policy_rng, CardDb, CardId, First, GameConfig, Phase, PlayerId,
    State,
};

pub const TURN_CAP: u32 = 60;
pub const ACTION_CAP: u32 = 800;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Random,
    FirstLegal,
}

impl Policy {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "random" => Ok(Policy::Random),
            "first-legal" => Ok(Policy::FirstLegal),
            other => Err(format!(
                "policy must be 'random' or 'first-legal' (got {other})"
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayResult {
    pub winner: Option<PlayerId>,
    pub turns: u32,
    pub actions: u32,
    pub first: PlayerId,
}

pub fn play_one(
    db: &CardDb,
    seed: u64,
    deck_a: &[CardId],
    deck_b: &[CardId],
    first: First,
    policy: Policy,
) -> Result<PlayResult, String> {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b: deck_b.to_vec(),
            first,
            opening_hands: None,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(drive_policy(db, &mut state, seed, policy))
}

pub fn drive_policy(db: &CardDb, state: &mut State, seed: u64, policy: Policy) -> PlayResult {
    let first = state.first;
    let mut rng = policy_rng(seed);
    let mut nact = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        if state.turn > TURN_CAP || nact >= ACTION_CAP {
            break;
        }
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            break;
        }
        let idx = match policy {
            Policy::FirstLegal => 0,
            Policy::Random => rng.gen_range(legal.len() as u32) as usize,
        };
        if apply(db, state, legal[idx].clone()).is_err() {
            break;
        }
        nact += 1;
    }
    PlayResult {
        winner: state.winner,
        turns: state.turn,
        actions: nact,
        first,
    }
}

/// `hash(seed, pair index, game index)` — FNV-1a 64 of the three little-endian words.
pub fn game_seed(seed: u64, pair_index: u64, game_index: u64) -> u64 {
    let mut bytes = [0u8; 24];
    bytes[0..8].copy_from_slice(&seed.to_le_bytes());
    bytes[8..16].copy_from_slice(&pair_index.to_le_bytes());
    bytes[16..24].copy_from_slice(&game_index.to_le_bytes());
    arena_engine::trace::fnv1a64(&bytes)
}
