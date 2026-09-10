//! Shared test harness. Not a `#[cfg(test)]` unit; integration tests `mod common`.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use arena_engine::{
    apply, legal_actions, new_game, Action, CardDb, CardId, CardInstance, First, GameConfig, Phase,
    PlayerId, State,
};

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn repo_root() -> PathBuf {
    crate_dir().parent().unwrap().to_path_buf()
}

pub fn fixtures_dir() -> PathBuf {
    crate_dir().join("tests/fixtures")
}

pub fn load_db() -> CardDb {
    let mut db = CardDb::load(repo_root()).expect("CardDb");
    db.load_extra_dir(fixtures_dir().join("cards"))
        .expect("fixtures");
    db
}

pub fn cid(s: &str) -> CardId {
    CardId::parse(s).unwrap_or_else(|| panic!("bad id {s}"))
}

pub fn pad_deck(ids: &[&str], n: usize) -> Vec<CardId> {
    let mut v: Vec<CardId> = ids.iter().map(|s| cid(s)).collect();
    let pad = cid("88001110");
    while v.len() < n {
        v.push(pad);
    }
    v
}

/// New game, both players keep their opening four, first player to act is A.
pub fn started(db: &CardDb, seed: u64) -> State {
    started_decks(db, seed, &["88001110"], &["88001110"])
}

pub fn started_decks(db: &CardDb, seed: u64, a: &[&str], b: &[&str]) -> State {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: pad_deck(a, 40),
            deck_b: pad_deck(b, 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull A");
    apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull B");
    assert!(matches!(state.phase, Phase::Main));
    state
}

pub fn give_pp(state: &mut State, who: PlayerId, pp: i32, pp_max: i32) {
    let p = state.player_mut(who);
    p.pp_max = pp_max;
    p.pp = pp;
}

pub fn put_hand(db: &CardDb, state: &mut State, who: PlayerId, id: &str) -> u8 {
    let card = db.card(cid(id)).expect("card");
    let inst = CardInstance::from_card(card, state.alloc_id());
    if state.player(who).hand.len() >= 9 {
        // same path as engine overflow — ruling 2026-08-10
        state.player_mut(who).shadows += 1;
        state.player_mut(who).cemetery.push(inst);
        return state.player(who).hand.len() as u8;
    }
    let pos = state.player(who).hand.len() as u8;
    state.player_mut(who).hand.push(inst);
    pos
}

pub fn put_field(db: &CardDb, state: &mut State, who: PlayerId, id: &str) -> u8 {
    let card = db.card(cid(id)).expect("card");
    let mut inst = CardInstance::from_card(card, state.alloc_id());
    inst.flags.summoning_sick = false;
    let slot = state.player(who).first_empty_slot().expect("field slot");
    if inst.is_earth_sigil() {
        state.player_mut(who).earth_slot = Some(slot);
        if state.player(who).earth == 0 {
            state.player_mut(who).earth = 1;
        }
    }
    state.player_mut(who).field[slot as usize] = Some(inst);
    slot
}

pub fn play(db: &CardDb, state: &mut State, hand: u8) {
    apply(db, state, Action::Play { hand }).unwrap_or_else(|e| panic!("play {hand}: {e}"));
}

pub fn play_id(db: &CardDb, state: &mut State, who: PlayerId, id: &str) {
    let h = put_hand(db, state, who, id);
    play(db, state, h);
}

pub fn end_turn(db: &CardDb, state: &mut State) {
    apply(db, state, Action::EndTurn).expect("end turn");
}

pub fn choose(db: &CardDb, state: &mut State, i: u8) {
    apply(db, state, Action::Choose(i)).unwrap_or_else(|e| panic!("choose {i}: {e}"));
}

pub fn confirm(db: &CardDb, state: &mut State) {
    apply(db, state, Action::Confirm).expect("confirm");
}

pub fn hand_has(state: &State, who: PlayerId, id: &str) -> bool {
    let id = cid(id);
    state.player(who).hand.iter().any(|c| c.card == id)
}

pub fn field_has(state: &State, who: PlayerId, id: &str) -> bool {
    let id = cid(id);
    state
        .player(who)
        .field
        .iter()
        .flatten()
        .any(|c| c.card == id)
}

pub fn field_count(state: &State, who: PlayerId) -> usize {
    state.player(who).field_count()
}

pub fn clear_hand(state: &mut State, who: PlayerId) {
    state.player_mut(who).hand.clear();
}

/// Advance `turns` full turns for `who` (EndTurn twice per increment of their count).
pub fn skip_to_player_turn(db: &CardDb, state: &mut State, who: PlayerId, turns: u32) {
    while state.player(who).turns_taken < turns || state.active != who {
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            break;
        }
        if legal.iter().any(|a| matches!(a, Action::EndTurn)) {
            end_turn(db, state);
        } else if matches!(state.phase, Phase::Choice { .. }) {
            choose(db, state, 0);
        } else {
            apply(db, state, legal[0].clone()).ok();
        }
    }
}

pub fn load_deck_file(path: impl AsRef<Path>) -> Vec<CardId> {
    let p = path.as_ref();
    let p = if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo_root().join(p)
    };
    let text = std::fs::read_to_string(&p).unwrap_or_else(|_| "{}".into());
    let map: std::collections::BTreeMap<String, u32> =
        serde_json::from_str(&text).unwrap_or_default();
    let mut ids = Vec::new();
    for (k, n) in map {
        if let Some(id) = CardId::parse(&k) {
            for _ in 0..n {
                ids.push(id);
            }
        }
    }
    ids
}

pub fn deck_ready(db: &CardDb, ids: &[CardId]) -> bool {
    ids.iter().all(|c| db.has_card(*c))
}
