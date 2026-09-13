//! Shared self-play via `engine::play::play_game`.

use arena_engine::{
    new_game, play_game, policy_rng, AnyPolicy, CardDb, CardId, First, GameConfig, Outcome,
    PlayerId, Recorder, Sample,
};

pub fn play_one(
    db: &CardDb,
    seed: u64,
    deck_a: &[CardId],
    deck_b: &[CardId],
    first: First,
    pol_a: &str,
    pol_b: &str,
) -> Result<Outcome, String> {
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
    let mut a = AnyPolicy::parse_spec(pol_a)?;
    let mut b = AnyPolicy::parse_spec(pol_b)?;
    let mut rng = policy_rng(seed);
    Ok(play_game(db, &mut state, &mut a, &mut b, &mut rng))
}

/// Same as [`play_one`], wrapping both seats in [`Recorder`]s. Labels both
/// buffers from the outcome. `epsilon = 0` is byte-identical to [`play_one`].
#[allow(clippy::too_many_arguments)]
pub fn play_one_export(
    db: &CardDb,
    seed: u64,
    deck_a: &[CardId],
    deck_b: &[CardId],
    first: First,
    pol_a: &str,
    pol_b: &str,
    epsilon: f32,
) -> Result<(Outcome, Vec<Sample>, Vec<Sample>), String> {
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
    let mut a =
        Recorder::new(Box::new(AnyPolicy::parse_spec(pol_a)?), PlayerId::A).with_epsilon(epsilon);
    let mut b =
        Recorder::new(Box::new(AnyPolicy::parse_spec(pol_b)?), PlayerId::B).with_epsilon(epsilon);
    let mut rng = policy_rng(seed);
    let out = play_game(db, &mut state, &mut a, &mut b, &mut rng);
    a.label(&out);
    b.label(&out);
    Ok((out, a.take(), b.take()))
}

/// `hash(seed, pair index, game index)` — FNV-1a 64 of the three little-endian words.
pub fn game_seed(seed: u64, pair_index: u64, game_index: u64) -> u64 {
    let mut bytes = [0u8; 24];
    bytes[0..8].copy_from_slice(&seed.to_le_bytes());
    bytes[8..16].copy_from_slice(&pair_index.to_le_bytes());
    bytes[16..24].copy_from_slice(&game_index.to_le_bytes());
    arena_engine::trace::fnv1a64(&bytes)
}
