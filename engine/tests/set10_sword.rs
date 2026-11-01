//! Set 10 Swordcraft — Newsie / Forensics / Dilettante / Almaelys / No Life /
//! Bejeweled Lady. Assert against card text and crest sequence rules.

use arena_engine::{apply, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const NEWSIE: &str = "11021110";
const BEJEWELED: &str = "11021120";
const CHARGE: &str = "11021310";
const ARCHITECT: &str = "11022110";
const ERRANT: &str = "11022120";
const FORENSICS: &str = "11022310";
const DILETTANTE: &str = "11023110";
const CROSS_EXAM: &str = "11023310";
const NO_LIFE: &str = "11024110";
const ALMAELYS: &str = "11024120";
const ALMAELYS_CREST: &str = "crest:11024120";
const BLADE: &str = "90021310";
const GOBLET: &str = "90021320";
const GOLD: &str = "90021350";
const TANK: &str = "89500001";
const VANILLA: &str = "88001110";
const ENGAGE_HELPER: &str = "89500021"; // non-Clue Engage amulet
const SPELL_PING: &str = "88001640"; // cheap spell (restore) to trigger Lady

const SET10_SWORD: &[&str] = &[
    NEWSIE, BEJEWELED, CHARGE, ARCHITECT, ERRANT, FORENSICS, DILETTANTE, CROSS_EXAM, NO_LIFE,
    ALMAELYS,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    give_pp(st, st.active, pp, pp.max(10));
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn engage_slot(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    give_pp(st, st.active, 10, 10);
    apply(db, st, Action::Engage { slot: Slot(slot) }).expect("engage");
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

fn hand_newsie(st: &arena_engine::State, who: PlayerId) -> &arena_engine::CardInstance {
    st.player(who)
        .hand
        .iter()
        .find(|c| c.card.as_str() == NEWSIE)
        .expect("Newsie in hand")
}

fn crest_cd(st: &arena_engine::State, who: PlayerId, id: &str) -> Option<i32> {
    st.player(who)
        .crests
        .iter()
        .find(|c| c.id == id)
        .and_then(|c| c.countdown)
}

#[test]
fn set10_sword_require_supported() {
    let db = load_db();
    for id in SET10_SWORD {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.crest(ALMAELYS_CREST)
        .unwrap_or_else(|e| panic!("{ALMAELYS_CREST}: {e}"));
}

#[test]
fn newsie_clue_engage_swaps_thrice_enhance_exact_copy_keeps_stats() {
    let db = load_db();
    let mut st = started(&db, 10_200);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, NEWSIE);
    // Base 2/1. Each Clue Engage: swap then +1/+0.
    // 1: 1/2 +1 → 2/2
    // 2: 2/2 +1 → 3/2
    // 3: 2/3 +1 → 3/3
    for _ in 0..3 {
        let slot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
        engage_slot(&db, &mut st, slot);
        // Evidence at 1 clue is destroyed; re-put next loop.
    }
    let n = hand_newsie(&st, PlayerId::A);
    assert_eq!(
        (n.attack, n.defense),
        (3, 3),
        "three Clue Engages → swap+1 each time"
    );

    // Enhance (4): exact copy keeps stats.
    give_pp(&mut st, PlayerId::A, 4, 4);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == NEWSIE)
        .expect("newsie hand idx") as u8;
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    let bodies: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == NEWSIE)
        .map(|c| (c.attack, c.defense))
        .collect();
    assert_eq!(bodies.len(), 2, "Enhance summons exact copy");
    assert!(
        bodies.iter().all(|&s| s == (3, 3)),
        "exact copy keeps hand stats; got {bodies:?}"
    );

    // Negative: Engage non-Clue amulet does not swap.
    let mut st = started(&db, 10_201);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, NEWSIE);
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    let n = hand_newsie(&st, PlayerId::A);
    assert_eq!(
        (n.attack, n.defense),
        (2, 1),
        "negative: non-Clue Engage must not swap Newsie"
    );
}

#[test]
fn dichotomous_forensics_and_cross_exam_use_engaged_clue_flag() {
    let db = load_db();
    // Forensics: without Clue Engage → 5 damage; with → 10.
    let mut st = started(&db, 10_210);
    put_field(&db, &mut st, PlayerId::B, TANK); // 2/10
    play_pp(&db, &mut st, FORENSICS, 2);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(5),
        "no Clue Engage this turn → 5 damage"
    );

    let mut st = started(&db, 10_211);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let slot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    engage_slot(&db, &mut st, slot);
    // Fresh full-health tank for the 10-damage branch.
    for s in st.player_mut(PlayerId::B).field.iter_mut() {
        *s = None;
    }
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, FORENSICS, 2);
    assert!(
        !field_has(&st, PlayerId::B, TANK),
        "engaged Clue → 10 damage destroys 10-def tank"
    );

    // Cross-Examination: Evidence always; Newsie only if engaged Clue.
    let mut st = started(&db, 10_212);
    play_pp(&db, &mut st, CROSS_EXAM, 1);
    assert!(field_has(&st, PlayerId::A, EVIDENCE));
    assert!(
        !field_has(&st, PlayerId::A, NEWSIE),
        "negative: no Clue Engage → no Newsie"
    );

    let mut st = started(&db, 10_213);
    let slot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    engage_slot(&db, &mut st, slot);
    play_pp(&db, &mut st, CROSS_EXAM, 1);
    assert!(
        field_has(&st, PlayerId::A, EVIDENCE),
        "summons Evidence again"
    );
    assert!(
        field_has(&st, PlayerId::A, NEWSIE),
        "engaged Clue this turn → summon Newsie"
    );
}

#[test]
fn dilettante_sets_newsie_copies_to_self_stats_not_exact() {
    let db = load_db();
    // Printed Dilettante 5/1 — X=5 ≠ Newsie's printed attack 2.
    let mut st = started(&db, 10_220);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, DILETTANTE);
    let dil = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == DILETTANTE)
        .expect("dilettante");
    let (x, y) = (dil.attack, dil.defense);
    assert_eq!((x, y), (5, 1));
    assert_ne!(x, 2, "X must differ from Newsie's printed attack");

    give_pp(&mut st, PlayerId::A, 6, 6);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == DILETTANTE)
        .unwrap() as u8;
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    let news: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == NEWSIE)
        .collect();
    assert_eq!(news.len(), 2, "summons 2 Newsie copies");
    assert!(
        news.iter()
            .all(|c| c.attack == x && c.defense == y && c.max_defense == y),
        "copies set attack={x}, defense and max_defense={y}; got {:?}",
        news.iter()
            .map(|c| (c.attack, c.defense, c.max_defense))
            .collect::<Vec<_>>()
    );
    // Negative: copies are not Newsie's printed 2/1.
    assert!(
        news.iter().all(|c| !(c.attack == 2 && c.defense == 1)),
        "negative: must not leave Newsie at printed 2/1"
    );

    // Evolve: Give all other allied followers Bane.
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == DILETTANTE)
                .map(|_| i as u8)
        })
        .expect("Dilettante");
    evolve_slot(&db, &mut st, slot, false);
    assert!(
        st.player(PlayerId::A)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == NEWSIE)
            .all(|c| c.traits.bane == Some(true)),
        "Evolve: other allied followers gain Bane"
    );
    let dil = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == DILETTANTE)
        .unwrap();
    assert_ne!(
        dil.traits.bane,
        Some(true),
        "negative: Evolve Bane is other-only"
    );
}

#[test]
fn almaelys_super_evolve_delays_crest_four_activations_without_se_only_two() {
    let db = load_db();
    // With SE Delay 2: Countdown 2→4 → four end-of-turn activations Blade, Goblet, Blade, Goblet.
    let mut st = started(&db, 10_230);
    play_pp(&db, &mut st, ALMAELYS, 3);
    assert!(hand_has(&st, PlayerId::A, GOLD));
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == ALMAELYS)
                .map(|_| i as u8)
        })
        .expect("almaelys");
    // Super-Evolve fires Evolve: (gain crest CD 2) and Super-Evolve: (delay +2) together.
    evolve_slot(&db, &mut st, slot, true);
    assert_eq!(
        crest_cd(&st, PlayerId::A, ALMAELYS_CREST),
        Some(4),
        "SE: gain crest (2) then Delay by 2 → Countdown 4"
    );

    let mut seq = Vec::new();
    for _ in 0..4 {
        clear_hand(&mut st, PlayerId::A);
        end_turn(&db, &mut st); // A's end → crest fires; B's turn
        if hand_has(&st, PlayerId::A, BLADE) {
            let cost = st
                .player(PlayerId::A)
                .hand
                .iter()
                .find(|c| c.card.as_str() == BLADE)
                .map(|c| c.cost)
                .unwrap();
            assert_eq!(cost, 0, "Blade cost set to 0");
            seq.push("blade");
        } else if hand_has(&st, PlayerId::A, GOBLET) {
            let cost = st
                .player(PlayerId::A)
                .hand
                .iter()
                .find(|c| c.card.as_str() == GOBLET)
                .map(|c| c.cost)
                .unwrap();
            assert_eq!(cost, 0, "Goblet cost set to 0");
            seq.push("goblet");
        }
        end_turn(&db, &mut st); // back to A (crest ticks at A's start)
    }
    assert_eq!(
        seq,
        vec!["blade", "goblet", "blade", "goblet"],
        "four activations cycling Blade, Goblet, Blade, Goblet"
    );

    // Negative: without SE, only two activations before crest expires (Countdown 2).
    let mut st = started(&db, 10_231);
    play_pp(&db, &mut st, ALMAELYS, 3);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == ALMAELYS)
                .map(|_| i as u8)
        })
        .expect("almaelys");
    evolve_slot(&db, &mut st, slot, false);
    let mut seq = Vec::new();
    for _ in 0..3 {
        clear_hand(&mut st, PlayerId::A);
        end_turn(&db, &mut st);
        if hand_has(&st, PlayerId::A, BLADE) {
            seq.push("blade");
        } else if hand_has(&st, PlayerId::A, GOBLET) {
            seq.push("goblet");
        } else {
            seq.push("none");
        }
        end_turn(&db, &mut st);
    }
    assert_eq!(
        &seq[..2],
        &["blade", "goblet"][..],
        "without SE: Blade then Goblet"
    );
    assert!(
        seq.get(2) == Some(&"none")
            || !st
                .player(PlayerId::A)
                .crests
                .iter()
                .any(|c| c.id == ALMAELYS_CREST),
        "negative: crest expires after two end-of-turn activations without SE delay"
    );
}

#[test]
fn no_life_atk_def_branches_bane_drain_ward_storm_or_neither() {
    let db = load_db();

    // atk < def (printed 5/6, no swaps): copy Bane + self Drain.
    let mut st = started(&db, 10_240);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, NO_LIFE);
    give_pp(&mut st, PlayerId::A, 9, 9);
    play(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    let bodies: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == NO_LIFE)
        .collect();
    assert_eq!(bodies.len(), 2);
    let (self_c, copy_c) = if bodies[0].traits.drain == Some(true) {
        (bodies[0], bodies[1])
    } else {
        (bodies[1], bodies[0])
    };
    assert_eq!(self_c.traits.drain, Some(true), "atk<def → self Drain");
    assert_eq!(copy_c.traits.bane, Some(true), "atk<def → copy Bane");
    assert_ne!(copy_c.traits.ward, Some(true), "negative: not Ward branch");
    assert_ne!(
        self_c.traits.storm,
        Some(true),
        "negative: not Storm branch"
    );

    // atk > def via Clue Engage swaps: 5/6 → 6/5 +1 = 7/5.
    let mut st = started(&db, 10_241);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, NO_LIFE);
    let slot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    engage_slot(&db, &mut st, slot);
    {
        let n = st
            .player(PlayerId::A)
            .hand
            .iter()
            .find(|c| c.card.as_str() == NO_LIFE)
            .unwrap();
        assert_eq!(
            (n.attack, n.defense, n.max_defense),
            (7, 5, 5),
            "one Clue Engage → 7/5 exact"
        );
    }
    give_pp(&mut st, PlayerId::A, 9, 9);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == NO_LIFE)
        .unwrap() as u8;
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    let bodies: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == NO_LIFE)
        .collect();
    assert_eq!(bodies.len(), 2);
    let self_c = bodies
        .iter()
        .find(|c| c.traits.storm == Some(true))
        .expect("self Storm");
    let copy_c = bodies
        .iter()
        .find(|c| c.traits.ward == Some(true))
        .expect("copy Ward");
    assert!(self_c.id != copy_c.id);
    assert_eq!(
        (copy_c.attack, copy_c.defense, copy_c.max_defense),
        (7, 5, 5),
        "exact copy keeps 7/5"
    );
    assert_eq!(
        (self_c.attack, self_c.defense, self_c.max_defense),
        (7, 5, 5),
        "self also 7/5 after Fanfare"
    );
    assert_ne!(
        self_c.traits.drain,
        Some(true),
        "negative: atk>def is not Drain branch"
    );

    // equal atk/def → neither branch.
    let mut st = started(&db, 10_242);
    clear_hand(&mut st, PlayerId::A);
    let h = put_hand(&db, &mut st, PlayerId::A, NO_LIFE);
    {
        let c = &mut st.player_mut(PlayerId::A).hand[h as usize];
        c.attack = 5;
        c.defense = 5;
        c.max_defense = 5;
    }
    give_pp(&mut st, PlayerId::A, 9, 9);
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    let bodies: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == NO_LIFE)
        .collect();
    assert_eq!(bodies.len(), 2);
    assert!(
        bodies.iter().all(|c| c.traits.bane != Some(true)
            && c.traits.ward != Some(true)
            && c.traits.drain != Some(true)
            && c.traits.storm != Some(true)),
        "equal atk/def → neither keyword branch"
    );
}

#[test]
fn bejeweled_lady_once_per_turn_spell_damages_all_enemies_including_leader() {
    let db = load_db();
    let mut st = started(&db, 10_250);
    play_pp(&db, &mut st, BEJEWELED, 5);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let opp_leader0 = leader_def(&st, PlayerId::B);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();

    // Play a cheap spell — use Onward is expensive; use a fixture restore/ping spell.
    // 89500020 restores leader — still a spell play.
    play_pp(&db, &mut st, SPELL_PING, 1);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        opp_leader0 - 2,
        "spell play deals 2 to enemy leader"
    );
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 2),
        "spell play deals 2 to enemy followers"
    );

    // Negative: second spell same turn does not deal again (once per turn).
    let leader1 = leader_def(&st, PlayerId::B);
    let tank1 = field_def(&st, PlayerId::B, TANK).unwrap_or(0);
    play_pp(&db, &mut st, SPELL_PING, 1);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader1,
        "negative: once per turn — second spell deals no Lady damage"
    );
    assert_eq!(
        field_def(&st, PlayerId::B, TANK).unwrap_or(0),
        tank1,
        "negative: once per turn on followers too"
    );
}

#[test]
fn newsie_last_words_summons_evidence_no_swap_while_on_field() {
    let db = load_db();
    let mut st = started(&db, 10_260);
    play_pp(&db, &mut st, NEWSIE, 2);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == NEWSIE)
                .map(|_| i as u8)
        })
        .expect("Newsie");
    let (atk0, def0) = {
        let n = st.field_inst(PlayerId::A, slot).unwrap();
        (n.attack, n.defense)
    };
    // Negative: Clue Engage while Newsie is on the field does not swap it.
    let eslot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    engage_slot(&db, &mut st, eslot);
    let n = st.field_inst(PlayerId::A, slot).expect("Newsie still on field");
    assert_eq!(
        (n.attack, n.defense),
        (atk0, def0),
        "negative: in-hand Engage trigger does not swap a Newsie on the field"
    );

    // Last Words: destroy Newsie → Divine Evidence.
    st.player_mut(PlayerId::A).field[slot as usize]
        .as_mut()
        .unwrap()
        .defense = 0;
    play_pp(&db, &mut st, "88001710", 1); // destroy random highest
    assert!(
        field_has(&st, PlayerId::A, EVIDENCE),
        "Last Words: Summon a Divine Evidence"
    );
}

#[test]
fn bejeweled_lady_fanfare_adds_gilded_necklace() {
    let db = load_db();
    let mut st = started(&db, 10_261);
    clear_hand(&mut st, PlayerId::A);
    play_pp(&db, &mut st, BEJEWELED, 5);
    assert!(
        hand_has(&st, PlayerId::A, "90021340"),
        "Fanfare: Add a Gilded Necklace to your hand"
    );
    // Negative: Necklace is not summoned to field.
    assert!(
        !field_has(&st, PlayerId::A, "90021340"),
        "negative: Necklace goes to hand, not field"
    );
}

#[test]
fn onward_charge_summons_two_evolved_knights_with_barrier() {
    let db = load_db();
    const KNIGHT: &str = "90021120";
    let mut st = started(&db, 10_262);
    play_pp(&db, &mut st, CHARGE, 7);
    let knights: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == KNIGHT)
        .collect();
    assert_eq!(knights.len(), 2, "summons 2 Steelclad Knights");
    assert!(
        knights.iter().all(|c| c.evolved && c.traits.barrier == Some(true)),
        "both evolved with Barrier"
    );

    // Negative: playing nothing else leaves no third knight.
    assert_eq!(
        st.player(PlayerId::A).field_count(),
        2,
        "negative: exactly two knights, no extras"
    );
}

#[test]
fn cheeky_architect_in_hand_swap_fanfare_x_damage_evolve_newsie() {
    let db = load_db();
    let mut st = started(&db, 10_263);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, ARCHITECT); // printed 1/4
    // One Clue Engage: 1/4 → 4/1 +1 → 5/1.
    let slot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    engage_slot(&db, &mut st, slot);
    let a = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == ARCHITECT)
        .unwrap();
    assert_eq!((a.attack, a.defense), (5, 1), "in-hand swap then +1/+0");
    let x = a.attack;

    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    give_pp(&mut st, PlayerId::A, 3, 3);
    let h = st
        .player(PlayerId::A)
        .hand
        .iter()
        .position(|c| c.card.as_str() == ARCHITECT)
        .unwrap() as u8;
    play(&db, &mut st, h);
    // Choose the tank.
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - x),
        "Fanfare deals X = this follower's attack ({x})"
    );
    assert!(field_has(&st, PlayerId::A, EVIDENCE), "Fanfare summons Evidence");

    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == ARCHITECT)
                .map(|_| i as u8)
        })
        .unwrap();
    evolve_slot(&db, &mut st, slot, false);
    assert!(
        field_has(&st, PlayerId::A, NEWSIE),
        "Evolve: Summon a Guileless Newsie"
    );

    // Negative: without Clue Engage, Architect stays 1/4 and deals 1.
    let mut st = started(&db, 10_264);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    play_pp(&db, &mut st, ARCHITECT, 3);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 1),
        "negative: printed attack 1 without hand swaps"
    );
}

#[test]
fn errant_knight_banish_requires_allied_amulet_se_replicates() {
    let db = load_db();
    // With allied amulet: banish chosen enemy.
    let mut st = started(&db, 10_265);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, VANILLA);
    play_pp(&db, &mut st, ERRANT, 7);
    if matches!(st.phase, Phase::Choice { .. }) {
        // Prefer tank if listed first; either way one enemy is banished.
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    let enemies = st.player(PlayerId::B).field_count();
    assert_eq!(enemies, 1, "banishes one enemy follower when allied amulet present");

    // Negative: no allied amulet → no banish.
    let mut st = started(&db, 10_266);
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, ERRANT, 7);
    assert!(
        field_has(&st, PlayerId::B, TANK),
        "negative: without allied amulet, Fanfare does not banish"
    );

    // Super-Evolve replicates Fanfare (needs amulet + enemy).
    let mut st = started(&db, 10_267);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, VANILLA);
    play_pp(&db, &mut st, ERRANT, 7);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == ERRANT)
                .map(|_| i as u8)
        })
        .unwrap();
    // Ensure amulet still present for SE replicate.
    if !field_has(&st, PlayerId::A, EVIDENCE) {
        put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    }
    if st.player(PlayerId::B).field_count() == 0 {
        put_field(&db, &mut st, PlayerId::B, TANK);
    }
    evolve_slot(&db, &mut st, slot, true);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        st.player(PlayerId::B).field_count(),
        0,
        "SE replicates Fanfare banish"
    );
}
