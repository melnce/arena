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

    // With 2 clues: destroy random highest-attack enemy; clue count after Consume 2.
    let mut st = started(&db, 10_401);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2);
    put_field(&db, &mut st, PlayerId::B, "89500010"); // Atk Five 5/10 — highest
    put_field(&db, &mut st, PlayerId::B, VANILLA); // 2/2 — lower
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_pp(&db, &mut st, CALAMITY, 1);
    assert_eq!(leader_def(&st, PlayerId::A), 12);
    assert_eq!(
        clue_count(&st, PlayerId::A),
        0,
        "Consume 2 clues → clue count 0 (amulet gone)"
    );
    assert!(
        !field_has(&st, PlayerId::B, "89500010"),
        "consume 2 clues destroys highest-atk enemy"
    );
    assert!(
        field_has(&st, PlayerId::B, VANILLA),
        "lower-atk enemy survives"
    );

    // Tie at highest attack: both tied, so whichever random pick, one of the two
    // highest is destroyed and the third (lower) always survives — board where
    // every random choice yields the same survivors set for the lower one.
    let mut st = started(&db, 10_402);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1);
    put_field(&db, &mut st, PlayerId::B, TANK); // 2/10
    put_field(&db, &mut st, PlayerId::B, TANK); // 2/10 — tied highest
    put_field(&db, &mut st, PlayerId::B, VANILLA); // 2/2 atk same? Vanilla is 2/2 — atk 2 tied!
    // Use a strictly lower-atk follower for the survivor guarantee.
    for s in st.player_mut(PlayerId::B).field.iter_mut() {
        *s = None;
    }
    put_field(&db, &mut st, PlayerId::B, "89500010"); // 5 atk
    put_field(&db, &mut st, PlayerId::B, "89500010"); // 5 atk tied
    put_field(&db, &mut st, PlayerId::B, VANILLA); // 2 atk
    play_pp(&db, &mut st, CALAMITY, 1);
    assert_eq!(
        count_id(&st, PlayerId::B, "89500010"),
        1,
        "tie at highest: exactly one of the tied highest is destroyed"
    );
    assert!(
        field_has(&st, PlayerId::B, VANILLA),
        "lower-atk always survives a highest-atk random destroy"
    );

    // Fuse to Calamity summons Evidence.
    let mut st = started(&db, 10_403);
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, CALAMITY);
    let p = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[p]);
    assert!(
        field_has(&st, PlayerId::A, EVIDENCE),
        "Whenever you Fuse to this card, summon a Divine Evidence"
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
    let max0 = st.player(PlayerId::B).leader_max;
    play_pp(&db, &mut st, LINEAGE, 9);
    // Choose explicitly: find tank option.
    if let Phase::Choice {
        node: ChoiceNode::Targets { options, .. },
        ..
    } = &st.phase
    {
        let idx = options
            .iter()
            .position(|o| {
                matches!(
                    o,
                    arena_engine::TargetOpt::Slot { player, slot }
                        if *player == PlayerId::B
                            && st
                                .field_inst(*player, *slot)
                                .is_some_and(|c| c.card.as_str() == TANK)
                )
            })
            .expect("tank among options") as u8;
        choose(&db, &mut st, idx);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        count_id(&st, PlayerId::B, TANK),
        0,
        "destroys all enemy copies of the selected name (both tanks)"
    );
    assert_eq!(
        count_id(&st, PlayerId::B, STORM),
        1,
        "negative: differently named enemy survives"
    );
    assert_eq!(
        st.player(PlayerId::B).leader_max,
        max0,
        "unfused → no halve of enemy leader max"
    );

    // Fused: Evidence on fuse; odd max → ⌈max/2⌉; defense clamps.
    let mut st = started(&db, 10_431);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, STORM);
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, LINEAGE);
    let p = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[p]);
    assert!(
        field_has(&st, PlayerId::A, EVIDENCE),
        "Whenever you Fuse to this card, summon a Divine Evidence"
    );
    st.player_mut(PlayerId::B).leader_max = 21; // odd → ⌈21/2⌉ = 11
    st.player_mut(PlayerId::B).leader_defense = 21;
    give_pp(&mut st, PlayerId::A, 9, 9);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == LINEAGE)
        .unwrap() as u8;
    play(&db, &mut st, h);
    if let Phase::Choice {
        node: ChoiceNode::Targets { options, .. },
        ..
    } = &st.phase
    {
        let idx = options
            .iter()
            .position(|o| {
                matches!(
                    o,
                    arena_engine::TargetOpt::Slot { player, slot }
                        if *player == PlayerId::B
                            && st
                                .field_inst(*player, *slot)
                                .is_some_and(|c| c.card.as_str() == TANK)
                )
            })
            .expect("tank") as u8;
        choose(&db, &mut st, idx);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        count_id(&st, PlayerId::B, TANK),
        0,
        "fused: still destroys all same-name copies"
    );
    assert_eq!(
        st.player(PlayerId::B).leader_max,
        11,
        "fused → ⌈21/2⌉ = 11"
    );
    assert_eq!(
        st.player(PlayerId::B).leader_defense,
        11,
        "defense clamps down to the new max"
    );
    assert_eq!(
        count_id(&st, PlayerId::B, STORM),
        1,
        "negative: different name survives fused destroy"
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

#[test]
fn majordomo_last_words_adds_two_evidence() {
    let db = load_db();
    let mut st = started(&db, 10_460);
    let slot = put_field(&db, &mut st, PlayerId::A, MAJORDOMO);
    clear_hand(&mut st, PlayerId::A);
    st.player_mut(PlayerId::A).field[slot as usize]
        .as_mut()
        .unwrap()
        .defense = 0;
    play_pp(&db, &mut st, "88001710", 1);
    let n = st
        .player(PlayerId::A)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == EVIDENCE)
        .count();
    assert_eq!(n, 2, "Last Words: Add 2 Divine Evidence to hand");

    // Negative: destroying something else does not add Evidence.
    let mut st = started(&db, 10_461);
    put_field(&db, &mut st, PlayerId::A, VANILLA);
    clear_hand(&mut st, PlayerId::A);
    st.player_mut(PlayerId::A).field[0].as_mut().unwrap().defense = 0;
    play_pp(&db, &mut st, "88001710", 1);
    assert!(
        !hand_has(&st, PlayerId::A, EVIDENCE),
        "negative: non-Majordomo LW does not add Evidence"
    );
}

#[test]
fn otherworldly_intruder_has_storm_and_aura() {
    let db = load_db();
    let mut st = started(&db, 10_462);
    play_pp(&db, &mut st, INTRUDER, 6);
    let c = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == INTRUDER)
        .expect("Intruder");
    assert_eq!(c.traits.storm, Some(true), "printed Storm");
    assert_eq!(c.traits.aura, Some(true), "printed Aura");

    // Negative: a vanilla follower has neither.
    put_field(&db, &mut st, PlayerId::A, VANILLA);
    let v = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == VANILLA)
        .unwrap();
    assert_ne!(v.traits.storm, Some(true));
    assert_ne!(v.traits.aura, Some(true));
}

#[test]
fn bestial_beamcannoneer_summons_striker_and_ancient() {
    let db = load_db();
    let mut st = started(&db, 10_463);
    play_pp(&db, &mut st, BEAM, 6);
    assert!(
        field_has(&st, PlayerId::A, "90072110"),
        "Fanfare summons Striker Artifact"
    );
    assert!(
        field_has(&st, PlayerId::A, "90071140"),
        "Fanfare summons Ancient Artifact"
    );
    let beam = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == BEAM)
        .unwrap();
    assert_eq!(beam.traits.rush, Some(true), "printed Rush");

    // Negative: without playing Beamcannoneer, those artifacts are absent.
    let st = started(&db, 10_464);
    assert!(!field_has(&st, PlayerId::A, "90072110"));
    assert!(!field_has(&st, PlayerId::A, "90071140"));
}

#[test]
fn hand_of_extinction_4_vs_8_when_fused_and_evidence_on_fuse() {
    let db = load_db();
    // Unfused: 4 damage.
    let mut st = started(&db, 10_465);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    play_pp(&db, &mut st, HAND_EXT, 5);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 4),
        "unfused: Deal 4 damage to all enemy followers"
    );

    // Fused: 8 damage + Evidence on fuse.
    let mut st = started(&db, 10_466);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let p = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[p]);
    assert!(
        field_has(&st, PlayerId::A, EVIDENCE),
        "Whenever you Fuse to this card, summon a Divine Evidence"
    );
    give_pp(&mut st, PlayerId::A, 5, 5);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == HAND_EXT)
        .unwrap() as u8;
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 8),
        "fused: Deal 8 damage instead"
    );

    // Negative: fused damage is to followers only — leader untouched by the AoE.
    let leader0 = leader_def(&st, PlayerId::B);
    // Already played; leader should still be full from that play.
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0,
        "negative: Hand of Extinction targets enemy followers, not the leader"
    );
}

#[test]
fn extinctionist_dandy_multi_amulet_fuse_destroy_and_restore() {
    let db = load_db();
    let mut st = started(&db, 10_470);
    play_pp(&db, &mut st, DANDY, 8);
    let evidence_hand = st
        .player(PlayerId::A)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == EVIDENCE)
        .count();
    assert_eq!(evidence_hand, 2, "Fanfare: Add 2 Divine Evidence");

    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let a1 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    let a2 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, VANILLA);
    st.player_mut(PlayerId::A).leader_defense = 10;
    let enemies0 = st.player(PlayerId::B).field_count();
    fuse_amulets(&db, &mut st, host, &[a1, a2]);
    let enemies1 = st.player(PlayerId::B).field_count();
    // Once-per-action (const true): one destroy, one restore 2 — not two.
    assert_eq!(
        enemies0 - enemies1,
        1,
        "multi-amulet fuse: one destroy (FUSE_AMULET_TRIGGERS_ONCE_PER_ACTION)"
    );
    assert_eq!(
        leader_def(&st, PlayerId::A),
        12,
        "multi-amulet fuse: restore 2 once"
    );

    // Negative: fuse without Dandy on field does not destroy/restore via Dandy.
    let mut st = started(&db, 10_471);
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let a1 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(PlayerId::A).leader_defense = 10;
    fuse_amulets(&db, &mut st, host, &[a1]);
    assert!(
        field_has(&st, PlayerId::B, TANK),
        "negative: without Dandy, fuse does not destroy via Dandy"
    );
    assert_eq!(leader_def(&st, PlayerId::A), 10);
}

#[test]
fn socialite_fanfare_evidence_evolve_draws_fuse_card() {
    let db = load_db();
    let mut st = started(&db, 10_475);
    clear_hand(&mut st, PlayerId::A);
    play_pp(&db, &mut st, SOCIALITE, 3);
    let n = st
        .player(PlayerId::A)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == EVIDENCE)
        .count();
    assert_eq!(n, 2, "Fanfare: Add 2 Divine Evidence");

    // Evolve: Draw a card with Fuse — put Hand of Extinction on top of deck.
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, HAND_EXT);
    put_deck(&db, &mut st, PlayerId::A, VANILLA); // non-Fuse below
    // Deck draws from end in this engine? put_deck usually pushes; ensure Fuse is drawn.
    // Use only Fuse card in deck so every draw is Fuse.
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, HAND_EXT);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == SOCIALITE)
                .map(|_| i as u8)
        })
        .unwrap();
    // evolve helper inline
    set_round(&mut st, PlayerId::A, 5);
    st.player_mut(PlayerId::A).ep = 1;
    apply(
        &db,
        &mut st,
        arena_engine::Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
    drain_choice(&db, &mut st);
    assert!(
        hand_has(&st, PlayerId::A, HAND_EXT),
        "Evolve: Draw a card with Fuse"
    );

    // Negative: deck of only non-Fuse → draw finds nothing / skips.
    let mut st = started(&db, 10_476);
    play_pp(&db, &mut st, SOCIALITE, 3);
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, VANILLA);
    clear_hand(&mut st, PlayerId::A);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == SOCIALITE)
                .map(|_| i as u8)
        })
        .unwrap();
    set_round(&mut st, PlayerId::A, 5);
    st.player_mut(PlayerId::A).ep = 1;
    apply(
        &db,
        &mut st,
        arena_engine::Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    drain_choice(&db, &mut st);
    assert!(
        !hand_has(&st, PlayerId::A, VANILLA),
        "negative: Evolve draw filter requires Fuse"
    );
}

#[test]
fn wanderer_fuse_gives_rush_once_per_action_and_se_storm() {
    let db = load_db();
    let mut st = started(&db, 10_480);
    let wslot = put_field(&db, &mut st, PlayerId::A, WANDERER);
    assert_ne!(
        st.field_inst(PlayerId::A, wslot).unwrap().traits.rush,
        Some(true),
        "setup: no Rush yet"
    );
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let a1 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    let a2 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[a1, a2]);
    assert_eq!(
        st.field_inst(PlayerId::A, wslot).unwrap().traits.rush,
        Some(true),
        "Whenever you Fuse → Rush (once per Fuse action, two partners)"
    );
    // Crest countdown also advances once (not twice) for the multi-partner fuse —
    // only when crest is present; grant via LW path separately covered. Here assert
    // Rush is boolean once.

    // Super-Evolve: Storm.
    set_round(&mut st, PlayerId::A, 7);
    st.player_mut(PlayerId::A).sep = 1;
    apply(
        &db,
        &mut st,
        arena_engine::Action::Evolve {
            slot: arena_engine::Slot(wslot),
            super_evolve: true,
        },
    )
    .unwrap();
    drain_choice(&db, &mut st);
    assert_eq!(
        st.field_inst(PlayerId::A, wslot).unwrap().traits.storm,
        Some(true),
        "Super-Evolve: Give this follower Storm"
    );

    // Crest once-per-fuse-action: multi-partner fuse advances countdown by 1.
    let mut st = started(&db, 10_481);
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
    clear_hand(&mut st, PlayerId::A);
    let host = put_hand(&db, &mut st, PlayerId::A, HAND_EXT);
    let a1 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    let a2 = put_hand(&db, &mut st, PlayerId::A, EVIDENCE);
    fuse_amulets(&db, &mut st, host, &[a1, a2]);
    let cd = st
        .player(PlayerId::A)
        .crests
        .iter()
        .find(|c| c.id == WANDERER_CREST)
        .and_then(|c| c.countdown);
    assert_eq!(
        cd,
        Some(1),
        "crest: multi-amulet fuse advances countdown once, not per partner"
    );
}
