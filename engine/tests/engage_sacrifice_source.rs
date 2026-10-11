//! Sacrifice Engage destroys its amulet before the rest of the text resolves;
//! `source_card_id` must still find that card (Earrings of Sunlight, Gems labels).

use arena_engine::event::Event;
use arena_engine::{
    apply, hash, legal_actions, Action, CardDb, Illegal, Phase, PlayerId, Slot, TargetOpt,
};

mod common;
use common::*;

const EARRINGS: &str = "10761210";
const HAND_CARD: &str = "88001320";
const DECK_TOP: &str = "88001110";
const OMERIO: &str = "10964120";
const TANK: &str = "89500001";

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

fn choice_target_options(st: &arena_engine::State) -> Vec<TargetOpt> {
    let Phase::Choice {
        node: arena_engine::ChoiceNode::Targets { options, .. },
        ..
    } = &st.phase
    else {
        panic!("expected Targets choice");
    };
    options.clone()
}

#[test]
fn earrings_engage_replicates_fanfare() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let earrings_slot = put_field(&db, &mut st, me, EARRINGS);
    let hand_inst = {
        clear_hand(&mut st, me);
        put_hand(&db, &mut st, me, HAND_CARD);
        st.player(me).hand[0].id
    };
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, DECK_TOP);
    let deck_len = st.player(me).deck.len();
    let shadows_before = st.player(me).shadows;

    apply(
        &db,
        &mut st,
        Action::Engage {
            slot: Slot(earrings_slot),
        },
    )
    .expect("engage");

    assert!(matches!(st.phase, Phase::Choice { .. }));
    assert_eq!(pending_kind(&st), arena_engine::PendingKind::EffectSelect);
    assert_eq!(
        choice_target_options(&st),
        vec![TargetOpt::Hand { player: me, pos: 0 }]
    );
    assert_eq!(legal_actions(&db, &st), vec![Action::Choose(0)]);

    let events = apply(&db, &mut st, Action::Choose(0)).expect("pick hand card");
    let draws = events
        .iter()
        .filter(|e| matches!(e, Event::Draw { player, .. } if *player == me))
        .count();
    assert_eq!(draws, 1);

    assert!(!field_has(&st, me, EARRINGS));
    assert!(st
        .player(me)
        .cemetery
        .iter()
        .any(|c| c.card == cid(EARRINGS)));
    assert_eq!(st.player(me).deck.len(), deck_len);
    assert!(st
        .player(me)
        .deck
        .iter()
        .any(|c| c.id == hand_inst && c.card == cid(HAND_CARD)));
    assert_eq!(st.player(me).hand.len(), 1);
    assert_eq!(st.player(me).hand[0].card, cid(DECK_TOP));
    let last = st
        .player(me)
        .destroyed_history
        .last()
        .expect("destroy record");
    assert_eq!(last.card, cid(EARRINGS));
    assert!(last.from_field);
    assert_eq!(st.player(me).shadows, shadows_before + 1);
}

#[test]
fn earrings_engage_empty_hand_still_draws() {
    let db = load_db();
    let mut st = started(&db, 10761211);
    let me = PlayerId::A;
    let earrings_slot = put_field(&db, &mut st, me, EARRINGS);
    clear_hand(&mut st, me);
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, DECK_TOP);
    let deck_len_before = st.player(me).deck.len();

    let events = apply(
        &db,
        &mut st,
        Action::Engage {
            slot: Slot(earrings_slot),
        },
    )
    .expect("engage");
    assert!(matches!(st.phase, Phase::Main));
    let draws = events
        .iter()
        .filter(|e| matches!(e, Event::Draw { player, .. } if *player == me))
        .count();
    assert_eq!(draws, 1);
    assert_eq!(st.player(me).hand.len(), 1);
    assert_eq!(st.player(me).hand[0].card, cid(DECK_TOP));
    assert_eq!(st.player(me).deck.len(), deck_len_before - 1);
}

#[test]
fn earrings_engage_destroy_reaction_waits_for_the_text() {
    let db = load_db();
    let mut st = started(&db, 10761212);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, OMERIO);
    let earrings_slot = put_field(&db, &mut st, me, EARRINGS);
    let enemy_slot = put_field(&db, &mut st, opp, TANK);
    let def_before = st.player(opp).field[enemy_slot as usize]
        .as_ref()
        .expect("tank")
        .defense;
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, HAND_CARD);
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, DECK_TOP);

    apply(
        &db,
        &mut st,
        Action::Engage {
            slot: Slot(earrings_slot),
        },
    )
    .expect("engage");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    let omerio = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(OMERIO))
        .expect("omerio");
    assert_eq!(
        omerio.sequence_index, 0,
        "destroy reaction waits for Engage text"
    );

    choose(&db, &mut st, 0);

    let omerio = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(OMERIO))
        .expect("omerio after pick");
    assert_eq!(omerio.sequence_index, 1);
    let def_after = st.player(opp).field[enemy_slot as usize]
        .as_ref()
        .expect("tank")
        .defense;
    assert_eq!(def_after, def_before - 3);
}

struct RecordedReplayRow {
    path: &'static str,
    use_recorded_db: bool,
    ply: usize,
    want_hash: u64,
    breaks_at: usize,
}

fn replay_to_ply(
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

#[test]
fn as_recorded_overlay_replays_like_main() {
    use serde_json::Value;
    use std::fs;

    let rows = [
        RecordedReplayRow {
            path: "hbcheck/neg-play-100087-ply0065.json",
            use_recorded_db: true,
            ply: 65,
            want_hash: 0x55023cb9ecf25a0c,
            breaks_at: 16,
        },
        RecordedReplayRow {
            path: "hbcheck/pos-play-100085-ply0058.json",
            use_recorded_db: true,
            ply: 58,
            want_hash: 0x6258bfd1771cfa5a,
            breaks_at: 44,
        },
        RecordedReplayRow {
            path: "tkroll/fa-play-103-ply0100.json",
            use_recorded_db: true,
            ply: 100,
            want_hash: 0x89909d423301302f,
            breaks_at: 44,
        },
        RecordedReplayRow {
            path: "tkroll/fa-play-150-ply0086.json",
            use_recorded_db: true,
            ply: 86,
            want_hash: 0xeeda041e005d64d8,
            breaks_at: 26,
        },
        RecordedReplayRow {
            path: "tkroll/fb-play-122-ply0064.json",
            use_recorded_db: true,
            ply: 64,
            want_hash: 0x9bdc48e2b98eaade,
            breaks_at: 19,
        },
    ];

    for row in &rows {
        let path = fixtures_dir().join(row.path);
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let base = if row.use_recorded_db {
            load_recorded_db()
        } else {
            load_db()
        };
        let with_overlay = with_earrings_engage_as_recorded(base);
        let st = replay_to_ply(&with_overlay, &cap, row.ply).expect("overlay replay");
        assert_eq!(hash(&st), row.want_hash, "{}", row.path);

        let fixed_db = if row.use_recorded_db {
            let mut db = load_db();
            db.load_replace_dir(repo_root().join("oracle/cards-as-recorded"))
                .expect("recorded");
            db
        } else {
            load_db()
        };
        let err = replay_to_ply(&fixed_db, &cap, row.breaks_at + 1);
        assert!(
            err.is_err(),
            "{}: expected replay to fail at action {}",
            row.path,
            row.breaks_at
        );
        if let Err((step, _)) = err {
            assert_eq!(step, row.breaks_at, "{}", row.path);
        }
    }

    let review_path =
        repo_root().join("py/tests/fixtures/review14/games/6598261483642061665-d90f8eb2.json");
    let cap: Value =
        serde_json::from_str(&fs::read_to_string(&review_path).expect("read")).expect("json");
    for (ply, want_hash, breaks_at) in [
        (27usize, 0x8a70c03b16308d7a_u64, 16usize),
        (52usize, 0xcdd772513492b496_u64, 16usize),
        (104usize, 0xfd49bf3f01b49fd4_u64, 16usize),
    ] {
        let with_overlay = with_earrings_engage_as_recorded(load_db());
        let st = replay_to_ply(&with_overlay, &cap, ply).expect("review overlay");
        assert_eq!(hash(&st), want_hash, "ply {ply}");

        let err = replay_to_ply(&load_db(), &cap, breaks_at + 1);
        assert!(
            err.is_err(),
            "ply {ply}: expected fail at action {breaks_at}"
        );
        if let Err((step, _)) = err {
            assert_eq!(step, breaks_at, "ply {ply}");
        }
    }
}
