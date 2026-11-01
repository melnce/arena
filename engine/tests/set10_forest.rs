//! Set 10 Forestcraft — Inquiring Mischief package, Untamed Testimony crest,
//! Nosche / Wolf Pack / Heartening Hyphae. Assert against card text and Q&As.

use arena_engine::{apply, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const MISCHIEF: &str = "11012110";
const DOGSITTER: &str = "11011110";
const UNTAMED: &str = "11012310";
const UNTAMED_CREST: &str = "crest:11012310";
const REINCARNATION: &str = "11011310";
const GARDEN_ELF: &str = "11012120";
const INTERROGATION: &str = "11013310";
const BEST_FRIEND: &str = "11013110";
const NOSCHE: &str = "11014110";
const LACKEY: &str = "11011120";
const HYPHAE: &str = "11014120";
const FAIRY: &str = "90011110";
const TANK: &str = "89500001";
const VANILLA: &str = "88001110";

const SET10_FOREST: &[&str] = &[
    MISCHIEF,
    DOGSITTER,
    UNTAMED,
    REINCARNATION,
    GARDEN_ELF,
    INTERROGATION,
    BEST_FRIEND,
    NOSCHE,
    LACKEY,
    HYPHAE,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    give_pp(st, st.active, pp, pp.max(10));
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn evolve_slot(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, supered: bool) {
    let me = st.active;
    if supered {
        if st.player(me).turns_taken < 7 {
            set_round(st, me, 7);
        }
        st.player_mut(me).sep = st.player(me).sep.max(1);
    } else {
        if st.player(me).turns_taken < 5 {
            set_round(st, me, 5);
        }
        st.player_mut(me).ep = st.player(me).ep.max(1);
    }
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: supered,
        },
    )
    .expect("evolve");
    drain_choice(db, st);
}

fn field_mischiefs(st: &arena_engine::State, who: PlayerId) -> Vec<&arena_engine::CardInstance> {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == MISCHIEF)
        .collect()
}

fn count_id(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == id)
        .count()
}

#[test]
fn set10_forest_require_supported() {
    let db = load_db();
    for id in SET10_FOREST {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.crest(UNTAMED_CREST)
        .unwrap_or_else(|e| panic!("{UNTAMED_CREST}: {e}"));
}

#[test]
fn mischief_enter_buffs_with_two_other_one_cost_allies_including_amulets() {
    let db = load_db();
    // ≥2 other 1-base-cost allies (follower + amulet) → +2/+1 (3/2).
    let mut st = started(&db, 10_100);
    put_field(&db, &mut st, PlayerId::A, FAIRY); // 1-cost follower
    put_field(&db, &mut st, PlayerId::A, EVIDENCE); // 1-cost Clue amulet
    play_pp(&db, &mut st, MISCHIEF, 1);
    let m = field_mischiefs(&st, PlayerId::A);
    assert_eq!(m.len(), 1);
    assert_eq!(
        (m[0].attack, m[0].defense),
        (3, 2),
        "printed enter: +2/+1 with ≥2 other 1-cost allies"
    );

    // Negative: only 1 other 1-cost ally → no buff (stays 1/1). Self not "other".
    let mut st = started(&db, 10_101);
    put_field(&db, &mut st, PlayerId::A, FAIRY);
    play_pp(&db, &mut st, MISCHIEF, 1);
    let m = field_mischiefs(&st, PlayerId::A);
    assert_eq!(m.len(), 1);
    assert_eq!(
        (m[0].attack, m[0].defense),
        (1, 1),
        "negative: one other 1-cost ally is not enough; self excluded"
    );
}

#[test]
fn untamed_testimony_crest_summons_mischief_two_turns_not_on_enemy_turn() {
    let db = load_db();
    let mut st = started(&db, 10_110);
    play_pp(&db, &mut st, UNTAMED, 2);
    assert!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .any(|c| c.id == UNTAMED_CREST),
        "spell gains Crest: Untamed Testimony"
    );
    assert!(field_has(&st, PlayerId::A, EVIDENCE));
    let cd0 = st
        .player(PlayerId::A)
        .crests
        .iter()
        .find(|c| c.id == UNTAMED_CREST)
        .and_then(|c| c.countdown);
    assert_eq!(cd0, Some(2), "Countdown (2)");

    // Negative: enemy turn start does not summon.
    end_turn(&db, &mut st); // → B's turn
    assert_eq!(
        count_id(&st, PlayerId::A, MISCHIEF),
        0,
        "negative: enemy turn start must not summon Mischief"
    );

    // First own turn start: summon #1; countdown advances.
    end_turn(&db, &mut st); // → A's turn
    assert_eq!(
        count_id(&st, PlayerId::A, MISCHIEF),
        1,
        "start of your turn summons Mischief"
    );

    // Second own turn start: summon #2 even as countdown hits 0 (queued same step).
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(
        count_id(&st, PlayerId::A, MISCHIEF),
        2,
        "second start-of-turn summon; losing crest does not cancel queued trigger"
    );
    assert!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .all(|c| c.id != UNTAMED_CREST),
        "crest gone after Countdown reaches 0"
    );
}

#[test]
fn nosches_interrogation_storm_gated_on_two_one_cost_allies() {
    let db = load_db();
    // Lone enemy tank: every random pick deals the printed 5 to it.
    let mut st = started(&db, 10_120);
    put_field(&db, &mut st, PlayerId::A, FAIRY);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    play_pp(&db, &mut st, INTERROGATION, 3);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 5),
        "printed: Deal 5 damage to a random enemy follower"
    );
    let hand_m = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == MISCHIEF)
        .expect("Mischief added to hand");
    assert_eq!(
        hand_m.traits.storm,
        Some(true),
        "≥2 one-cost allies → Storm on added Mischief"
    );

    // Negative: without two one-cost allies, no Storm; damage still 5.
    let mut st = started(&db, 10_121);
    put_field(&db, &mut st, PlayerId::A, FAIRY);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    play_pp(&db, &mut st, INTERROGATION, 3);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 5),
        "damage is still exactly 5 without the Storm branch"
    );
    let hand_m = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == MISCHIEF)
        .expect("Mischief added");
    assert_ne!(
        hand_m.traits.storm,
        Some(true),
        "negative: fewer than two 1-cost allies → no Storm"
    );
}

#[test]
fn wolf_pack_lackey_fanfare_copies_no_nested_summon_evolve_replicates() {
    let db = load_db();
    let mut st = started(&db, 10_130);
    play_pp(&db, &mut st, LACKEY, 7);
    assert_eq!(
        count_id(&st, PlayerId::A, LACKEY),
        3,
        "Fanfare summons 2 copies (self + 2)"
    );

    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == LACKEY && !c.evolved)
                .map(|_| i as u8)
        })
        .expect("unevolved Lackey");
    // Field has 3; Evolve adds 2 → needs 2 free → currently 2 free.
    evolve_slot(&db, &mut st, slot, false);
    assert_eq!(
        count_id(&st, PlayerId::A, LACKEY),
        5,
        "Evolve replicates Fanfare → two more copies"
    );

    // Negative: with only one free slot, Fanfare summons one copy (not two) —
    // summoned copies are still not played, so no nested Fanfare fills the rest.
    let mut st = started(&db, 10_131);
    for _ in 0..3 {
        put_field(&db, &mut st, PlayerId::A, VANILLA);
    }
    play_pp(&db, &mut st, LACKEY, 7);
    assert_eq!(
        count_id(&st, PlayerId::A, LACKEY),
        2,
        "negative: one free slot → only one summoned copy; no nested Fanfare"
    );
    assert_eq!(
        st.player(PlayerId::A).field_count(),
        5,
        "negative: field stays full; excess summon skipped"
    );
}

#[test]
fn heartening_hyphae_full_field_second_draw_stays_hand_se_full_hand_burns() {
    let db = load_db();
    // Q&A: 3 Fairies + Hyphae → first forest draw summons, second stays in hand.
    let mut st = started(&db, 10_140);
    st.player_mut(PlayerId::A).deck.clear();
    for _ in 0..4 {
        put_deck(&db, &mut st, PlayerId::A, FAIRY);
    }
    for _ in 0..3 {
        put_field(&db, &mut st, PlayerId::A, FAIRY);
    }
    play_pp(&db, &mut st, HYPHAE, 8);
    // Hyphae + 3 Fairies + 1 summoned = 5; one Fairy remains in hand.
    assert_eq!(
        st.player(PlayerId::A).field_count(),
        5,
        "field full after first summon"
    );
    assert!(
        hand_has(&st, PlayerId::A, FAIRY),
        "Q&A: second Forestcraft draw stays in hand on full field"
    );
    assert_eq!(
        count_id(&st, PlayerId::A, FAIRY),
        4,
        "exactly one of the two draws was summoned"
    );

    // Super-Evolve with full hand: draws burn → no summon.
    let mut st = started(&db, 10_141);
    st.player_mut(PlayerId::A).deck.clear();
    for _ in 0..4 {
        put_deck(&db, &mut st, PlayerId::A, FAIRY);
    }
    play_pp(&db, &mut st, HYPHAE, 8);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == HYPHAE)
                .map(|_| i as u8)
        })
        .expect("Hyphae");
    clear_hand(&mut st, PlayerId::A);
    for _ in 0..9 {
        put_hand(&db, &mut st, PlayerId::A, VANILLA);
    }
    let field_before = st.player(PlayerId::A).field_count();
    let shadows0 = st.player(PlayerId::A).shadows;
    evolve_slot(&db, &mut st, slot, true);
    assert_eq!(
        st.player(PlayerId::A).field_count(),
        field_before,
        "Q&A: full hand burns SE draws — no Forestcraft summoned"
    );
    assert!(
        st.player(PlayerId::A).shadows > shadows0,
        "overflow draws convert to shadows"
    );

    // Negative: non-forestcraft draws stay in hand unsunmmoned.
    let mut st = started(&db, 10_142);
    st.player_mut(PlayerId::A).deck.clear();
    for _ in 0..4 {
        put_deck(&db, &mut st, PlayerId::A, VANILLA);
    }
    play_pp(&db, &mut st, HYPHAE, 8);
    assert!(
        !field_has(&st, PlayerId::A, VANILLA),
        "negative: non-forestcraft draw must not be summoned"
    );
    assert!(
        st.player(PlayerId::A)
            .hand
            .iter()
            .filter(|c| c.card.as_str() == VANILLA)
            .count()
            >= 2,
        "negative: both non-forest draws remain in hand"
    );
}

#[test]
fn debonair_dogsitter_evolve_restores_with_two_one_cost_allies() {
    let db = load_db();
    let mut st = started(&db, 10_150);
    play_pp(&db, &mut st, DOGSITTER, 2);
    assert!(field_has(&st, PlayerId::A, EVIDENCE), "Fanfare summons Evidence");
    // Evidence (1) + Mischief (1) → two 1-cost allies; Dogsitter itself is 2-cost.
    put_field(&db, &mut st, PlayerId::A, MISCHIEF);
    st.player_mut(PlayerId::A).leader_defense = 10;
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == DOGSITTER)
                .map(|_| i as u8)
        })
        .expect("Dogsitter");
    evolve_slot(&db, &mut st, slot, false);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        12,
        "Evolve with ≥2 one-cost allies restores 2"
    );

    // Negative: fewer than two 1-cost allies → no restore.
    let mut st = started(&db, 10_151);
    play_pp(&db, &mut st, DOGSITTER, 2);
    // Only Evidence (1); no second 1-cost.
    st.player_mut(PlayerId::A).leader_defense = 10;
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == DOGSITTER)
                .map(|_| i as u8)
        })
        .expect("Dogsitter");
    evolve_slot(&db, &mut st, slot, false);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        10,
        "negative: fewer than two 1-cost allies → no restore"
    );
}

#[test]
fn natural_reincarnation_enhance_6_summons_3_and_evolves_all_3() {
    let db = load_db();
    // Base: summon 1 Fairy and evolve it.
    let mut st = started(&db, 10_160);
    play_pp(&db, &mut st, REINCARNATION, 3);
    assert_eq!(count_id(&st, PlayerId::A, FAIRY), 1);
    let f = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == FAIRY)
        .expect("Fairy");
    assert!(f.evolved, "base: the summoned Fairy is evolved");

    // Enhance (6): Summon 3 instead — and evolve all 3 (bound evolve on the group).
    let mut st = started(&db, 10_161);
    play_pp(&db, &mut st, REINCARNATION, 6);
    assert_eq!(
        count_id(&st, PlayerId::A, FAIRY),
        3,
        "Enhance (6) summons 3 Fairies"
    );
    let evolved = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FAIRY && c.evolved)
        .count();
    assert_eq!(
        evolved, 3,
        "Enhance (6) evolves all 3 summoned Fairies"
    );

    // Negative: paying only 3 does not summon three.
    let mut st = started(&db, 10_162);
    play_pp(&db, &mut st, REINCARNATION, 3);
    assert_eq!(
        count_id(&st, PlayerId::A, FAIRY),
        1,
        "negative: without Enhance cost, only one Fairy"
    );
}

#[test]
fn garden_elf_end_of_own_turn_adds_bounty_and_fairy() {
    let db = load_db();
    let mut st = started(&db, 10_170);
    play_pp(&db, &mut st, GARDEN_ELF, 3);
    clear_hand(&mut st, PlayerId::A);
    end_turn(&db, &mut st); // A's end → adds
    assert!(
        hand_has(&st, PlayerId::A, "90011310"),
        "end of your turn adds Deepwood Bounty"
    );
    assert!(
        hand_has(&st, PlayerId::A, FAIRY),
        "end of your turn adds Fairy"
    );

    // Negative: enemy end of turn does not add.
    clear_hand(&mut st, PlayerId::A);
    end_turn(&db, &mut st); // B's end
    assert!(
        !hand_has(&st, PlayerId::A, "90011310") && !hand_has(&st, PlayerId::A, FAIRY),
        "negative: opponent's end of turn must not add"
    );
}

#[test]
fn sleuths_best_friend_evolve_summons_mischief_with_two_one_cost() {
    let db = load_db();
    let mut st = started(&db, 10_180);
    play_pp(&db, &mut st, BEST_FRIEND, 4);
    assert!(field_has(&st, PlayerId::A, EVIDENCE), "Fanfare Evidence");
    put_field(&db, &mut st, PlayerId::A, FAIRY); // Evidence + Fairy = two 1-cost
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == BEST_FRIEND)
                .map(|_| i as u8)
        })
        .expect("Best Friend");
    evolve_slot(&db, &mut st, slot, false);
    assert_eq!(
        count_id(&st, PlayerId::A, MISCHIEF),
        1,
        "Evolve with ≥2 one-cost allies summons Mischief"
    );

    // Negative: only Evidence (one 1-cost) → no Mischief.
    let mut st = started(&db, 10_181);
    play_pp(&db, &mut st, BEST_FRIEND, 4);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == BEST_FRIEND)
                .map(|_| i as u8)
        })
        .expect("Best Friend");
    evolve_slot(&db, &mut st, slot, false);
    assert_eq!(
        count_id(&st, PlayerId::A, MISCHIEF),
        0,
        "negative: fewer than two 1-cost allies → no Mischief"
    );
}

#[test]
fn nosche_fanfare_summons_and_super_evolve_plus_2_plus_0_other_allies() {
    let db = load_db();
    // Fanfare summons.
    let mut st = started(&db, 10_190);
    play_pp(&db, &mut st, NOSCHE, 6);
    assert!(
        field_has(&st, PlayerId::A, BEST_FRIEND),
        "Fanfare summons Sleuth's Best Friend"
    );
    assert!(
        field_has(&st, PlayerId::A, MISCHIEF),
        "Fanfare summons Inquiring Mischief"
    );

    // SE +2/+0 on a clean board: only Nosche + one other ally (clear Fanfare summons).
    let mut st = started(&db, 10_191);
    put_field(&db, &mut st, PlayerId::A, VANILLA);
    let vanilla_atk0 = field_atk(&st, PlayerId::A, VANILLA).unwrap();
    let vanilla_def0 = field_def(&st, PlayerId::A, VANILLA).unwrap();
    let nslot = put_field(&db, &mut st, PlayerId::A, NOSCHE);
    let nosche_atk0 = field_atk(&st, PlayerId::A, NOSCHE).unwrap();
    let nosche_def0 = field_def(&st, PlayerId::A, NOSCHE).unwrap();
    evolve_slot(&db, &mut st, nslot, true);
    assert_eq!(
        field_atk(&st, PlayerId::A, VANILLA),
        Some(vanilla_atk0 + 2),
        "SE: +2/+0 to other allied followers"
    );
    assert_eq!(
        field_def(&st, PlayerId::A, VANILLA),
        Some(vanilla_def0),
        "+2/+0 leaves other allies' defense unchanged"
    );
    // Negative: Nosche gets only the SE body +3/+3, not the printed +2/+0.
    assert_eq!(
        (
            field_atk(&st, PlayerId::A, NOSCHE),
            field_def(&st, PlayerId::A, NOSCHE)
        ),
        (Some(nosche_atk0 + 3), Some(nosche_def0 + 3)),
        "negative: other:true excludes Nosche from the +2/+0 buff"
    );
}
