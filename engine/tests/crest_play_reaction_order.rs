//! Crest `ally_card_played` reactions on follower / amulet plays resolve after Fanfare.

use arena_engine::event::Event;
use arena_engine::state::CrestInstance;
use arena_engine::{apply, hash, Action, CardDb, Illegal, Phase, PlayForm, PlayerId, State};

mod common;
use common::*;

const YUEL_CREST: &str = "crest:10414120";
const CUTTHROAT_CREST: &str = "crest:10974110";
const MAJESTIC_CREST: &str = "crest:10622310";
const THESTAE: &str = "10714110";
const ASHER: &str = "10874110";
const AIZEDEN: &str = "10974120";
const TANK: &str = "89500001";
const WARDEN: &str = "90074150";
const AIZEDEN_ENEMY: &str = "10974120";
const TIA: &str = "10814120";
const FATE_ENHANCED: &str = "10503310";
const FEARLESS: &str = "10621110";

fn put_crest_countdown(st: &mut State, who: PlayerId, id: &str, cd: i32) {
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

fn replay_capture_to_ply(
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

    let seed = cap["seed"]
        .as_u64()
        .or_else(|| cap["seed"].as_str().and_then(|s| s.parse().ok()))
        .expect("seed");
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

#[test]
fn thestae_yuel_crest_x_is_unevolved_attack() {
    let db = load_db();
    let mut st = started(&db, 701);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_crest_countdown(&mut st, me, YUEL_CREST, 4);
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 10, 10);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, THESTAE);
    drain_choice(&db, &mut st);
    assert_eq!(field_def(&st, opp, TANK), Some(7));
    let thestae = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == THESTAE)
        .expect("thestae");
    assert!(thestae.evolved);
    assert_eq!(thestae.attack, 5);
    assert_eq!(thestae.defense, 5);
    assert_eq!(st.player(me).combo, 2, "play +1 and Fanfare +1");
}

#[test]
fn thestae_cutthroat_crest_x_is_unevolved_attack() {
    let db = load_db();
    let mut st = started(&db, 702);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_crest(&mut st, me, CUTTHROAT_CREST);
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 10, 10);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, THESTAE);
    drain_choice(&db, &mut st);
    assert_eq!(field_def(&st, opp, TANK), Some(7));
}

#[test]
fn asher_cutthroat_crest_destroys_the_fanfare_ward() {
    let db = load_db();
    let mut st = started(&db, 703);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_crest(&mut st, me, CUTTHROAT_CREST);
    put_field(&db, &mut st, opp, AIZEDEN_ENEMY);
    put_field(&db, &mut st, opp, WARDEN);
    st.player_mut(opp).leader_defense = 18;
    st.player_mut(opp).leader_max = 20;
    give_pp(&mut st, me, 7, 7);
    clear_hand(&mut st, me);
    let hand = put_hand(&db, &mut st, me, ASHER);
    let mut all_events = apply(&db, &mut st, Action::Play { hand }).expect("play asher");
    let play = all_events
        .iter()
        .find_map(|e| match e {
            Event::Play { form, .. } => Some(*form),
            _ => None,
        })
        .expect("play event");
    assert!(matches!(play, PlayForm::Normal));
    assert_eq!(st.player(me).pp, 2);
    all_events.extend(apply(&db, &mut st, Action::Choose(0)).expect("fanfare pick"));
    while matches!(st.phase, Phase::Choice { .. }) {
        all_events.extend(apply(&db, &mut st, Action::Choose(0)).expect("drain choice"));
    }
    assert_eq!(field_count(&st, opp), 0);
    let destroys = all_events
        .iter()
        .filter(|e| matches!(e, Event::Destroy { .. }))
        .count();
    assert_eq!(destroys, 2);
    assert_eq!(st.player(opp).leader_defense, 20);
    let asher = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ASHER)
        .expect("asher");
    assert!(asher.evolved);
    assert_eq!(asher.attack, 7);
    assert_eq!(asher.defense, 7);
    assert_ne!(asher.traits.storm, Some(true));
}

#[test]
fn owner_capture_asher_game() {
    // Origin: captures/2026-10/6750670994633786279-d96c4c62-lr4npoqps2.json @ engine b110fd1
    let db = load_db();
    let path = fixtures_dir().join("captures/6750670994633786279-d96c4c62-lr4npoqps2.json");
    let cap: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("capture")).expect("json");
    let st = replay_capture_to_ply(&db, &cap, 61).expect("replay through Fanfare pick");
    assert_eq!(field_count(&st, PlayerId::B), 0);
    assert_eq!(st.player(PlayerId::B).leader_defense, 20);
}

#[test]
fn aizeden_cutthroat_crest_evolves_the_played_card() {
    let db = load_db();
    let mut st = started(&db, 704);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_crest(&mut st, me, CUTTHROAT_CREST);
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 7, 7);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, AIZEDEN);
    drain_choice(&db, &mut st);
    let aizeden = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == AIZEDEN)
        .expect("aizeden");
    assert!(aizeden.evolved);
    let warden = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WARDEN)
        .expect("warden");
    assert!(!warden.evolved);
    assert!(!field_has(&st, opp, TANK));
}

#[test]
fn majestic_conquest_soldier_after_enhance_body() {
    // Tia Enhance (4) buffs all allied followers; if the crest summoned Fearless
    // Soldier before the body, the token is 2/2; after the body it stays 1/1.
    let db = load_db();
    let mut st = started(&db, 705);
    let me = PlayerId::A;
    put_crest(&mut st, me, MAJESTIC_CREST);
    give_pp(&mut st, me, 4, 4);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, TIA);
    drain_choice(&db, &mut st);
    let soldier = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == FEARLESS)
        .expect("soldier");
    assert_eq!(
        soldier.attack, 2,
        "Enhance body before crest: Soldier not buffed"
    );
    assert_eq!(soldier.defense, 2);
}

#[test]
fn majestic_crest_soldier_before_enhanced_spell_body() {
    let db = load_db();
    let mut st = started(&db, 706);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_crest(&mut st, me, MAJESTIC_CREST);
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 10, 10);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, FATE_ENHANCED);
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, opp, TANK),
        "spell play: crest soldier before Enhance body; 4 damage removed the enemy first"
    );
    assert!(
        field_has(&st, me, FEARLESS),
        "Fearless Soldier (2/2) survives after the spell — summoned before the damage"
    );
}

struct RecordedReplayRow {
    path: &'static str,
    ply: usize,
    want_hash: u64,
}

#[test]
fn as_recorded_crest_order_replays_like_main() {
    use serde_json::Value;
    use std::fs;

    let rows = [
        RecordedReplayRow {
            path: "tkroll/fa-play-150-ply0086.json",
            ply: 86,
            want_hash: 0xeeda041e005d64d8,
        },
        RecordedReplayRow {
            path: "tkroll/fb-play-122-ply0064.json",
            ply: 64,
            want_hash: 0x9bdc48e2b98eaade,
        },
    ];

    for row in &rows {
        let path = fixtures_dir().join(row.path);
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let with_overlay = with_crest_play_reactions_as_recorded(load_recorded_db());
        let st = replay_capture_to_ply(&with_overlay, &cap, row.ply).expect("overlay replay");
        assert_eq!(hash(&st), row.want_hash, "{}", row.path);

        let mut recorded_cards = load_db();
        recorded_cards
            .load_replace_dir(repo_root().join("oracle/cards-as-recorded"))
            .expect("recorded");
        match replay_capture_to_ply(&recorded_cards, &cap, row.ply) {
            Ok(st) => assert_ne!(
                hash(&st),
                row.want_hash,
                "{}: new crest order changes final hash",
                row.path
            ),
            Err((step, _)) => assert!(
                step <= row.ply,
                "{}: live replay breaks by action {}",
                row.path,
                step
            ),
        }
    }
}

// Unchanged regressions (see PR body): `m2_wave2::world_of_games_divine_thunder_qa_count_is_four`,
// `m2_wave2::world_of_games_advances_on_same_base_cost`, `play_time_selection::artisan_damage_kills_spell_target_after_play_time_pick`,
// `play_time_selection::majestic_crest_summon_not_in_enhanced_play_pick` (spell plays),
// `m2_portal::asher_enhance_granted_ward_is_a_destroy_candidate`, `play_time_selection::yuel_crest_fanfare_pick_before_evolve_pick`.
