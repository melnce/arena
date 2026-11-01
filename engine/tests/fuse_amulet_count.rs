//! Documents `FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION` for Socialite / Dandy.
//!
//! Flipping `arena_engine::apply::FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION` should
//! invert the expected fire count: `true` → once per Fuse action that includes
//! ≥1 amulet; `false` → once per amulet partner in that action.
//!
//! Mirrors the WORLD_OF_GAMES either-side pattern: assert against the live const.

#![allow(dead_code)]

use arena_engine::apply::FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION;
use arena_engine::{apply, Action, ChoiceNode, Phase, PlayerId};

mod common;
use common::*;

const SOCIALITE: &str = "11072110";
const DANDY: &str = "11073110";
const HAND_EXT: &str = "11072310";
const EVIDENCE: &str = "90001210";
const TANK: &str = "89500001";

fn fuse_two_amulets(db: &arena_engine::CardDb, st: &mut arena_engine::State, host: u8) {
    apply(db, st, Action::Fuse { host }).expect("fuse");
    // Pick both Evidence partners.
    for _ in 0..2 {
        let Phase::Choice {
            node: ChoiceNode::FusePartners {
                options, picked, ..
            },
            ..
        } = &st.phase
        else {
            panic!("expected FusePartners");
        };
        let next = options
            .iter()
            .copied()
            .find(|o| !picked.contains(o))
            .expect("another partner");
        let idx = options.iter().position(|&o| o == next).unwrap() as u8;
        choose(db, st, idx);
    }
    confirm(db, st);
    drain_choice(db, st);
}

#[test]
fn fuse_amulet_trigger_count_matches_const() {
    let db = load_db();
    let mut st = started(&db, 77_001);
    play_keeping_hand(&db, &mut st, PlayerId::A, SOCIALITE, 3);
    // Socialite Fanfare put Evidence in hand — rebuild hand for a clean fuse.
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);

    fuse_two_amulets(&db, &mut st, host);

    let defs: Vec<i32> = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == TANK)
        .map(|c| c.defense)
        .collect();
    // Tanks start at 10 def (89500001 is 2/10).
    if FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION {
        assert!(
            defs.iter().all(|&d| d == 9),
            "const true → Socialite fires once (1 dmg each); got {defs:?}"
        );
    } else {
        assert!(
            defs.iter().all(|&d| d == 8),
            "const false → Socialite fires per amulet (2 dmg each); got {defs:?}"
        );
    }
}

#[test]
fn fuse_amulet_dandy_trigger_count_matches_const() {
    let db = load_db();
    let mut st = started(&db, 77_002);
    clear_hand(&mut st, PlayerId::A);
    put_field(&db, &mut st, PlayerId::A, DANDY);
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(PlayerId::A).leader_defense = 10;
    let enemies0 = st.player(PlayerId::B).field_count();
    fuse_two_amulets(&db, &mut st, host);
    let destroyed = enemies0 - st.player(PlayerId::B).field_count();
    let healed = leader_def(&st, PlayerId::A) - 10;
    if FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION {
        assert_eq!(
            destroyed, 1,
            "const true → Dandy destroys once; destroyed={destroyed}"
        );
        assert_eq!(healed, 2, "const true → restore 2 once; healed={healed}");
    } else {
        assert_eq!(
            destroyed, 2,
            "const false → Dandy destroys per amulet; destroyed={destroyed}"
        );
        assert_eq!(
            healed, 4,
            "const false → restore 2 per amulet; healed={healed}"
        );
    }
}
