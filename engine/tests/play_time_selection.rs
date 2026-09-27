//! Play-time selections: picks on a played card lock in before play reactions.

use arena_engine::state::CrestInstance;
use arena_engine::{apply, legal_actions, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const MOELLE: &str = "10811130";
const WORLD: &str = "10503210";
const ELMOTT: &str = "10433110";
const SPILLING_RED: &str = "10642310";
const ITSURUGI: &str = "10854110";
const IMARI: &str = "10574120";
const SINCERITY: &str = "10573310";
const ARTISAN: &str = "10571120";
const ARA: &str = "10534120";
const YUEL_CREST: &str = "crest:10414120";
const MAJESTIC_CREST: &str = "crest:10622310";
const ENHANCE_PICK: &str = "89209991";
const DRAW_THEN_PICK: &str = "89209990";
const VANILLA: &str = "88001110";
const TANK: &str = "89500001";
const FEARLESS: &str = "10621110";
const SWEET_ABOMINATION: &str = "10733110";
const COST_FIVE_ALLY: &str = "89500011";

fn hand_inst_ids(st: &arena_engine::State, who: PlayerId) -> Vec<u32> {
    st.player(who).hand.iter().map(|c| c.id).collect()
}

fn choice_hand_ids(st: &arena_engine::State) -> Vec<u32> {
    let Phase::Choice { node, .. } = &st.phase else {
        panic!("expected Choice phase");
    };
    match node {
        arena_engine::ChoiceNode::Targets { options, .. } => options
            .iter()
            .filter_map(|t| match t {
                arena_engine::TargetOpt::Hand { player, pos } => {
                    st.player(*player).hand.get(*pos as usize).map(|c| c.id)
                }
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

fn pending_kind(st: &arena_engine::State) -> arena_engine::PendingKind {
    let Phase::Choice { node, .. } = &st.phase else {
        panic!("expected Choice");
    };
    match node {
        arena_engine::ChoiceNode::Targets { pending, .. }
        | arena_engine::ChoiceNode::Modes { pending, .. }
        | arena_engine::ChoiceNode::Cards { pending, .. } => pending.kind,
        _ => panic!("unexpected choice node"),
    }
}

fn add_crest(st: &mut arena_engine::State, who: PlayerId, id: &str, cd: i32) {
    let order = st.crest_order;
    st.crest_order += 1;
    st.player_mut(who).crests.push(CrestInstance {
        id: id.into(),
        countdown: Some(cd),
        faith: false,
        once_used: vec![],
        granted_order: order,
        granted: vec![],
        choose_used: Default::default(),
    });
}

fn wog_countdown(st: &arena_engine::State, who: PlayerId) -> Option<i32> {
    st.player(who)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown)
}

/// Moelle + World of Games at count 1: Last Words draws are not candidates.
#[test]
fn moelle_pick_freezes_hand_before_wog_last_words() {
    let db = load_db();
    let mut st = started(&db, 901);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    let slot = put_field(&db, &mut st, me, WORLD);
    if let Some(w) = st.field_inst_mut(me, slot) {
        w.countdown = Some(1);
    }
    let keeper = put_hand(&db, &mut st, me, VANILLA);
    let _moelle = put_hand(&db, &mut st, me, MOELLE);
    let pre = hand_inst_ids(&st, me);
    play(&db, &mut st, 1);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    assert_eq!(wog_countdown(&st, me), Some(1), "pick before WoG reaction");
    let opts = choice_hand_ids(&st);
    assert_eq!(opts.len(), 1);
    assert!(opts.contains(&pre[keeper as usize]));
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, me, WORLD),
        "WoG Last Words fired after the play-time pick"
    );
    assert_eq!(
        st.player(me).hand.len(),
        pre.len() - 2 + 2 + 1,
        "hand before − Moelle − pick + WoG draws + Moelle draw"
    );
}

/// Evolved Imari + Sincerity: buddies summoned by play reaction are not candidates.
#[test]
fn sincerity_pick_excludes_imari_buddies_summoned_on_play() {
    let db = load_db();
    let mut st = started(&db, 902);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, IMARI);
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.evolved = true;
    }
    put_field(&db, &mut st, me, VANILLA);
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SINCERITY);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    assert!(
        !field_has(&st, me, "90074140"),
        "buddies not on field until after play-time pick"
    );
    let Phase::Choice { node, .. } = &st.phase else {
        unreachable!();
    };
    if let arena_engine::ChoiceNode::Targets { options, .. } = node {
        for t in options {
            if let arena_engine::TargetOpt::Slot { player, slot } = t {
                let id = st
                    .field_inst(*player, *slot)
                    .map(|c| c.card.as_str().to_string())
                    .unwrap_or_default();
                assert_ne!(id, "90074140");
            }
        }
    }
}

/// Flowering Artisan + Elmott: target picked while alive; damage kill fizzles destroy.
#[test]
fn artisan_damage_kills_spell_target_after_play_time_pick() {
    let db = load_db();
    let mut st = started(&db, 903);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, ARTISAN);
    let weak = put_field(&db, &mut st, opp, VANILLA);
    if let Some(f) = st.field_inst_mut(opp, weak) {
        f.defense = 2;
        f.max_defense = 2;
    }
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ELMOTT);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    assert!(
        st.field_inst(opp, weak).is_none(),
        "Artisan killed the target; spell part fizzled harmlessly"
    );
}

/// Majestic Conquest crest + Enhanced ally-follower pick: Fearless Soldier not a candidate.
#[test]
fn majestic_crest_summon_not_in_enhanced_play_pick() {
    let db = load_db();
    let mut st = started(&db, 904);
    let me = PlayerId::A;
    add_crest(&mut st, me, MAJESTIC_CREST, 2);
    let ally = put_field(&db, &mut st, me, VANILLA);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ENHANCE_PICK);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    assert!(
        !field_has(&st, me, FEARLESS),
        "Fearless Soldier not summoned before play-time pick"
    );
    let Phase::Choice { node, .. } = &st.phase else {
        unreachable!();
    };
    if let arena_engine::ChoiceNode::Targets { options, .. } = node {
        assert_eq!(options.len(), 2, "vanilla ally and the played follower");
        assert!(options.iter().any(|t| matches!(
            t,
            arena_engine::TargetOpt::Slot { slot, .. } if *slot == ally
        )));
        for t in options {
            if let arena_engine::TargetOpt::Slot { player, slot } = t {
                let id = st
                    .field_inst(*player, *slot)
                    .map(|c| c.card.as_str().to_string())
                    .unwrap_or_default();
                assert_ne!(id, FEARLESS);
            }
        }
    }
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    assert!(field_has(&st, me, FEARLESS));
}

/// Yuel crest + Ara: Fanfare pick before crest evolve; Evolve pick after.
#[test]
fn yuel_crest_fanfare_pick_before_evolve_pick() {
    let db = load_db();
    let mut st = started(&db, 905);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    add_crest(&mut st, me, YUEL_CREST, 4);
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, me, VANILLA);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ARA);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    let ara = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ARA);
    assert!(ara.is_some_and(|c| !c.evolved), "crest has not evolved yet");
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    let ara = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ARA)
        .expect("Ara");
    assert!(ara.evolved, "crest evolved after fanfare pick");
    assert!(
        !field_has(&st, opp, TANK) || field_def(&st, opp, TANK).unwrap_or(10) < 10,
        "fanfare resolved after play-time pick"
    );
}

/// Itsurugi mode before play reaction when a reaction is present.
#[test]
fn itsurugi_mode_before_play_reaction() {
    let db = load_db();
    let mut st = started(&db, 906);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, IMARI);
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.evolved = true;
    }
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ITSURUGI);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    assert!(
        !field_has(&st, me, "90074140"),
        "mode before Imari play reaction"
    );
}

/// Spilling Red: both picks at play, in text order.
#[test]
fn spilling_red_both_picks_at_play_in_order() {
    let db = load_db();
    let mut st = started(&db, 907);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, VANILLA);
    play_id(&db, &mut st, me, SPILLING_RED);
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    choose(&db, &mut st, 0);
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    assert!(!field_has(&st, opp, TANK));
}

/// Draw-then-pick fixture: only the draw is play-time prefix; discard picks at resolution.
#[test]
fn pick_after_non_selecting_effect_stays_at_resolution() {
    let db = load_db();
    let mut st = started(&db, 908);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, VANILLA);
    play_id(&db, &mut st, me, DRAW_THEN_PICK);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(
        pending_kind(&st),
        arena_engine::PendingKind::EffectSelect,
        "discard picks at resolution after draw"
    );
    drain_choice(&db, &mut st);
    assert!(st.player(me).hand.is_empty() || st.player(me).hand.len() <= 1);
}

/// Regression: evolve pick still after evolve reactions (Faith).
#[test]
fn evolve_pick_still_after_evolve_reactions_regression() {
    let db = load_db();
    let mut st = started_decks(&db, 909, &["10614120"], &["88001110"]);
    let me = PlayerId::A;
    set_round(&mut st, me, 7);
    st.player_mut(me).sep = 1;
    st.player_mut(me).faith = 2;
    put_field(&db, &mut st, me, "10514120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(0),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(st.player(me).faith, 3);
}

/// Regression: play without reactions is unchanged.
#[test]
fn play_without_reactions_unchanged_regression() {
    let db = load_db();
    let mut st = started(&db, 910);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, VANILLA);
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    let mut st2 = started(&db, 910);
    give_pp(&mut st2, me, 1, 1);
    st2.player_mut(me).hand.clear();
    let h2 = put_hand(&db, &mut st2, me, VANILLA);
    play(&db, &mut st2, h2);
    drain_choice(&db, &mut st2);
    assert_eq!(arena_engine::hash(&st), arena_engine::hash(&st2));
    assert_eq!(st.picks.len(), st2.picks.len());
}

/// Earth Rite pay gates the play-time walk: no sigils → no mode prompt.
#[test]
fn sweet_abomination_no_earth_no_play_time_prompt() {
    let db = load_db();
    let mut st = started(&db, 920);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    st.player_mut(me).earth = 0;
    put_hand(&db, &mut st, me, SWEET_ABOMINATION);
    play(&db, &mut st, 0);
    assert!(
        matches!(st.phase, Phase::Main),
        "unpayable Earth Rite: no play-time mode prompt"
    );
    assert_eq!(st.player(me).earth, 0, "earth not spent at play");
}

/// Payable Earth Rite: mode prompt at play before play reactions (WoG included).
#[test]
fn sweet_abomination_with_earth_mode_at_play_before_reactions() {
    let db = load_db();
    let mut st = started(&db, 921);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    st.player_mut(me).earth = 1;
    let wog = put_field(&db, &mut st, me, WORLD);
    if let Some(w) = st.field_inst_mut(me, wog) {
        w.countdown = Some(5);
    }
    put_field(&db, &mut st, me, COST_FIVE_ALLY);
    let tank = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, tank) {
        f.defense = 3;
        f.max_defense = 3;
    }
    put_hand(&db, &mut st, me, SWEET_ABOMINATION);
    play(&db, &mut st, 0);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::PlaySelect);
    assert_eq!(
        st.player(me).earth,
        1,
        "earth not spent until mode resolves"
    );
    assert_eq!(
        wog_countdown(&st, me),
        Some(5),
        "World of Games play reaction waits until after the mode pick"
    );
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).earth, 0, "earth spent when Fanfare resolves");
    assert_eq!(
        wog_countdown(&st, me),
        Some(4),
        "WoG advanced after Sweet Abomination (cost 5) fully resolved"
    );
    assert!(field_has(&st, me, SWEET_ABOMINATION));
    assert!(
        !field_has(&st, opp, TANK),
        "mode 0: 3 damage to all enemy followers"
    );
}

/// Regression: Engage pick still at resolution.
#[test]
fn engage_pick_stays_at_resolution_regression() {
    let db = load_db();
    let mut st = started(&db, 911);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    let slot = put_field(&db, &mut st, me, "10001210");
    let legal = legal_actions(&db, &st);
    let engage = legal
        .iter()
        .find(|a| matches!(a, Action::Engage { slot: s } if *s == Slot(slot)))
        .cloned()
        .expect("engage legal");
    apply(&db, &mut st, engage).expect("engage");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::EffectSelect);
}
