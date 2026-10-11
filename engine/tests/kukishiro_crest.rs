//! Kukishiro crest `randomNamed`, Sofina mode-2 bind, burned draws.

use arena_engine::{
    apply, hash, Action, CardDb, GameRng, Illegal, Phase, Pick, PickChose, PickWhat, PlayerId,
};

mod common;
use common::*;

const KUKISHIRO: &str = "10564120";
const CRYSTAL: &str = "10631110";
const FIGHTER: &str = "10001110";
const FOX: &str = "10061120";
const FALCON: &str = "90061110";
const PAD: &str = "88001110";
const SOFINA: &str = "10564110";
const COWARD: &str = "10661110";
const SHRINEMOUSE: &str = "10562120";
const DRAGONSLAYER: &str = "10041120";
const STAFFMASTER: &str = "10522110";
const REMI_RAMI: &str = "10032110";
const GOLEM: &str = "90031120";
const CREST_ID: &str = "crest:10564120";

fn token_followers_on_field(st: &arena_engine::State, who: PlayerId) -> Vec<String> {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FOX || c.card.as_str() == FALCON)
        .map(|c| c.card.as_str().to_string())
        .collect()
}

fn count_in_deck(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    let id = cid(id);
    st.player(who).deck.iter().filter(|c| c.card == id).count()
}

fn deck_len(st: &arena_engine::State, who: PlayerId) -> usize {
    st.player(who).deck.len()
}

fn has_crest(st: &arena_engine::State, who: PlayerId, id: &str) -> bool {
    st.player(who).crests.iter().any(|c| c.id == id)
}

fn random_card_picks(st: &arena_engine::State) -> Vec<&Pick> {
    st.picks
        .iter()
        .filter(|p| p.what == PickWhat::RandomCard)
        .collect()
}

fn follower_stats(st: &arena_engine::State, who: PlayerId, id: &str) -> Option<(i32, i32, i32)> {
    let id = cid(id);
    st.player(who)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == id)
        .map(|c| (c.attack, c.defense, c.max_defense))
}

fn hand_card_cost(st: &arena_engine::State, who: PlayerId, id: &str) -> Option<i32> {
    let id = cid(id);
    st.player(who)
        .hand
        .iter()
        .find(|c| c.card == id)
        .map(|c| c.cost)
}

fn setup_deck(db: &CardDb, st: &mut arena_engine::State, who: PlayerId, fill: &str) {
    st.player_mut(who).deck.clear();
    for _ in 0..40 {
        put_deck(db, st, who, fill);
    }
}

fn next_turn_start_draw(db: &CardDb, st: &mut arena_engine::State, who: PlayerId) {
    let turns = st.player(who).turns_taken;
    skip_to_player_turn(db, st, who, turns + 1);
    assert_eq!(st.active, who);
}

fn burn_draw_turn_start(db: &CardDb, st: &mut arena_engine::State, who: PlayerId) {
    assert_eq!(
        st.player(who).hand.len(),
        9,
        "hand must be full before burn draw"
    );
    let shadows_before = st.player(who).shadows;
    next_turn_start_draw(db, st, who);
    assert!(
        st.player(who).shadows > shadows_before,
        "expected a burned turn-start draw (hand={})",
        st.player(who).hand.len()
    );
}

#[test]
fn kukishiro_odd_draws_summon_fox_or_falcon() {
    let db = load_db();
    let mut st = started_decks(&db, 101, &[PAD], &[PAD]);
    setup_deck(&db, &mut st, PlayerId::A, CRYSTAL);
    let deck_before = deck_len(&st, PlayerId::A);
    play_fresh(&db, &mut st, PlayerId::A, KUKISHIRO);
    assert!(has_crest(&st, PlayerId::A, CREST_ID));
    assert!(field_has(&st, PlayerId::A, KUKISHIRO));
    let tokens = token_followers_on_field(&st, PlayerId::A);
    assert_eq!(tokens.len(), 2);
    for t in &tokens {
        assert!(t == FOX || t == FALCON);
        let slot = st
            .player(PlayerId::A)
            .field
            .iter()
            .flatten()
            .find(|c| c.card.as_str() == *t)
            .expect("token");
        assert!(slot.flags.summoning_sick);
    }
    let rc = random_card_picks(&st);
    assert_eq!(rc.len(), 2);
    assert!(rc.iter().all(|p| p.among.is_none()));
    let chose: Vec<_> = rc
        .iter()
        .map(|p| match &p.chose {
            PickChose::Id(s) => s.clone(),
            _ => panic!("expected id chose"),
        })
        .collect();
    assert_eq!(chose, tokens);
    assert!(!st.picks.iter().any(|p| p.what == PickWhat::MultisetPick));
    assert_eq!(token_followers_on_field(&st, PlayerId::B).len(), 0);
    assert_eq!(deck_len(&st, PlayerId::A), deck_before - 2);
}

#[test]
fn kukishiro_even_draws_summon_for_the_enemy() {
    let db = load_db();
    let mut st = started_decks(&db, 102, &[PAD], &[PAD]);
    setup_deck(&db, &mut st, PlayerId::A, FIGHTER);
    play_fresh(&db, &mut st, PlayerId::A, KUKISHIRO);
    assert_eq!(token_followers_on_field(&st, PlayerId::A).len(), 0);
    assert_eq!(token_followers_on_field(&st, PlayerId::B).len(), 2);
}

#[test]
fn kukishiro_crest_never_takes_from_the_deck() {
    let db = load_db();
    let mut st = started_decks(&db, 120, &[PAD], &[PAD]);
    st.player_mut(PlayerId::A).deck.clear();
    for _ in 0..20 {
        put_deck(&db, &mut st, PlayerId::A, FOX);
    }
    for _ in 0..20 {
        put_deck(&db, &mut st, PlayerId::A, CRYSTAL);
    }
    let fox_before = count_in_deck(&st, PlayerId::A, FOX);
    play_fresh(&db, &mut st, PlayerId::A, KUKISHIRO);
    assert_eq!(count_in_deck(&st, PlayerId::A, FOX), fox_before - 1);
    assert_eq!(token_followers_on_field(&st, PlayerId::A).len(), 1);
    assert_eq!(token_followers_on_field(&st, PlayerId::B).len(), 1);
    assert!(!st.picks.iter().any(|p| p.what == PickWhat::MultisetPick));
}

#[test]
fn kukishiro_crest_full_field_no_rng() {
    let db = load_db();
    let mut st = started_decks(&db, 104, &[PAD], &[PAD]);
    setup_deck(&db, &mut st, PlayerId::A, CRYSTAL);
    for _ in 0..4 {
        put_field(&db, &mut st, PlayerId::A, PAD);
    }
    let tokens_before = token_followers_on_field(&st, PlayerId::A).len();
    play_fresh(&db, &mut st, PlayerId::A, KUKISHIRO);
    assert_eq!(
        token_followers_on_field(&st, PlayerId::A).len(),
        tokens_before
    );
    assert!(random_card_picks(&st).is_empty());
}

#[test]
fn crest_burned_draw_summons_nothing() {
    let db = load_db();
    let mut st = started_decks(&db, 105, &[PAD], &[PAD]);
    setup_deck(&db, &mut st, PlayerId::A, CRYSTAL);
    put_field(&db, &mut st, PlayerId::A, PAD);
    put_crest(&mut st, PlayerId::A, CREST_ID);
    let field_before = field_count(&st, PlayerId::A);
    for _ in 0..9 {
        put_hand(&db, &mut st, PlayerId::A, CRYSTAL);
    }
    let shadows_before = st.player(PlayerId::A).shadows;
    burn_draw_turn_start(&db, &mut st, PlayerId::A);
    assert_eq!(field_count(&st, PlayerId::A), field_before);
    assert_eq!(st.player(PlayerId::A).shadows, shadows_before + 1);
    assert!(random_card_picks(&st).is_empty());
}

#[test]
fn shrinemouse_burned_draw_deals_nothing() {
    let db = load_db();
    let mut st = started_decks(&db, 106, &[PAD], &[PAD]);
    clear_hand(&mut st, PlayerId::A);
    put_field(&db, &mut st, PlayerId::A, SHRINEMOUSE);
    put_field(&db, &mut st, PlayerId::B, DRAGONSLAYER);
    for _ in 0..9 {
        put_hand(&db, &mut st, PlayerId::A, CRYSTAL);
    }
    setup_deck(&db, &mut st, PlayerId::A, CRYSTAL);
    burn_draw_turn_start(&db, &mut st, PlayerId::A);
    assert_eq!(
        follower_stats(&st, PlayerId::B, DRAGONSLAYER),
        Some((5, 10, 10))
    );

    let mut st2 = started_decks(&db, 106, &[PAD], &[PAD]);
    clear_hand(&mut st2, PlayerId::A);
    put_field(&db, &mut st2, PlayerId::A, SHRINEMOUSE);
    put_field(&db, &mut st2, PlayerId::B, DRAGONSLAYER);
    for _ in 0..8 {
        put_hand(&db, &mut st2, PlayerId::A, CRYSTAL);
    }
    setup_deck(&db, &mut st2, PlayerId::A, CRYSTAL);
    next_turn_start_draw(&db, &mut st2, PlayerId::A);
    assert_eq!(field_atk(&st2, PlayerId::B, DRAGONSLAYER), Some(5));
    assert_eq!(field_def(&st2, PlayerId::B, DRAGONSLAYER), Some(9));
}

#[test]
fn staffmaster_burn_leaves_the_old_copy_alone() {
    let db = load_db();
    let mut st = started_decks(&db, 107, &[PAD], &[PAD]);
    clear_hand(&mut st, PlayerId::A);
    for _ in 0..8 {
        put_hand(&db, &mut st, PlayerId::A, CRYSTAL);
    }
    put_hand(&db, &mut st, PlayerId::A, STAFFMASTER);
    setup_deck(&db, &mut st, PlayerId::A, STAFFMASTER);
    burn_draw_turn_start(&db, &mut st, PlayerId::A);
    assert_eq!(hand_card_cost(&st, PlayerId::A, STAFFMASTER), Some(6));

    let mut st2 = started_decks(&db, 107, &[PAD], &[PAD]);
    clear_hand(&mut st2, PlayerId::A);
    setup_deck(&db, &mut st2, PlayerId::A, STAFFMASTER);
    next_turn_start_draw(&db, &mut st2, PlayerId::A);
    assert_eq!(hand_card_cost(&st2, PlayerId::A, STAFFMASTER), Some(3));
}

#[test]
fn sofina_mode2_evolves_and_gives_plus_one() {
    let db = load_db();
    let mut st = started_decks(&db, 108, &[PAD], &[PAD]);
    put_field(&db, &mut st, PlayerId::A, COWARD);
    clear_hand(&mut st, PlayerId::A);
    give_pp(&mut st, PlayerId::A, 4, 10);
    play_id(&db, &mut st, PlayerId::A, SOFINA);
    choose(&db, &mut st, 1);
    drain_choice(&db, &mut st);
    assert_eq!(follower_stats(&st, PlayerId::A, COWARD), Some((7, 8, 8)));
    assert_eq!(follower_stats(&st, PlayerId::A, SOFINA), Some((2, 2, 2)));
    assert!(st.bindings.is_empty());

    let mut st2 = started_decks(&db, 109, &[PAD], &[PAD]);
    put_field(&db, &mut st2, PlayerId::A, COWARD);
    put_field(&db, &mut st2, PlayerId::A, FOX);
    clear_hand(&mut st2, PlayerId::A);
    give_pp(&mut st2, PlayerId::A, 4, 10);
    play_id(&db, &mut st2, PlayerId::A, SOFINA);
    choose(&db, &mut st2, 1);
    drain_choice(&db, &mut st2);
    assert_eq!(follower_stats(&st2, PlayerId::A, COWARD), Some((7, 8, 8)));
    assert_eq!(follower_stats(&st2, PlayerId::A, FOX), Some((1, 3, 3)));
    let fox_inst = st2
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == FOX)
        .expect("fox");
    assert!(!fox_inst.evolved);
}

#[test]
fn remi_rami_super_evolve_golem_control() {
    let db = load_db();
    let mut st = started_decks(&db, 110, &[PAD], &[PAD]);
    let golem_slot = put_field(&db, &mut st, PlayerId::A, GOLEM);
    let remi_slot = put_field(&db, &mut st, PlayerId::A, REMI_RAMI);
    set_round(&mut st, PlayerId::A, 7);
    st.player_mut(PlayerId::A).sep = 1;
    st.player_mut(PlayerId::A).evolved_this_turn = false;
    give_pp(&mut st, PlayerId::A, 10, 10);
    let _ = golem_slot;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(remi_slot),
            super_evolve: true,
        },
    )
    .expect("super evolve");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert_eq!(follower_stats(&st, PlayerId::A, GOLEM), Some((8, 8, 8)));
}

#[test]
fn crest_summons_replay_scripted() {
    let db = load_db();
    let decks = pad_deck(&[CRYSTAL], 40);
    let mut live = arena_engine::new_game(
        &db,
        arena_engine::GameConfig {
            seed: 111,
            deck_a: decks.clone(),
            deck_b: pad_deck(&[PAD], 40),
            first: arena_engine::First::A,
            opening_hands: None,
        },
    )
    .expect("new");
    apply(&db, &mut live, Action::MulliganConfirm { swap: [false; 4] }).unwrap();
    apply(&db, &mut live, Action::MulliganConfirm { swap: [false; 4] }).unwrap();
    clear_hand(&mut live, PlayerId::A);
    give_pp(&mut live, PlayerId::A, 10, 10);
    play_id(&db, &mut live, PlayerId::A, KUKISHIRO);
    drain_choice(&db, &mut live);
    let want_hash = hash(&live);
    let picks = live.picks.clone();

    let mut replay = arena_engine::new_game(
        &db,
        arena_engine::GameConfig {
            seed: 111,
            deck_a: decks.clone(),
            deck_b: pad_deck(&[PAD], 40),
            first: arena_engine::First::A,
            opening_hands: None,
        },
    )
    .expect("new");
    apply(
        &db,
        &mut replay,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    apply(
        &db,
        &mut replay,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    clear_hand(&mut replay, PlayerId::A);
    give_pp(&mut replay, PlayerId::A, 10, 10);
    let h = put_hand(&db, &mut replay, PlayerId::A, KUKISHIRO);
    replay.rng = GameRng::scripted(picks.clone(), 111);
    apply(&db, &mut replay, Action::Play { hand: h }).unwrap();
    drain_choice(&db, &mut replay);
    assert_eq!(hash(&replay), want_hash);

    let bad: Vec<Pick> = picks
        .iter()
        .map(|p| {
            if p.what == PickWhat::RandomCard {
                Pick {
                    what: PickWhat::RandomCard,
                    among: p.among.clone(),
                    chose: PickChose::Id(PAD.to_string()),
                }
            } else {
                p.clone()
            }
        })
        .collect();
    let mut bad_st = arena_engine::new_game(
        &db,
        arena_engine::GameConfig {
            seed: 111,
            deck_a: decks,
            deck_b: pad_deck(&[PAD], 40),
            first: arena_engine::First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    apply(
        &db,
        &mut bad_st,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    apply(
        &db,
        &mut bad_st,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    clear_hand(&mut bad_st, PlayerId::A);
    give_pp(&mut bad_st, PlayerId::A, 10, 10);
    let h2 = put_hand(&db, &mut bad_st, PlayerId::A, KUKISHIRO);
    bad_st.rng = GameRng::scripted(bad, 111);
    let err = apply(&db, &mut bad_st, Action::Play { hand: h2 }).unwrap_err();
    assert!(matches!(err, Illegal::OraclePickNotLegal(_)));
}

#[test]
fn kukishiro_crest_as_recorded_overlay() {
    use serde_json::Value;
    use std::fs;

    const PATH: &str = "hbcheck/neg-play-100087-ply0065.json";
    const WANT: u64 = 0x55023cb9ecf25a0c;
    const BREAKS_AT: usize = 54;

    let cap: Value =
        serde_json::from_str(&fs::read_to_string(fixtures_dir().join(PATH)).expect("read"))
            .expect("json");

    let st = replay_fixture_to_ply(&load_recorded_db(), &cap, 65).expect("recorded");
    assert_eq!(hash(&st), WANT);

    let mut db = load_db();
    db.load_replace_dir(repo_root().join("oracle/cards-as-recorded"))
        .expect("recorded cards");
    db = with_earrings_engage_as_recorded(db);
    let (at, _) = replay_fixture_to_ply(&db, &cap, 65).expect_err("no crest overlay");
    assert_eq!(at, BREAKS_AT);
}

fn replay_fixture_to_ply(
    db: &CardDb,
    cap: &serde_json::Value,
    n: usize,
) -> Result<arena_engine::State, (usize, Illegal)> {
    use arena_engine::{apply_neutral, new_game, First, GameConfig, NeutralAction};
    use serde_json::Value;
    use std::collections::BTreeMap;

    fn deck_from_json(v: &Value) -> Vec<arena_engine::CardId> {
        let map = v.as_object().expect("deck object");
        let mut sorted = BTreeMap::new();
        for (id, n) in map {
            sorted.insert(id.clone(), n.as_u64().unwrap_or(0) as usize);
        }
        let mut out = Vec::new();
        for (id, count) in sorted {
            let cid = arena_engine::CardId::parse(&id).unwrap_or_else(|| panic!("bad id {id}"));
            out.extend(std::iter::repeat_n(cid, count));
        }
        out
    }

    let seed = cap["seed"].as_u64().expect("seed");
    let deck_a = deck_from_json(&cap["deckA"]);
    let deck_b = deck_from_json(&cap["deckB"]);
    let first = match cap.get("first").and_then(|v| v.as_str()) {
        Some("b") | Some("B") => First::B,
        Some("a") | Some("A") => First::A,
        _ => First::Coin,
    };
    let mut st = new_game(
        db,
        GameConfig {
            seed,
            deck_a,
            deck_b,
            first,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let actions = cap["actions"].as_array().expect("actions");
    for (i, step) in actions.iter().take(n).enumerate() {
        if step.get("reseed").is_some() {
            st.reseed(step["reseed"].as_u64().expect("reseed"));
            continue;
        }
        let mut body = serde_json::Map::new();
        for (k, v) in step.as_object().expect("action object") {
            if k == "value" || k == "bot_value" {
                continue;
            }
            body.insert(k.clone(), v.clone());
        }
        let neu: NeutralAction = serde_json::from_value(Value::Object(body)).expect("action");
        if let Err(e) = apply_neutral(db, &mut st, &neu) {
            return Err((i, e));
        }
    }
    Ok(st)
}
