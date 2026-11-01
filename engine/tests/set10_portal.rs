//! Set 10 Portalcraft — Calamity, Socialite/Dandy fuse-amulet, Wanderer crest,
//! Lineage, Incohesive Construction, Fuse: Amulets partners.

use arena_engine::{apply, Action, ChoiceNode, Phase, PlayerId};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const MAJORDOMO: &str = "11071110";
const INTRUDER: &str = "11071120";
const INCOHESIVE: &str = "11071310";
const SOCIALITE: &str = "11072110";
const BEAM: &str = "11072120";
const HAND_EXT: &str = "11072310";
const DANDY: &str = "11073110";
const CALAMITY: &str = "11073310";
const LINEAGE: &str = "11074110";
const WANDERER: &str = "11074120";
const WANDERER_CREST: &str = "crest:11074120";
const PUPPET: &str = "90071120";
const TANK: &str = "89500001";
const VANILLA: &str = "88001110";
const STORM: &str = "88001120";

const SET10_PORTAL: &[&str] = &[
    MAJORDOMO, INTRUDER, INCOHESIVE, SOCIALITE, BEAM, HAND_EXT, DANDY, CALAMITY, LINEAGE, WANDERER,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    give_pp(st, st.active, pp, pp.max(10));
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn fuse_amulets(
    db: &arena_engine::CardDb,
    st: &mut arena_engine::State,
    host: u8,
    partners: &[u8],
) {
    apply(db, st, Action::Fuse { host }).expect("fuse start");
    for &p in partners {
        // Choose by current FusePartners option index matching hand pos.
        let Phase::Choice {
            node: ChoiceNode::FusePartners { options, .. },
            ..
        } = &st.phase
        else {
            panic!("expected FusePartners, got {:?}", st.phase);
        };
        let idx = options
            .iter()
            .position(|&o| o == p)
            .unwrap_or_else(|| panic!("partner {p} not in options {options:?}"))
            as u8;
        choose(db, st, idx);
    }
    confirm(db, st);
    drain_choice(db, st);
}

fn clue_count(st: &arena_engine::State, who: PlayerId) -> i32 {
    st.player(who)
        .field
        .iter()
        .flatten()
        .find(|c| c.is_clue())
        .map(|c| c.clue_count)
        .unwrap_or(0)
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
fn set10_portal_require_supported() {
    let db = load_db();
    for id in SET10_PORTAL {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.crest(WANDERER_CREST)
        .unwrap_or_else(|e| panic!("{WANDERER_CREST}: {e}"));
}

#[test]
fn calamity_in_custody_one_clue_heals_without_consume_two_destroys_highest() {
    let db = load_db();
    // Q&A: 1 clue → heal still happens; nothing consumed/destroyed.
    let mut st = started(&db, 10_400);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    assert_eq!(clue_count(&st, PlayerId::A), 1);
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_pp(&db, &mut st, CALAMITY, 1);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        12,
        "Q&A: restore 2 still happens with 1 clue"
    );
    assert_eq!(
        clue_count(&st, PlayerId::A),
        1,
        "Q&A: 1 clue is not consumed"
    );
    assert!(
        field_has(&st, PlayerId::B, TANK),
        "negative: destroy branch must not fire with <2 clues"
    );

    // With 2 clues: destroy random highest-attack enemy (ties random).
    let mut st = started(&db, 10_401);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    // Merge second Evidence → count 2.
    play_pp(&db, &mut st, EVIDENCE, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2);
    put_field(&db, &mut st, PlayerId::B, "89500010"); // Atk Five 5/10 — highest
    put_field(&db, &mut st, PlayerId::B, VANILLA); // 2/2 — lower
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_pp(&db, &mut st, CALAMITY, 1);
    assert_eq!(leader_def(&st, PlayerId::A), 12);
    assert!(
        !field_has(&st, PlayerId::B, "89500010"),
        "consume 2 clues destroys highest-atk enemy"
    );
    assert!(
        field_has(&st, PlayerId::B, VANILLA),
        "lower-atk enemy survives"
    );
}

#[test]
fn socialite_fuse_two_amulets_fires_ally_fuse_amulet_once() {
    let db = load_db();
    let mut st = started(&db, 10_410);
    // Socialite on field; two enemy tanks to observe AoE once vs twice.
    play_pp(&db, &mut st, SOCIALITE, 3);
    // Fanfare added Evidence to hand — clear and set up fuse host + 2 amulets.
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let a1 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    let a2 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let defs0: Vec<i32> = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .map(|c| c.defense)
        .collect();
    fuse_amulets(&db, &mut st, host, &[a1, a2]);
    let defs1: Vec<i32> = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == TANK)
        .map(|c| c.defense)
        .collect();
    // Once per action: each tank takes 1, not 2.
    assert_eq!(
        defs1.len(),
        2,
        "both tanks survive 1 damage from once-per-action"
    );
    assert!(
        defs1.iter().all(|&d| d == defs0[0] - 1),
        "FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION: 1 damage each, not per amulet; got {defs1:?}"
    );
    // Host was fused.
    assert!(
        st.player(PlayerId::A)
            .hand
            .iter()
            .any(|c| c.card.as_str() == HAND_EXT && c.flags.was_fused),
        "host marked was_fused"
    );
}

#[test]
fn wanderer_crest_fuse_advances_lw_summons_regain_bounces() {
    let db = load_db();
    let mut st = started(&db, 10_421);
    let wslot = put_field(&db, &mut st, PlayerId::A, WANDERER);
    st.player_mut(PlayerId::A).field[wslot as usize]
        .as_mut()
        .unwrap()
        .defense = 0;
    // Destroy via random highest-atk destroy so Last Words gains the crest.
    play_pp(&db, &mut st, "88001710", 1);
    assert!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .any(|c| c.id == WANDERER_CREST),
        "LW gains Crest: Dimensional Wanderer"
    );
    let cd = st
        .player(PlayerId::A)
        .crests
        .iter()
        .find(|c| c.id == WANDERER_CREST)
        .and_then(|c| c.countdown);
    assert_eq!(cd, Some(2));

    // Fuse advances countdown by 1; at 0 destroys and LW summons Wanderer.
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let p = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[p]);
    let cd = st
        .player(PlayerId::A)
        .crests
        .iter()
        .find(|c| c.id == WANDERER_CREST)
        .and_then(|c| c.countdown);
    assert_eq!(cd, Some(1), "fuse advances crest countdown");

    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, CALAMITY);
    let p = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    // Ensure field has room for LW summon.
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        *s = None;
    }
    fuse_amulets(&db, &mut st, host, &[p]);
    assert!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .all(|c| c.id != WANDERER_CREST),
        "countdown 0 destroys crest"
    );
    assert!(
        field_has(&st, PlayerId::A, WANDERER),
        "crest LW summons Dimensional Wanderer"
    );

    // Re-gaining while active bounces (no second crest).
    // Re-gain while crest already active bounces (no second crest).
    let mut st = started(&db, 10_422);
    let order = st.crest_order;
    st.crest_order += 1;
    st.player_mut(PlayerId::A)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: WANDERER_CREST.into(),
            countdown: Some(2),
            faith: false,
            once_used: vec![],
            granted_order: order,
            granted: vec![],
            choose_used: Default::default(),
            sequence_index: 0,
        });
    let wslot = put_field(&db, &mut st, PlayerId::A, WANDERER);
    st.player_mut(PlayerId::A).field[wslot as usize]
        .as_mut()
        .unwrap()
        .defense = 0;
    play_pp(&db, &mut st, "88001710", 1);
    assert_eq!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .filter(|c| c.id == WANDERER_CREST)
            .count(),
        1,
        "negative: re-gain while active bounces — still one crest"
    );
    assert_eq!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .find(|c| c.id == WANDERER_CREST)
            .and_then(|c| c.countdown),
        Some(2),
        "bounce does not reset countdown"
    );
}

#[test]
fn lineage_destroys_all_same_name_fused_halves_max_unfused_does_not() {
    let db = load_db();
    // Enemy: two same-name + one different. Destroy all copies of selected.
    let mut st = started(&db, 10_430);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, STORM);
    clear_hand(&mut st, PlayerId::A);
    // Unfused Lineage: destroy copies, no halve.
    let max0 = st.player(PlayerId::B).leader_max;
    play_pp(&db, &mut st, LINEAGE, 9);
    // Choice: select a tank.
    assert!(
        !field_has(&st, PlayerId::B, TANK),
        "destroys all copies of selected (including selected)"
    );
    assert!(
        field_has(&st, PlayerId::B, STORM),
        "negative: differently named enemy survives"
    );
    assert_eq!(
        st.player(PlayerId::B).leader_max,
        max0,
        "unfused → no halve of enemy leader max"
    );

    // Fused → also halve enemy leader max (ceil).
    let mut st = started(&db, 10_431);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, STORM);
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, LINEAGE);
    let p = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[p]);
    st.player_mut(PlayerId::B).leader_max = 20;
    st.player_mut(PlayerId::B).leader_defense = 20;
    give_pp(&mut st, PlayerId::A, 9, 9);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == LINEAGE)
        .unwrap() as u8;
    play(&db, &mut st, h);
    // Choose tank among options.
    if matches!(st.phase, Phase::Choice { .. }) {
        // Prefer tank if options expose both; choose 0 is fine if tanks first.
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        st.player(PlayerId::B).leader_max,
        10,
        "fused → halve enemy leader max (⌈20/2⌉=10)"
    );
    assert!(
        field_has(&st, PlayerId::B, STORM)
            || count_id(&st, PlayerId::B, STORM) + count_id(&st, PlayerId::B, TANK) <= 1,
        "different name can survive depending on selection"
    );
}

#[test]
fn incohesive_construction_enhanced_puppet_bane_xor_ward() {
    let db = load_db();
    let mut saw_bane = false;
    let mut saw_ward = false;
    for seed in 10_440..10_460 {
        let mut st = started(&db, seed);
        clear_hand(&mut st, PlayerId::A);
        play_pp(&db, &mut st, INCOHESIVE, 1);
        let p = st
            .player(PlayerId::A)
            .hand
            .iter()
            .find(|c| c.card.as_str() == PUPPET)
            .expect("Enhanced Puppet added");
        let bane = p.traits.bane == Some(true);
        let ward = p.traits.ward == Some(true);
        assert!(
            bane ^ ward,
            "Enhanced Puppet gets Bane XOR Ward; got bane={bane} ward={ward}"
        );
        saw_bane |= bane;
        saw_ward |= ward;
        if saw_bane && saw_ward {
            break;
        }
    }
    assert!(saw_bane && saw_ward, "random covers both keyword arms");
}

#[test]
fn fuse_amulets_partners_only_accept_amulets() {
    let db = load_db();
    let mut st = started(&db, 10_450);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_hand(&db, &mut st, PlayerId::A, VANILLA); // follower — not a legal partner
    apply(&db, &mut st, Action::Fuse { host: 0 }).expect("fuse");
    let Phase::Choice {
        node: ChoiceNode::FusePartners { options, .. },
        ..
    } = &st.phase
    else {
        panic!("expected FusePartners");
    };
    assert!(
        options.iter().all(|&i| {
            st.player(PlayerId::A).hand[i as usize].kind == arena_engine::CardKind::Amulet
        }),
        "Fuse: Amulets partners are amulets only; options={options:?}"
    );
    assert!(
        !options.contains(&2),
        "negative: follower must not be a Fuse: Amulets partner"
    );
}
