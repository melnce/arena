//! Encoding version 2: strict pool, zone bonuses, model loading.

use arena_engine::{
    apply, encode, encode_version, CardInstance, CardKind, EncodingVersion, Observation, PlayerId,
    ValueNet,
};

mod common;
use common::*;

fn offset(name: &str) -> usize {
    Observation::LAYOUT_V2_EXTRA
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.offset)
        .unwrap_or_else(|| panic!("missing layout field {name}"))
}

#[test]
fn v1_encode_unchanged_on_thestae_crest_buff() {
    let db = load_db();
    let mut st = started_decks(&db, 103, &["88001110", "88001110"], &["88001110"]);
    let me = PlayerId::A;
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10714110".into(),
            countdown: Some(3),
            faith: false,
            once_used: vec![],
            granted_order: 1,
            granted: vec![],
            choose_used: Default::default(),
        });
    st.player_mut(me).combo = 3;
    for c in &mut st.player_mut(me).deck {
        if c.kind == CardKind::Follower {
            c.attack += 1;
            c.defense += 1;
            c.max_defense += 1;
        }
    }
    let buffed = encode(&st, me);
    for c in &mut st.player_mut(me).deck {
        if let Ok(card) = db.card(c.card) {
            c.attack = card.attack();
            c.defense = card.defense();
            c.max_defense = card.defense();
        }
    }
    let unbuffed = encode(&st, me);
    assert_eq!(
        buffed.features, unbuffed.features,
        "v1 ignores deck stat buffs"
    );
}

#[test]
fn v2_deck_bonuses_rise_on_thestae_crest_buff() {
    let db = load_db();
    let mut st = started_decks(&db, 103, &["88001110", "88001110"], &["88001110"]);
    let me = PlayerId::A;
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10714110".into(),
            countdown: Some(3),
            faith: false,
            once_used: vec![],
            granted_order: 1,
            granted: vec![],
            choose_used: Default::default(),
        });
    st.player_mut(me).combo = 3;
    for c in &mut st.player_mut(me).deck {
        if c.kind == CardKind::Follower {
            c.attack += 1;
            c.defense += 1;
            c.max_defense += 1;
        }
    }
    let v2_after = encode_version(&st, me, EncodingVersion::V2, Some(&db));

    let deck = &st.player(me).deck;
    let mut le2_atk = 0.0f32;
    let mut le2_def = 0.0f32;
    let mut c34_atk = 0.0f32;
    let mut c34_def = 0.0f32;
    let mut ge5_atk = 0.0f32;
    let mut ge5_def = 0.0f32;
    let mut storm_atk = 0.0f32;
    let mut storm_def = 0.0f32;
    let mut followers = 0u32;
    for c in deck {
        if c.kind != CardKind::Follower {
            continue;
        }
        followers += 1;
        let card = db.card(c.card).unwrap();
        let atk = (c.attack - card.attack()) as f32;
        let def = (c.max_defense - card.defense()) as f32;
        let cost = c.cost;
        if cost <= 2 {
            le2_atk += atk;
            le2_def += def;
        } else if cost <= 4 {
            c34_atk += atk;
            c34_def += def;
        } else {
            ge5_atk += atk;
            ge5_def += def;
        }
        if c.is_storm() {
            storm_atk += atk;
            storm_def += def;
        }
    }
    assert_eq!(v2_after.features[offset("own_deck_atk_bonus_le2")], le2_atk);
    assert_eq!(
        v2_after.features[offset("own_deck_atk_bonus_cost_3_4")],
        c34_atk
    );
    assert_eq!(
        v2_after.features[offset("own_deck_atk_bonus_cost_ge5")],
        ge5_atk
    );
    assert_eq!(
        v2_after.features[offset("own_deck_atk_bonus_storm")],
        storm_atk
    );
    assert_eq!(v2_after.features[offset("own_deck_def_bonus_le2")], le2_def);
    assert_eq!(
        v2_after.features[offset("own_deck_def_bonus_cost_3_4")],
        c34_def
    );
    assert_eq!(
        v2_after.features[offset("own_deck_def_bonus_cost_ge5")],
        ge5_def
    );
    assert_eq!(
        v2_after.features[offset("own_deck_def_bonus_storm")],
        storm_def
    );
    assert_eq!(
        followers, le2_atk as u32,
        "all deck followers are cost <= 2"
    );
    assert!(followers > 0);
}

#[test]
fn v2_hand_bonuses_see_buffed_follower() {
    let db = load_db();
    let mut st = started(&db, 104);
    let me = PlayerId::A;
    let card = db.card(cid("88001110")).expect("follower");
    let mut buffed = CardInstance::from_card(card, st.alloc_id());
    buffed.attack += 1;
    buffed.defense += 1;
    buffed.max_defense += 1;
    st.player_mut(me).hand.clear();
    st.player_mut(me).hand.push(buffed);
    let obs = encode_version(&st, me, EncodingVersion::V2, Some(&db));
    assert_eq!(obs.features[offset("own_hand_atk_bonus_le2")], 1.0);
    assert_eq!(obs.features[offset("own_hand_def_bonus_le2")], 1.0);
}

#[test]
fn v2_strict_pool_counts_fuse_partners_as_remaining() {
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, "90071210");
    let partner_pos = put_hand(&db, &mut st, me, "90071220");
    let partner_id = st.player(me).hand[partner_pos as usize].id;
    let partner_card = st.player(me).hand[partner_pos as usize].card;
    apply(&db, &mut st, arena_engine::Action::Fuse { host: 0 }).unwrap();
    choose(&db, &mut st, 0);
    confirm(&db, &mut st);
    assert!(st.player(me).hidden_removals.contains(&partner_id));

    let known = st.player(me).known_remaining_pool();
    let strict = st.player(me).strict_remaining_pool();
    let k = known.get(&partner_card).copied().unwrap_or(0);
    let s = strict.get(&partner_card).copied().unwrap_or(0);
    assert!(
        s > k,
        "strict pool keeps hidden fuse partner (known={k} strict={s})"
    );

    let v1 = encode(&st, opp);
    let v2 = encode_version(&st, opp, EncodingVersion::V2, Some(&db));
    let pool_off = 353 + arena_engine::encode::HIST_WIDTH;
    for i in 0..pool_off {
        assert_eq!(v1.features[i], v2.features[i], "feat {i}");
    }
    for i in pool_off + arena_engine::encode::HIST_WIDTH..Observation::LEN {
        assert_eq!(v1.features[i], v2.features[i], "feat {i}");
    }
}

#[test]
fn v2_model_loads_and_evaluates() {
    let db = load_db();
    let st = started(&db, 1);
    let obs = encode_version(&st, PlayerId::A, EncodingVersion::V2, Some(&db));
    let n = Observation::LEN_V2;
    let mean: Vec<f32> = vec![0.0; n];
    let std: Vec<f32> = vec![1.0; n];
    let mut w: Vec<f32> = vec![0.0; n];
    w[0] = 1.0;
    let zone_w: Vec<Vec<f32>> = (0..5).map(|_| vec![0.0]).collect();
    let json = serde_json::json!({
        "arch": "linear",
        "encoding": 2,
        "feature_len": n,
        "feat_mean": mean,
        "feat_std": std,
        "vocab": [0u32],
        "zones": [
            {"name": "own_hand", "id_offset": 0, "count": 9},
            {"name": "own_deck", "id_offset": 9, "count": 96, "hist_offset": 353},
            {"name": "opp_board", "id_offset": 105, "count": 5},
            {"name": "own_board", "id_offset": 110, "count": 5},
            {"name": "opp_pool", "id_offset": 115, "count": 96, "hist_offset": 449},
        ],
        "scale": 1.0,
        "linear": {"w": w, "zone_w": zone_w, "b": 0.0},
    });
    let net = ValueNet::from_json(&json.to_string()).expect("v2 model parses");
    let v = net.value(&obs);
    assert!(v.is_finite());
}

#[test]
fn model_rejects_feature_len_mismatch() {
    let zone_w: Vec<Vec<f32>> = (0..5).map(|_| vec![0.0]).collect();
    let json = serde_json::json!({
        "arch": "linear",
        "encoding": 2,
        "feature_len": 545,
        "feat_mean": [0.0],
        "feat_std": [1.0],
        "vocab": [0u32],
        "zones": [
            {"name": "own_hand", "id_offset": 0, "count": 9},
            {"name": "own_deck", "id_offset": 9, "count": 96, "hist_offset": 353},
            {"name": "opp_board", "id_offset": 105, "count": 5},
            {"name": "own_board", "id_offset": 110, "count": 5},
            {"name": "opp_pool", "id_offset": 115, "count": 96, "hist_offset": 449},
        ],
        "scale": 1.0,
        "linear": {"w": [0.0], "zone_w": zone_w, "b": 0.0},
    });
    let err = ValueNet::from_json(&json.to_string()).unwrap_err();
    assert!(err.contains("feature_len"), "{err}");
}

fn clear_hand(st: &mut arena_engine::State, who: PlayerId) {
    st.player_mut(who).hand.clear();
}

fn choose(db: &arena_engine::CardDb, st: &mut arena_engine::State, idx: u32) {
    apply(db, st, arena_engine::Action::Choose(idx as u8)).expect("choose");
}

fn confirm(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    apply(db, st, arena_engine::Action::Confirm).expect("confirm");
}
