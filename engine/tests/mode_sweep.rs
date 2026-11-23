//! Every player-picked mode option must change game state (no silent no-ops).
#![allow(dead_code, clippy::needless_lifetimes)]

use arena_engine::card::{
    Ability, Card, ChooseBy, Crest, Effect, Mode, PoolPick, Selector, SelectorKind, Side, Zone,
};
use arena_engine::{apply, hash, Action, CardDb, CardId, ChoiceNode, Phase, PlayerId, State};

mod common;
use common::*;

const VANILLA: &str = "88001110";
const TANK: &str = "89500001";
const CLUE: &str = "90001210";
const SPELL_HAND: &str = "10051310";
const FAITH: &str = "faith:10614120";
const CREST_A: &str = "crest:10714110";
const CREST_B: &str = "crest:10412310";

/// (card id, choose index in ability/mode effect list, option index)
type CaseKey = (String, u8, u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HarnessTrigger {
    Play,
    EvolveField,
    SuperEvolveField,
    Engage,
    CardStartOfTurn,
    CrestStartOfTurn,
}

#[derive(Clone)]
struct ModeCase {
    card_key: String,
    card_id: CardId,
    trigger: HarnessTrigger,
    choose_ord: u8,
    option_ord: u8,
    by: ChooseBy,
    option_printed: String,
}

struct Exclusion {
    card: &'static str,
    choose: u8,
    option: u8,
    _class: &'static str,
    reason: &'static str,
}

/// Options exempt from the gate (conditions, harness limits, owner questions).
/// Never pin wrong engine behavior — only quoted conditions or gaps.
const EXCLUSIONS: &[Exclusion] = &[];

fn excluded(key: &CaseKey) -> Option<&'static str> {
    EXCLUSIONS
        .iter()
        .find(|e| e.card == key.0 && e.choose == key.1 && e.option == key.2)
        .map(|e| e.reason)
}

fn trigger_of(ab: &Ability) -> HarnessTrigger {
    match ab {
        Ability::Fanfare { .. }
        | Ability::Spellboost { .. }
        | Ability::Enter { .. }
        | Ability::LastWords { .. } => HarnessTrigger::Play,
        Ability::Evolve { .. } | Ability::AnyEvolve { .. } => HarnessTrigger::EvolveField,
        Ability::SuperEvolve { .. } | Ability::AnySuperEvolve { .. } => {
            HarnessTrigger::SuperEvolveField
        }
        Ability::Engage { .. } => HarnessTrigger::Engage,
        Ability::StartOfTurn { .. } => HarnessTrigger::CardStartOfTurn,
        _ => HarnessTrigger::Play,
    }
}

fn collect_chooses_in_effects(
    effects: &[Effect],
    out: &mut Vec<(u8, u8, ChooseBy, String)>,
    choose_ord: &mut u8,
) {
    for e in effects {
        if let Effect::Choose {
            by,
            options,
            options_from,
            ..
        } = e
        {
            if options_from.is_some() {
                *choose_ord += 1;
                continue;
            }
            if let Some(opts) = options {
                if opts.is_empty() {
                    *choose_ord += 1;
                    continue;
                }
                for (oi, o) in opts.iter().enumerate() {
                    out.push((*choose_ord, oi as u8, *by, o.printed.clone()));
                }
            }
            *choose_ord += 1;
        }
        if let Effect::If {
            then, else_effects, ..
        } = e
        {
            let co = *choose_ord;
            collect_chooses_in_effects(then, out, choose_ord);
            if let Some(els) = else_effects {
                collect_chooses_in_effects(els, out, choose_ord);
            }
            *choose_ord = co;
        }
        if let Effect::Sequence { steps, .. } = e {
            for s in steps {
                collect_chooses_in_effects(&s.effects, out, choose_ord);
            }
        }
    }
}

fn collect_card_cases(
    card: &Card,
    card_key: &str,
    trigger: HarnessTrigger,
    effects: &[Effect],
) -> Vec<ModeCase> {
    let mut raw = Vec::new();
    let mut ord = 0u8;
    collect_chooses_in_effects(effects, &mut raw, &mut ord);
    let id = card.id();
    raw.into_iter()
        .filter(|(_, _, by, _)| *by == ChooseBy::Player)
        .map(|(co, oi, by, printed)| ModeCase {
            card_key: card_key.to_string(),
            card_id: id,
            trigger,
            choose_ord: co,
            option_ord: oi,
            by,
            option_printed: printed,
        })
        .collect()
}

fn enumerate_cases(db: &CardDb) -> Vec<ModeCase> {
    let mut out = Vec::new();
    for card in db.cards.values() {
        if card.token() {
            continue;
        }
        let key = card.id().as_str().to_string();
        for ab in card.abilities() {
            let trig = trigger_of(ab);
            out.extend(collect_card_cases(card, &key, trig, ab.effects()));
        }
        for m in card.modes() {
            match m {
                Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
                    out.extend(collect_card_cases(
                        card,
                        &key,
                        HarnessTrigger::Play,
                        effects,
                    ));
                }
                Mode::Crystallize { abilities, .. } => {
                    if let Some(abs) = abilities {
                        for ab in abs {
                            out.extend(collect_card_cases(
                                card,
                                &key,
                                HarnessTrigger::Play,
                                ab.effects(),
                            ));
                        }
                    }
                }
            }
        }
    }
    for crest in db.crests.values() {
        for ab in crest.abilities() {
            out.extend(collect_crest_cases(crest, ab));
        }
    }
    out
}

fn collect_crest_cases(crest: &Crest, ab: &Ability) -> Vec<ModeCase> {
    let key = crest.id.clone();
    let mut raw = Vec::new();
    let mut ord = 0u8;
    collect_chooses_in_effects(ab.effects(), &mut raw, &mut ord);
    raw.into_iter()
        .filter(|(_, _, by, _)| *by == ChooseBy::Player)
        .map(|(co, oi, by, printed)| ModeCase {
            card_key: key.clone(),
            card_id: CardId::parse("88001110").expect("pad"),
            trigger: HarnessTrigger::CrestStartOfTurn,
            choose_ord: co,
            option_ord: oi,
            by,
            option_printed: printed,
        })
        .collect()
}

fn ability_for_trigger(card: &Card, trigger: HarnessTrigger) -> Option<&Ability> {
    card.abilities().iter().find(|ab| {
        matches!(
            (trigger, ab),
            (HarnessTrigger::Play, Ability::Fanfare { .. })
                | (HarnessTrigger::Play, Ability::Spellboost { .. })
                | (HarnessTrigger::Play, Ability::Enter { .. })
                | (HarnessTrigger::EvolveField, Ability::Evolve { .. })
                | (HarnessTrigger::EvolveField, Ability::AnyEvolve { .. })
                | (
                    HarnessTrigger::SuperEvolveField,
                    Ability::SuperEvolve { .. }
                )
                | (
                    HarnessTrigger::SuperEvolveField,
                    Ability::AnySuperEvolve { .. }
                )
                | (HarnessTrigger::Engage, Ability::Engage { .. })
                | (HarnessTrigger::CardStartOfTurn, Ability::StartOfTurn { .. })
        )
    })
}

fn option_effects_for(case: &ModeCase, db: &CardDb) -> Vec<Effect> {
    if case.card_key.starts_with("crest:") {
        let crest = db.crest(&case.card_key).expect("crest");
        for ab in crest.abilities() {
            if matches!(ab, Ability::StartOfTurn { .. }) {
                return effects_for_option(ab.effects(), case.choose_ord, case.option_ord);
            }
        }
        return vec![];
    }
    let card = db.card(case.card_id).expect("card");
    if let Some(ab) = ability_for_trigger(card, case.trigger) {
        return effects_for_option(ab.effects(), case.choose_ord, case.option_ord);
    }
    for m in card.modes() {
        let slice = match m {
            Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => effects,
            Mode::Crystallize { .. } => continue,
        };
        let effs = effects_for_option(slice, case.choose_ord, case.option_ord);
        if !effs.is_empty() {
            return effs;
        }
    }
    vec![]
}

fn effects_for_option(effects: &[Effect], choose_ord: u8, option_ord: u8) -> Vec<Effect> {
    let mut seen = 0u8;
    let mut found = Vec::new();
    walk_effects_for_option(effects, choose_ord, option_ord, &mut seen, &mut found);
    found
}

fn walk_effects_for_option(
    effects: &[Effect],
    want_choose: u8,
    want_option: u8,
    seen: &mut u8,
    out: &mut Vec<Effect>,
) {
    for e in effects {
        if let Effect::Choose {
            options,
            options_from,
            ..
        } = e
        {
            if options_from.is_some() {
                *seen += 1;
                continue;
            }
            if *seen == want_choose {
                if let Some(opts) = options {
                    if let Some(opt) = opts.get(want_option as usize) {
                        out.clone_from(&opt.effects);
                        return;
                    }
                }
            }
            *seen += 1;
        }
        if let Effect::If {
            then, else_effects, ..
        } = e
        {
            walk_effects_for_option(then, want_choose, want_option, seen, out);
            if out.is_empty() {
                if let Some(els) = else_effects {
                    walk_effects_for_option(els, want_choose, want_option, seen, out);
                }
            }
        }
        if out.is_empty() {
            if let Effect::Sequence { steps, .. } = e {
                for s in steps {
                    walk_effects_for_option(&s.effects, want_choose, want_option, seen, out);
                    if !out.is_empty() {
                        return;
                    }
                }
            }
        }
    }
}

#[derive(Default, Clone)]
struct ZoneSnap {
    leader_hp: i32,
    hand_len: usize,
    field_cards: Vec<String>,
    crest_ids: Vec<String>,
    amulet_on_field: usize,
    deck_len: usize,
    cemetery_len: usize,
    pp: i32,
    ep: i32,
    sep: i32,
    faith: i32,
    earth: i32,
    shadows: i32,
}

fn snap_player(st: &State, who: PlayerId) -> ZoneSnap {
    let p = st.player(who);
    ZoneSnap {
        leader_hp: p.leader_defense,
        hand_len: p.hand.len(),
        field_cards: p
            .field
            .iter()
            .flatten()
            .map(|c| format!("{}:{}:{}", c.card, c.attack, c.defense))
            .collect(),
        crest_ids: p.crests.iter().map(|c| c.id.clone()).collect(),
        amulet_on_field: p
            .field
            .iter()
            .flatten()
            .filter(|c| c.kind == arena_engine::CardKind::Amulet)
            .count(),
        deck_len: p.deck.len(),
        cemetery_len: p.cemetery.len(),
        pp: p.pp,
        ep: p.ep,
        sep: p.sep,
        faith: p.faith,
        earth: p.earth,
        shadows: p.shadows,
    }
}

#[derive(Clone, Default)]
struct BoardSnap {
    a: ZoneSnap,
    b: ZoneSnap,
    digest: u64,
}

fn board_snap(st: &State) -> BoardSnap {
    let mut scratch = st.clone();
    for who in [PlayerId::A, PlayerId::B] {
        let p = scratch.player_mut(who);
        p.rally = 0;
        p.combo = 0;
    }
    BoardSnap {
        a: snap_player(st, PlayerId::A),
        b: snap_player(st, PlayerId::B),
        digest: hash(&scratch),
    }
}

fn changed(before: &BoardSnap, after: &BoardSnap) -> bool {
    before.digest != after.digest
}

fn zone_changed_pair(before: &ZoneSnap, after: &ZoneSnap, zone: Zone) -> bool {
    match zone {
        Zone::Crests => before.crest_ids != after.crest_ids,
        Zone::Field => before.field_cards != after.field_cards,
        Zone::Hand => before.hand_len != after.hand_len,
        Zone::Leader => before.leader_hp != after.leader_hp,
        Zone::Deck => before.deck_len != after.deck_len,
        Zone::Cemetery => before.cemetery_len != after.cemetery_len,
    }
}

fn pool_zone_changed(before: &BoardSnap, after: &BoardSnap, side: Side, zone: Zone) -> [bool; 2] {
    let ally = zone_changed_pair(&before.a, &after.a, zone);
    let enemy = zone_changed_pair(&before.b, &after.b, zone);
    match side {
        Side::Ally => [ally, false],
        Side::Enemy => [false, enemy],
        Side::Any => [ally, enemy],
    }
}

fn had_pool_candidates(snap: &BoardSnap, side: Side, zone: Zone, kind: SelectorKind) -> [bool; 2] {
    let me = &snap.a;
    let opp = &snap.b;
    let had = |z: &ZoneSnap| match zone {
        Zone::Crests => !z.crest_ids.is_empty(),
        Zone::Field => match kind {
            SelectorKind::Amulet => z.amulet_on_field > 0,
            SelectorKind::Follower => z.field_cards.len() > z.amulet_on_field,
            _ => !z.field_cards.is_empty(),
        },
        Zone::Hand => z.hand_len > 0,
        Zone::Leader => true,
        Zone::Deck => z.deck_len > 0,
        Zone::Cemetery => z.cemetery_len > 0,
    };
    match side {
        Side::Ally => [had(me), false],
        Side::Enemy => [false, had(opp)],
        Side::Any => [had(me), had(opp)],
    }
}

fn check_side_coverage(
    case: &ModeCase,
    db: &CardDb,
    before: &BoardSnap,
    after: &BoardSnap,
) -> Option<String> {
    for e in option_effects_for(case, db) {
        if let Effect::Damage { select, .. }
        | Effect::Restore { select, .. }
        | Effect::Buff { select, .. }
        | Effect::Destroy { select, .. }
        | Effect::Banish { select, .. }
        | Effect::RemoveCrests { select, .. } = &e
        {
            if let Selector::Pool(p) = select {
                // Random/Dsplit ops may legitimately skip a side; side coverage is
                // only enforced for `all`/deterministic pool picks.
                if matches!(p.pick, PoolPick::Random) {
                    continue;
                }
                let had = had_pool_candidates(before, p.side, p.zone, p.kind);
                let chg = pool_zone_changed(before, after, p.side, p.zone);
                for (i, (&h, &c)) in had.iter().zip(chg.iter()).enumerate() {
                    if h && !c {
                        return Some(format!(
                            "side coverage: {} opt {} side {:?} {} had candidates but zone {:?} unchanged",
                            case.card_key,
                            case.option_ord,
                            p.side,
                            if i == 0 { "ally" } else { "enemy" },
                            p.zone
                        ));
                    }
                }
            }
        }
    }
    None
}

fn build_rich_board(db: &CardDb, st: &mut State, me: PlayerId, case: &ModeCase) {
    let opp = me.opponent();
    st.player_mut(me).leader_defense = 15;
    st.player_mut(me).leader_max = 20;
    st.player_mut(opp).leader_defense = 15;
    st.player_mut(opp).leader_max = 20;
    give_pp(st, me, 10, 10);
    st.player_mut(me).ep = 3;
    st.player_mut(me).sep = 3;
    put_field(db, st, me, VANILLA);
    let van2 = put_field(db, st, me, VANILLA);
    if let Some(f) = st.field_inst_mut(me, van2) {
        f.defense = 3;
        f.evolved = true;
    }
    put_field(db, st, opp, TANK);
    put_field(db, st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 5;
    }
    put_field(db, st, me, CLUE);
    put_field(db, st, opp, CLUE);
    put_faith_crest(st, me, FAITH, 2);
    put_crest_cd(st, me, CREST_B, 2);
    put_faith_crest(st, opp, FAITH, 3);
    put_crest_cd(st, opp, CREST_A, 1);
    put_hand(db, st, me, SPELL_HAND);
    put_hand(db, st, me, VANILLA);
    put_hand(db, st, opp, VANILLA);
    put_deck(db, st, me, VANILLA);
    put_deck(db, st, opp, VANILLA);
    {
        let id1 = st.alloc_id();
        let card = db.card(cid(VANILLA)).unwrap();
        st.player_mut(me)
            .cemetery
            .push(arena_engine::CardInstance::from_card(card, id1));
    }
    {
        let id2 = st.alloc_id();
        let card = db.card(cid(VANILLA)).unwrap();
        st.player_mut(opp)
            .cemetery
            .push(arena_engine::CardInstance::from_card(card, id2));
    }
    // Slice of Domesticity option 0 needs <2 allied field cards
    if case.card_key == "10823310" && case.choose_ord == 0 {
        clear_field(st, me);
        clear_field(st, opp);
        put_field(db, st, opp, TANK);
    }
    if case.card_key == "10823310" && case.choose_ord == 1 {
        put_field(db, st, me, VANILLA);
        put_field(db, st, me, TANK);
    }
    if case.card_key == "10604110" {
        st.player_mut(me).pp = 0;
    }
}

fn clear_field(st: &mut State, who: PlayerId) {
    for slot in st.player_mut(who).field.iter_mut() {
        *slot = None;
    }
}

fn at_mode_choice(st: &State) -> bool {
    matches!(
        st.phase,
        Phase::Choice {
            node: ChoiceNode::Modes { .. },
            ..
        }
    )
}

fn mode_pick_index(st: &State, want_option: u8) -> Option<u8> {
    let Phase::Choice {
        node: ChoiceNode::Modes { options, .. },
        ..
    } = &st.phase
    else {
        return None;
    };
    options
        .iter()
        .position(|&o| o == want_option)
        .map(|i| i as u8)
}

fn follower_slot(st: &State, me: PlayerId, id: &str) -> u8 {
    st.player(me)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, c)| {
            c.as_ref()
                .filter(|c| c.card.as_str() == id)
                .map(|_| i as u8)
        })
        .expect("follower on field")
}

fn evolve_slot(
    db: &CardDb,
    st: &mut State,
    me: PlayerId,
    slot: u8,
    super_evolve: bool,
) -> Result<(), String> {
    set_round(st, me, 8);
    st.player_mut(me).ep = 3;
    st.player_mut(me).sep = 3;
    apply(
        db,
        st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve,
        },
    )
    .map(|_| ())
    .map_err(|e| format!("evolve slot {slot} super={super_evolve}: {e:?}"))
}

fn fire_trigger(db: &CardDb, st: &mut State, me: PlayerId, case: &ModeCase) -> Result<(), String> {
    let id = case.card_key.as_str();
    let opp = me.opponent();
    match case.trigger {
        HarnessTrigger::Play => {
            if case.card_key.starts_with("crest:") {
                return Err("crest is not played from hand".into());
            }
            let card = db.card(case.card_id).map_err(|e| e.to_string())?;
            let cost = card.cost().max(0);
            clear_hand(st, me);
            if case.card_key == "10804110" {
                clear_field(st, me);
                clear_field(st, opp);
                put_field(db, st, opp, TANK);
                put_field(db, st, opp, TANK);
                put_field(db, st, me, VANILLA);
            } else {
                clear_field(st, me);
            }
            // Pay base cost only so Enhance does not replace fanfare (Golden Knight 10423110).
            give_pp(st, me, cost, cost.max(10));
            play_id(db, st, me, id);
            if !at_mode_choice(st) {
                // replay path: may need only the Nth choose — for multi-choose cards,
                // complete earlier chooses with option 0
                let mut guard = 0;
                while !at_mode_choice(st) && guard < 8 {
                    if matches!(st.phase, Phase::Choice { .. }) {
                        choose(db, st, 0);
                        drain_choice(db, st);
                    } else {
                        break;
                    }
                    guard += 1;
                }
            }
        }
        HarnessTrigger::EvolveField => {
            clear_hand(st, me);
            clear_field(st, me);
            let slot = put_field(db, st, me, id);
            evolve_slot(db, st, me, slot, false)?;
        }
        HarnessTrigger::SuperEvolveField => {
            clear_hand(st, me);
            clear_field(st, me);
            let slot = put_field(db, st, me, id);
            evolve_slot(db, st, me, slot, true)?;
        }
        HarnessTrigger::Engage => {
            clear_hand(st, me);
            let slot = put_field(db, st, me, id);
            apply(
                db,
                st,
                Action::Engage {
                    slot: arena_engine::Slot(slot),
                },
            )
            .map_err(|e| e.to_string())?;
        }
        HarnessTrigger::CardStartOfTurn => {
            clear_hand(st, me);
            clear_field(st, me);
            let slot = put_field(db, st, me, id);
            evolve_slot(db, st, me, slot, false)?;
            end_turn(db, st);
            end_turn(db, st);
        }
        HarnessTrigger::CrestStartOfTurn => {
            put_crest(st, me, &case.card_key);
            end_turn(db, st);
            end_turn(db, st);
        }
    }
    Ok(())
}

fn drain_after_mode(db: &CardDb, st: &mut State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
    drain_choice(db, st);
}

fn run_one_case(db: &CardDb, case: &ModeCase, seed: u64) -> Result<(), String> {
    let key = (case.card_key.clone(), case.choose_ord, case.option_ord);
    if excluded(&key).is_some() {
        return Ok(());
    }
    let mut st = started(db, seed);
    let me = PlayerId::A;
    build_rich_board(db, &mut st, me, case);
    fire_trigger(db, &mut st, me, case)?;
    if !at_mode_choice(&st) {
        return Err(format!(
            "{} choose {} opt {}: harness could not reach mode choice (phase {:?})",
            case.card_key, case.choose_ord, case.option_ord, st.phase
        ));
    }
    let pick = mode_pick_index(&st, case.option_ord).ok_or_else(|| {
        format!(
            "{} option {} not in mode list {:?}",
            case.card_key, case.option_ord, st.phase
        )
    })?;
    let before = board_snap(&st);
    choose(db, &mut st, pick);
    drain_after_mode(db, &mut st);
    let after = board_snap(&st);
    if !changed(&before, &after) {
        return Err(format!(
            "NO-OP {} choose {} option {} ({})",
            case.card_key, case.choose_ord, case.option_ord, case.option_printed
        ));
    }
    if let Some(msg) = check_side_coverage(case, db, &before, &after) {
        return Err(msg);
    }
    Ok(())
}

const RANDOM_SEED_SEARCH: u64 = 4096;

struct RandomExclusion {
    card: &'static str,
    choose: u8,
    option: u8,
    reason: &'static str,
}

/// Options no seed in `0..RANDOM_SEED_SEARCH` could activate (quoted reason).
const RANDOM_EXCLUSIONS: &[RandomExclusion] = &[
    RandomExclusion {
        card: "10574110",
        choose: 0,
        option: 0,
        reason: "follower startOfTurn randomUnused — covered on crest:10574110 (same option text)",
    },
    RandomExclusion {
        card: "10574110",
        choose: 0,
        option: 1,
        reason: "follower startOfTurn randomUnused — crest harness",
    },
    RandomExclusion {
        card: "10574110",
        choose: 0,
        option: 2,
        reason: "follower startOfTurn randomUnused — crest harness",
    },
];

fn random_excluded(key: &CaseKey) -> Option<&'static str> {
    RANDOM_EXCLUSIONS
        .iter()
        .find(|e| e.card == key.0 && e.choose == key.1 && e.option == key.2)
        .map(|e| e.reason)
}

fn enumerate_random_cases(db: &CardDb) -> Vec<ModeCase> {
    let all = enumerate_cases(db);
    let mut out = Vec::new();
    for card in db.cards.values() {
        if card.token() {
            continue;
        }
        let key = card.id().as_str().to_string();
        for ab in card.abilities() {
            let trig = trigger_of(ab);
            let mut raw = Vec::new();
            let mut ord = 0u8;
            collect_chooses_in_effects(ab.effects(), &mut raw, &mut ord);
            for (co, oi, by, printed) in raw {
                if matches!(by, ChooseBy::Random | ChooseBy::RandomUnused) {
                    out.push(ModeCase {
                        card_key: key.clone(),
                        card_id: card.id(),
                        trigger: trig,
                        choose_ord: co,
                        option_ord: oi,
                        by,
                        option_printed: printed,
                    });
                }
            }
        }
    }
    for crest in db.crests.values() {
        let key = crest.id.clone();
        for ab in crest.abilities() {
            let mut raw = Vec::new();
            let mut ord = 0u8;
            collect_chooses_in_effects(ab.effects(), &mut raw, &mut ord);
            for (co, oi, by, printed) in raw {
                if matches!(by, ChooseBy::Random | ChooseBy::RandomUnused) {
                    out.push(ModeCase {
                        card_key: key.clone(),
                        card_id: CardId::parse("88001110").expect("pad"),
                        trigger: HarnessTrigger::CrestStartOfTurn,
                        choose_ord: co,
                        option_ord: oi,
                        by,
                        option_printed: printed,
                    });
                }
            }
        }
    }
    let _ = all;
    out.sort_by(|a, b| {
        (&a.card_key, a.choose_ord, a.option_ord).cmp(&(&b.card_key, b.choose_ord, b.option_ord))
    });
    out
}

#[allow(clippy::collapsible_match, clippy::collapsible_if)]
fn effect_visible_in_snap(before: &BoardSnap, after: &BoardSnap, e: &Effect) -> bool {
    match e {
        Effect::Damage { select, .. } => {
            if let Selector::Pool(p) = select {
                match (p.side, p.zone) {
                    (Side::Enemy, Zone::Leader) => {
                        if after.b.leader_hp < before.b.leader_hp {
                            return true;
                        }
                    }
                    (Side::Enemy, Zone::Field) => {
                        if after.b.field_cards.len() < before.b.field_cards.len() {
                            return true;
                        }
                    }
                    (Side::Ally, Zone::Leader) => {
                        if after.a.leader_hp < before.a.leader_hp {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        Effect::Destroy { select, .. } | Effect::Banish { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.side == Side::Enemy && p.zone == Zone::Field {
                    if after.b.field_cards.len() < before.b.field_cards.len() {
                        return true;
                    }
                }
            }
        }
        Effect::Restore { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.side == Side::Ally
                    && p.zone == Zone::Leader
                    && after.a.leader_hp > before.a.leader_hp
                {
                    return true;
                }
            }
        }
        Effect::Pp { .. } => {
            if after.a.pp > before.a.pp {
                return true;
            }
        }
        Effect::Buff { .. } | Effect::GrantTraits { .. } => {
            if after.a.field_cards != before.a.field_cards {
                return true;
            }
        }
        Effect::Cost { .. } => {
            if after.digest != before.digest {
                return true;
            }
        }
        _ => {}
    }
    false
}

fn option_visible(before: &BoardSnap, after: &BoardSnap, case: &ModeCase, db: &CardDb) -> bool {
    option_effects_for(case, db)
        .iter()
        .any(|e| effect_visible_in_snap(before, after, e))
}

fn choose_used_after_random(
    st: &State,
    me: PlayerId,
    case: &ModeCase,
) -> std::collections::BTreeSet<u8> {
    if case.card_key.starts_with("crest:") {
        return st
            .player(me)
            .crests
            .iter()
            .find(|c| c.id == case.card_key)
            .map(|c| c.choose_used.clone())
            .unwrap_or_default();
    }
    for slot in st.player(me).field.iter().flatten() {
        if slot.card == case.card_id {
            return slot.choose_used.clone();
        }
    }
    for c in st.player(me).hand.iter() {
        if c.card == case.card_id {
            return c.choose_used.clone();
        }
    }
    std::collections::BTreeSet::new()
}

fn discover_random_seed(db: &CardDb, case: &ModeCase, option: u8) -> Option<u64> {
    for seed in 0..RANDOM_SEED_SEARCH {
        let mut st = started(db, seed);
        let me = PlayerId::A;
        build_rich_board(db, &mut st, me, case);
        let before = board_snap(&st);
        if fire_trigger(db, &mut st, me, case).is_err() {
            continue;
        }
        while matches!(st.phase, Phase::Choice { .. }) {
            choose(db, &mut st, 0);
        }
        drain_choice(db, &mut st);
        let after = board_snap(&st);
        if before.digest == after.digest {
            continue;
        }
        let probe = ModeCase {
            option_ord: option,
            ..case.clone()
        };
        let hit = if matches!(case.by, ChooseBy::RandomUnused) {
            choose_used_after_random(&st, me, case).contains(&option)
                && option_visible(&before, &after, &probe, db)
        } else {
            option_visible(&before, &after, &probe, db)
        };
        if hit {
            return Some(seed);
        }
    }
    None
}

fn run_random_option_case(db: &CardDb, case: &ModeCase, seed: u64) -> Result<(), String> {
    let key = (case.card_key.clone(), case.choose_ord, case.option_ord);
    if random_excluded(&key).is_some() {
        return Ok(());
    }
    let mut st = started(db, seed);
    let me = PlayerId::A;
    build_rich_board(db, &mut st, me, case);
    let before = board_snap(&st);
    fire_trigger(db, &mut st, me, case)?;
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, &mut st, 0);
    }
    drain_choice(db, &mut st);
    let after = board_snap(&st);
    if !changed(&before, &after) {
        return Err(format!(
            "NO-OP random {} opt {} ({}) seed {seed}",
            case.card_key, case.option_ord, case.option_printed
        ));
    }
    let probe = ModeCase {
        option_ord: case.option_ord,
        ..case.clone()
    };
    if !option_visible(&before, &after, &probe, db) {
        let used = choose_used_after_random(&st, me, case);
        return Err(format!(
            "{} opt {} seed {seed}: option not visible in snapshot (used {:?})",
            case.card_key, case.option_ord, used
        ));
    }
    if matches!(case.by, ChooseBy::RandomUnused)
        && !choose_used_after_random(&st, me, case).contains(&case.option_ord)
    {
        return Err(format!(
            "{} opt {}: seed {seed} did not mark choose_used",
            case.card_key, case.option_ord
        ));
    }
    if let Some(msg) = check_side_coverage(case, db, &before, &after) {
        return Err(msg);
    }
    Ok(())
}

fn play_spell_with_pp(db: &CardDb, st: &mut State, me: PlayerId, id: &str, pp: i32) {
    clear_hand(st, me);
    give_pp(st, me, pp, pp.max(10));
    play_id(db, st, me, id);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
    drain_choice(db, st);
}

#[test]
fn mode_sweep_random_cards() {
    let db = load_db();
    let cases = enumerate_random_cases(&db);
    assert!(
        cases
            .iter()
            .any(|c| c.card_key == "10604110" && c.option_ord == 0),
        "10604110 random fanfare options"
    );
    let mut failures = Vec::new();
    let mut seeds: Vec<String> = Vec::new();
    for case in &cases {
        let key = (case.card_key.clone(), case.choose_ord, case.option_ord);
        if random_excluded(&key).is_some() {
            continue;
        }
        let seed = discover_random_seed(&db, case, case.option_ord);
        let Some(seed) = seed else {
            failures.push(format!(
                "{} choose {} opt {}: no seed in 0..{} (not listed in RANDOM_EXCLUSIONS)",
                case.card_key, case.choose_ord, case.option_ord, RANDOM_SEED_SEARCH
            ));
            continue;
        };
        seeds.push(format!(
            "{} choose {} opt {} => seed {}",
            case.card_key, case.choose_ord, case.option_ord, seed
        ));
        if let Err(e) = run_random_option_case(&db, case, seed) {
            failures.push(e);
        }
    }
    assert!(
        failures.is_empty(),
        "random mode sweep ({}):\n{}\n\nseeds:\n{}",
        failures.len(),
        failures.join("\n"),
        seeds.join("\n")
    );
}

#[test]
fn mode_sweep_autopick_branches() {
    let db = load_db();
    let me = PlayerId::A;

    // 10912310 — when: `combo: {n: 3}` pick-all (optionsFrom fanfare)
    {
        let mut st = started(&db, 77_100);
        build_rich_board(
            &db,
            &mut st,
            me,
            &ModeCase {
                card_key: "10912310".into(),
                card_id: cid("10912310"),
                trigger: HarnessTrigger::Play,
                choose_ord: 0,
                option_ord: 0,
                by: ChooseBy::Player,
                option_printed: String::new(),
            },
        );
        st.player_mut(me).combo = 3;
        let before = board_snap(&st);
        play_spell_with_pp(&db, &mut st, me, "10912310", 2);
        assert!(
            !at_mode_choice(&st),
            "10912310 combo≥3 should auto-activate all modes"
        );
        assert_ne!(
            before.digest,
            board_snap(&st).digest,
            "10912310 pick-all NO-OP"
        );
    }

    // 10823310 — when: countAtLeast ≥2 allied field cards
    {
        let mut st = started(&db, 77_101);
        let case = ModeCase {
            card_key: "10823310".into(),
            card_id: cid("10823310"),
            trigger: HarnessTrigger::Play,
            choose_ord: 1,
            option_ord: 0,
            by: ChooseBy::Player,
            option_printed: String::new(),
        };
        build_rich_board(&db, &mut st, me, &case);
        let before = board_snap(&st);
        play_spell_with_pp(&db, &mut st, me, "10823310", 2);
        assert!(!at_mode_choice(&st), "10823310 field≥2 pick-all");
        assert_ne!(
            before.digest,
            board_snap(&st).digest,
            "10823310 pick-all NO-OP"
        );
    }

    // 10623310 — Enhance (3) replaces fanfare with pick-all
    {
        let mut st = started(&db, 77_102);
        build_rich_board(
            &db,
            &mut st,
            me,
            &ModeCase {
                card_key: "10623310".into(),
                card_id: cid("10623310"),
                trigger: HarnessTrigger::Play,
                choose_ord: 0,
                option_ord: 0,
                by: ChooseBy::Player,
                option_printed: String::new(),
            },
        );
        let before = board_snap(&st);
        play_spell_with_pp(&db, &mut st, me, "10623310", 3);
        assert!(!at_mode_choice(&st), "10623310 Enhance(3) pick-all");
        assert_ne!(
            before.digest,
            board_snap(&st).digest,
            "10623310 enhance NO-OP"
        );
    }

    // 10423110 — Enhance (8) pick-all fanfare modes
    {
        let mut st = started(&db, 77_103);
        clear_field(&mut st, me);
        give_pp(&mut st, me, 8, 10);
        st.player_mut(me).sep = 3;
        let before = board_snap(&st);
        play_id(&db, &mut st, me, "10423110");
        while matches!(st.phase, Phase::Choice { .. }) {
            choose(&db, &mut st, 0);
        }
        drain_choice(&db, &mut st);
        assert!(!at_mode_choice(&st), "10423110 Enhance(8) pick-all");
        assert_ne!(
            before.digest,
            board_snap(&st).digest,
            "10423110 enhance NO-OP"
        );
    }

    // 10413310 — Super Skybound Art gauge ≥ 15 (`skyboundArt: {n: 15}`)
    {
        let mut st = started(&db, 77_104);
        set_round(&mut st, me, 15);
        build_rich_board(
            &db,
            &mut st,
            me,
            &ModeCase {
                card_key: "10413310".into(),
                card_id: cid("10413310"),
                trigger: HarnessTrigger::Play,
                choose_ord: 0,
                option_ord: 0,
                by: ChooseBy::Player,
                option_printed: String::new(),
            },
        );
        let before = board_snap(&st);
        play_spell_with_pp(&db, &mut st, me, "10413310", 2);
        assert!(!at_mode_choice(&st), "10413310 super skybound pick-all");
        assert_ne!(
            before.digest,
            board_snap(&st).digest,
            "10413310 skybound NO-OP"
        );
    }

    // 10962310 — allied field card with baseCost ≥ 6
    {
        let mut st = started(&db, 77_105);
        build_rich_board(
            &db,
            &mut st,
            me,
            &ModeCase {
                card_key: "10962310".into(),
                card_id: cid("10962310"),
                trigger: HarnessTrigger::Play,
                choose_ord: 0,
                option_ord: 0,
                by: ChooseBy::Player,
                option_printed: String::new(),
            },
        );
        put_field(&db, &mut st, me, "10604110");
        let before = board_snap(&st);
        play_spell_with_pp(&db, &mut st, me, "10962310", 1);
        assert!(!at_mode_choice(&st), "10962310 fieldHas cost≥6 pick-all");
        assert_ne!(
            before.digest,
            board_snap(&st).digest,
            "10962310 pick-all NO-OP"
        );
    }
}

#[test]
fn mode_sweep_all_player_options() {
    let db = load_db();
    let cases = enumerate_cases(&db);
    let mut failures = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let seed = 90_800_000 + i as u64;
        if let Err(e) = run_one_case(&db, case, seed) {
            failures.push(e);
        }
    }
    assert!(
        failures.is_empty(),
        "mode sweep failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn mode_sweep_scope_counts() {
    let db = load_db();
    let cases = enumerate_cases(&db);
    let cards: std::collections::BTreeSet<_> = cases.iter().map(|c| c.card_key.clone()).collect();
    // d9f6e18 pool: 44 cards hold a `choose`, 111 options total; 96 are `by: player`.
    assert_eq!(
        cards.len(),
        36,
        "unique player-pick mode cards (excl. fixture tokens)"
    );
    assert_eq!(
        cases.len(),
        86,
        "player-picked mode options (excl. fixture tokens)"
    );
}
