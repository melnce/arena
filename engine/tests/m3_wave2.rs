//! M3 wave 2 — Runecraft + Dragoncraft fixtures.

use arena_engine::{apply, Action, AttackTarget, Phase, Pick, PickChose, PickWhat, PlayerId, Slot};

mod common;
use common::*;

fn require_ok(db: &arena_engine::CardDb, ids: &[&str]) {
    for id in ids {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

fn set_round(st: &mut arena_engine::State, who: PlayerId, n: u32) {
    st.player_mut(who).turns_taken = n;
    st.turn = n;
    give_pp(st, who, n as i32, n as i32);
}

fn grant_evolve(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    let me = st.active;
    if st.player(me).turns_taken < 5 {
        set_round(st, me, 5);
    }
    st.player_mut(me).ep = st.player(me).ep.max(1);
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
}

fn grant_super_evolve(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    let me = st.active;
    if st.player(me).turns_taken < 7 {
        set_round(st, me, 7);
    }
    st.player_mut(me).sep = st.player(me).sep.max(1);
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
}

fn put_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect("card");
    let inst = arena_engine::CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

fn drain_choice(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn attack_leader(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack leader");
}

fn attack_follower(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, def: u8) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Slot(Slot(def)),
        },
    )
    .expect("attack follower");
}

fn deck_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    let id = cid(id);
    st.player(who).deck.iter().filter(|c| c.card == id).count()
}

fn crest_has(st: &arena_engine::State, who: PlayerId, id: &str) -> bool {
    st.player(who).crests.iter().any(|c| c.id == id)
}

#[test]
fn require_supported_runecraft_m3() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10431110", "10431120", "10431310", "10432110", "10432120", "10432310", "10433110",
            "10434110", "10531110", "10531120", "10531310", "10532110", "10532120", "10532310",
            "10533110", "10533310", "10534110", "10631120", "10631310", "10632110", "10632120",
            "10632310", "10633110", "10634110", "10731110", "10731120", "10731310", "10732110",
            "10732120", "10732310", "10733310", "10734110", "10831110", "10831120", "10831310",
            "10832110", "10832310", "10832320", "10833310", "10834120", "10931120", "10931310",
            "10932120", "10934120", "90031110", "90031120", "90031310",
        ],
    );
}

#[test]
fn require_supported_dragoncraft_m3() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10441110", "10441120", "10441310", "10442110", "10442120", "10442310", "10443110",
            "10444110", "10541110", "10541120", "10541310", "10542110", "10542120", "10542310",
            "10543110", "10544120", "10641110", "10641120", "10641310", "10642110", "10642120",
            "10643110", "10643310", "10741120", "10741310", "10742110", "10742120", "10742310",
            "10743110", "10743310", "10744120", "10841110", "10841120", "10841130", "10842110",
            "10842120", "10842310", "10843110", "10843310", "10844110", "10941110", "10941120",
            "10941310", "10942110", "10942120", "10942310", "10943110", "10943310", "90041110",
            "90041120", "90041130",
        ],
    );
}

#[test]
fn suframare_spellboost_x_equals_attack_eot() {
    let db = load_db();
    let mut st = started(&db, 201);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10431120");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.attack = 3;
    }
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10831310");
    end_turn(&db, &mut st);
    let blast = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10831310")
        .expect("blast");
    assert_eq!(
        *blast.vars.get(&arena_engine::card::VarKey::X).unwrap_or(&0),
        5,
        "X starts at 2; three EOT boosts"
    );
}

#[test]
fn key_spirit_spellboosts_selected_hand_card_four_times() {
    let db = load_db();
    let mut st = started(&db, 202);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10931120");
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10831310");
    grant_evolve(&db, &mut st, slot);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let blast = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10831310")
        .expect("blast");
    assert_eq!(
        *blast.vars.get(&arena_engine::card::VarKey::X).unwrap_or(&0),
        6,
        "X starts at 2; four selected boosts"
    );
}

#[test]
fn stormy_blast_x_after_two_boosts() {
    let db = load_db();
    let mut st = started(&db, 203);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 10;
        f.max_defense = 10;
    }
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10831310");
    let slot = put_field(&db, &mut st, me, "10431120");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.attack = 2;
    }
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, me, 1, 1);
    let h = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "10831310")
        .expect("boosted blast") as u8;
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    let tank = st.player(opp).field[0].as_ref().unwrap();
    assert_eq!(tank.defense, 6, "X is 4 after two boosts (10 - 4)");
}

#[test]
fn amethyst_naptime_x_at_least_5() {
    let db = load_db();
    let mut st = started(&db, 204);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10833310");
    let slot = put_field(&db, &mut st, me, "10431120");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.attack = 5;
    }
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, me, 3, 3);
    let before_pp = st.player(me).pp;
    let h = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "10833310")
        .expect("boosted naptime") as u8;
    play(&db, &mut st, h);
    assert_eq!(st.player(me).leader_defense, 12, "restore 2 at X >= 5");
    assert!(
        st.player(me).pp > before_pp.saturating_sub(3),
        "recover 2 pp"
    );
}

#[test]
fn kitty_cunning_two_distinct_earth_rite_abilities() {
    let db = load_db();
    let mut st = started(&db, 205);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "90031210");
    st.player_mut(me).earth = 2;
    st.player_mut(me).leader_defense = 15;
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10532310");
    st.rng = arena_engine::GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::RandomUnused,
                among: None,
                chose: PickChose::Id("0".into()),
            },
            Pick {
                what: PickWhat::RandomUnused,
                among: None,
                chose: PickChose::Id("1".into()),
            },
        ],
        205,
    );
    give_pp(&mut st, me, 1, 1);
    apply(&db, &mut st, Action::Play { hand: h }).expect("kitty");
    assert!(field_has(&st, me, "90031110"), "ability 1: Clay Golem");
    assert_eq!(st.player(me).leader_defense, 17, "ability 2: restore 2");
    assert_eq!(st.player(me).earth, 0, "ability 3 (gain 3) was not picked");
}

#[test]
fn emperor_earth_rite_evolves_until_sigils_run_out() {
    let db = load_db();
    let mut st = started(&db, 206);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "90031210");
    st.player_mut(me).earth = 1;
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10533110");
    let golems: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "90031120")
        .collect();
    assert_eq!(golems.len(), 2);
    let evolved = golems.iter().filter(|c| c.evolved).count();
    assert_eq!(evolved, 1, "one sigil evolves only the first Golem");
    assert_eq!(st.player(me).earth, 0);
}

#[test]
fn grandeur_independent_per_follower_picks() {
    let db = load_db();
    let mut st = started(&db, 207);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, "90031110");
    put_deck(&db, &mut st, me, "90031120");
    put_field(&db, &mut st, me, "90031110");
    put_field(&db, &mut st, me, "90031110");
    st.rng = arena_engine::GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::MultisetPick,
                among: Some("deck".into()),
                chose: PickChose::Id("90031110".into()),
            },
            Pick {
                what: PickWhat::MultisetPick,
                among: Some("deck".into()),
                chose: PickChose::Id("90031120".into()),
            },
        ],
        207,
    );
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10533310");
    let ids: Vec<String> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect();
    assert!(ids.contains(&"90031110".to_string()));
    assert!(ids.contains(&"90031120".to_string()));
}

#[test]
fn grandeur_exact_copy_keeps_deck_cost() {
    let db = load_db();
    let mut st = started(&db, 221);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, "90031110");
    if let Some(c) = st.player_mut(me).deck.last_mut() {
        c.cost = 0;
    }
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10533310");
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "90031110")
        .expect("transformed into deck follower");
    assert_eq!(f.cost, 0, "exact copy keeps the deck-instance cost");
}

#[test]
fn lhynkal_ten_deck_copies_and_crest_max_defense() {
    let db = load_db();
    let mut st = started(&db, 208);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10534110");
    assert!(crest_has(&st, me, "crest:10534110"));
    let slot = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10534110")
        .expect("lhynkal") as u8;
    grant_super_evolve(&db, &mut st, slot);
    assert_eq!(deck_count(&st, me, "10534110"), 10);
    st.player_mut(opp).leader_defense = 20;
    st.player_mut(opp).leader_max = 20;
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, "10534110");
    play_id(&db, &mut st, me, "10534110");
    assert_eq!(st.player(opp).leader_max, 16);
    assert_eq!(st.player(opp).leader_defense, 16);
}

#[test]
fn lhynkal_five_entries_zero_max_is_lethal() {
    let db = load_db();
    let mut st = started(&db, 222);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10534110");
    assert!(crest_has(&st, me, "crest:10534110"));
    // Enter-before-fanfare: the first body does not trip its own crest.
    // Clear the slot so five later entries fit on a 5-wide board.
    st.player_mut(me).field = Default::default();
    st.player_mut(opp).leader_defense = 10;
    st.player_mut(opp).leader_max = 10;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    for _ in 0..5 {
        put_hand(&db, &mut st, me, "10534110");
        play_id(&db, &mut st, me, "10534110");
    }
    assert_eq!(st.player(opp).leader_max, 0);
    assert_eq!(st.player(opp).leader_defense, 0);
    assert_eq!(st.winner, Some(me));
    assert!(matches!(st.phase, Phase::Terminal));
}

#[test]
fn insomniac_witch_evolve_destroys_crest_and_last_words_fire() {
    let db = load_db();
    let mut st = started(&db, 209);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10532110");
    assert!(crest_has(&st, me, "crest:10532110"));
    let slot = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10532110")
        .unwrap() as u8;
    grant_evolve(&db, &mut st, slot);
    assert!(!crest_has(&st, me, "crest:10532110"));
    assert!(
        !field_has(&st, opp, "88001110") || st.player(opp).field[0].as_ref().unwrap().defense < 4,
        "crest Last Words dealt 3 to followers"
    );
}

#[test]
fn noble_philosopher_draws_returned_count() {
    let db = load_db();
    let mut st = started(&db, 210);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 3, 3);
    play_id(&db, &mut st, me, "10932120");
    assert_eq!(
        st.player(me).hand.len(),
        3,
        "returned 3, drew 3 (philosopher is on the field)"
    );
}

#[test]
fn ruinbringer_banish_count_into_split() {
    let db = load_db();
    let mut st = started(&db, 211);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, "10431120"); // 1
    put_deck(&db, &mut st, me, "10932120"); // 3
    put_deck(&db, &mut st, me, "10541310"); // 3
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 10;
        f.max_defense = 10;
    }
    let slot = put_field(&db, &mut st, me, "10543110");
    grant_super_evolve(&db, &mut st, slot);
    let tank = st.player(opp).field[0].as_ref().unwrap();
    assert_eq!(tank.defense, 7, "3 odd-cost cards banished → 3 split");
    assert_eq!(st.player(me).deck.len(), 0);
}

#[test]
fn beheading_eld_blades_two_discard_tiers() {
    let db = load_db();
    let mut st = started(&db, 212);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10643310");
    play_id(&db, &mut st, me, "10544120");
    let yube = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10544120")
        .unwrap() as u8;
    grant_evolve(&db, &mut st, yube);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let first = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10643310")
        .expect("tier 7→5");
    assert_eq!(first.cost, 5);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, me, 3, 3);
    play_id(&db, &mut st, me, "10544120");
    let unevolved = st
        .player(me)
        .field
        .iter()
        .enumerate()
        .find(|(_, s)| {
            s.as_ref()
                .is_some_and(|c| c.card.as_str() == "10544120" && !c.evolved)
        })
        .map(|(i, _)| i as u8)
        .unwrap();
    grant_evolve(&db, &mut st, unevolved);
    choose(&db, &mut st, 0);
    let second = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10643310")
        .expect("tier 5→3");
    assert_eq!(second.cost, 3);
}

#[test]
fn apathetic_gaze_transforms_hand_and_deck_copies() {
    let db = load_db();
    let mut st = started(&db, 213);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10741310");
    put_deck(&db, &mut st, me, "10741310");
    give_pp(&mut st, me, 4, 4);
    play_id(&db, &mut st, me, "10741310");
    assert!(
        st.player(me)
            .hand
            .iter()
            .any(|c| c.card.as_str() == "10742310"),
        "hand copy becomes Lazing Flame"
    );
    assert!(
        st.player(me)
            .deck
            .iter()
            .any(|c| c.card.as_str() == "10742310"),
        "deck copy becomes Lazing Flame"
    );
    assert_eq!(st.player(me).pp_max, 5);
}

#[test]
fn ephemeral_foxfire_deck_add_and_overflow_draw() {
    let db = load_db();
    let mut st = started(&db, 214);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 7);
    st.player_mut(me).hand.clear();
    let before_deck = deck_count(&st, me, "10843310");
    let before_hand = st.player(me).hand.len();
    play_id(&db, &mut st, me, "10843310");
    drain_choice(&db, &mut st);
    assert_eq!(deck_count(&st, me, "10843310"), before_deck + 1);
    assert_eq!(
        st.player(me).hand.len(),
        before_hand + 1,
        "Overflow draws a card"
    );
}

#[test]
fn ripper_last_words_copy_without_last_words() {
    let db = load_db();
    let mut st = started(&db, 215);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10941110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.defense = 0;
    }
    end_turn(&db, &mut st);
    let copy = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10941110")
        .expect("lw copy");
    assert!(
        !copy.printed_tags.contains("lastWords"),
        "Last Words removed from the hand copy"
    );
}

#[test]
fn ripper_storm_gate_across_three_turns() {
    let db = load_db();
    let mut st = started(&db, 216);
    let me = PlayerId::A;
    let rusher = put_field(&db, &mut st, me, "88001110");
    if let Some(f) = st.field_inst_mut(me, rusher) {
        f.traits.rush = Some(true);
        f.flags.summoning_sick = false;
    }
    attack_leader(&db, &mut st, rusher);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10941110");
    let rip = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10941110")
        .unwrap();
    assert_eq!(rip.traits.storm, Some(true));
}

#[test]
fn giada_second_attack_on_follower_strike() {
    let db = load_db();
    let mut st = started(&db, 217);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.attack = 0;
        f.defense = 20;
        f.max_defense = 20;
    }
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10843110");
    let slot = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10843110")
        .unwrap() as u8;
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.flags.summoning_sick = false;
    }
    attack_follower(&db, &mut st, slot, 0);
    let g = st.field_inst(me, slot).unwrap();
    assert_eq!(g.traits.barrier, Some(true));
    assert!(
        g.flags.attacks_left >= 1,
        "second attack available after follower Strike"
    );
    attack_follower(&db, &mut st, slot, 0);
}

#[test]
fn giada_leader_strike_gives_barrier_only() {
    let db = load_db();
    let mut st = started(&db, 223);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10843110");
    let slot = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10843110")
        .unwrap() as u8;
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.flags.summoning_sick = false;
    }
    attack_leader(&db, &mut st, slot);
    let g = st.field_inst(me, slot).unwrap();
    assert_eq!(g.traits.barrier, Some(true));
    assert_eq!(
        g.flags.attacks_left, 0,
        "leader Strike does not grant a second attack"
    );
}

#[test]
fn mari_cost_zero_expires_end_of_turn() {
    let db = load_db();
    let mut st = started(&db, 218);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10441120");
    let slot = put_field(&db, &mut st, me, "10441110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.base_cost = 3;
    }
    grant_super_evolve(&db, &mut st, slot);
    let mari = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10441120")
        .unwrap();
    assert_eq!(mari.cost, 0);
    end_turn(&db, &mut st);
    let mari = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10441120")
        .unwrap();
    assert_eq!(mari.cost, 2);
}

#[test]
fn meg_skybound_super_evolve_without_sep() {
    let db = load_db();
    let mut st = started(&db, 219);
    let me = PlayerId::A;
    set_round(&mut st, me, 10);
    let sep_before = st.player(me).sep;
    give_pp(&mut st, me, 3, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10443110");
    let meg = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10443110")
        .unwrap();
    assert!(meg.super_evolved, "Skybound Art super-evolves without SEP");
    assert_eq!(st.player(me).sep, sep_before);
}

#[test]
fn drache_x_counts_only_other_copies() {
    let db = load_db();
    let mut st = started(&db, 220);
    let me = PlayerId::A;
    st.player_mut(me).enter_counts.insert(cid("10844110"), 2);
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10844110");
    let d = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10844110")
        .unwrap();
    // play increments to 3; X = 3-1 = 2; +2/+2 then evolve +2/+2 → 8/8
    assert_eq!(d.attack, 8);
    assert_eq!(d.defense, 8);
    assert!(d.evolved);
}

#[test]
fn blade_of_the_crestpetal_empty_deck_x_is_zero() {
    let db = load_db();
    let mut st = started(&db, 221);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).deck.clear();
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 5;
        f.max_defense = 5;
    }
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10541310");
    let tank = st.player(opp).field[0].as_ref().unwrap();
    assert_eq!(tank.defense, 5, "no follower drawn → X = 0");
}

#[test]
fn yube_crest_once_per_turn_half() {
    let db = load_db();
    let mut st = started(&db, 222);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    play_id(&db, &mut st, me, "10544120");
    let yube = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10544120")
        .unwrap() as u8;
    grant_evolve(&db, &mut st, yube);
    drain_choice(&db, &mut st);
    assert!(crest_has(&st, me, "crest:10544120"));
    put_field(&db, &mut st, PlayerId::B, "88001110");
    if let Some(f) = st.field_inst_mut(PlayerId::B, 0) {
        f.defense = 20;
        f.max_defense = 20;
        f.attack = 0;
    }
    let orca = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "90041130")
        .unwrap() as u8;
    if let Some(f) = st.field_inst_mut(me, orca) {
        f.flags.summoning_sick = false;
    }
    let hand_before = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == "90041130")
        .count();
    attack_follower(&db, &mut st, orca, 0);
    let hand_mid = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == "90041130")
        .count();
    assert_eq!(hand_mid, hand_before + 1, "first marine attack adds orca");
    let orca2 = put_field(&db, &mut st, me, "90041130");
    if let Some(f) = st.field_inst_mut(me, orca2) {
        f.flags.summoning_sick = false;
    }
    attack_follower(&db, &mut st, orca2, 0);
    let hand_after = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == "90041130")
        .count();
    assert_eq!(
        hand_after, hand_mid,
        "once-per-turn half does not add a second orca"
    );
    let buffed = st.field_inst(me, orca2).unwrap();
    assert!(
        buffed.attack > 2,
        "+1/+0 still applies on the second attack"
    );
}
