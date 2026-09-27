//! Encoding version 2: strict pool, zone bonuses, model loading.

use arena_engine::determinize::{determinize_with, Info};
use arena_engine::{
    apply, encode, encode_version, encode_with_vocab, legal_actions, new_game, policy_rng, vocab,
    Action, EncodingVersion, First, GameConfig, Observation, Phase, PlayerId, ValueNet,
};

mod common;
use common::*;

const POOL_OFF: usize = 353 + arena_engine::encode::HIST_WIDTH;
const POOL_WIDTH: usize = arena_engine::encode::HIST_WIDTH;
const V2_EXTRA_OFF: usize = arena_engine::encode::LEN_V1;
const V2_EXTRA_LEN: usize = 18;
const OPEN_WORLDS: u64 = 8;

fn offset(name: &str) -> usize {
    Observation::LAYOUT_V2_EXTRA
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.offset)
        .unwrap_or_else(|| panic!("missing layout field {name}"))
}

fn thestae_crest() -> arena_engine::state::CrestInstance {
    arena_engine::state::CrestInstance {
        id: "crest:10714110".into(),
        countdown: Some(3),
        faith: false,
        once_used: vec![],
        granted_order: 1,
        granted: vec![],
        choose_used: Default::default(),
    }
}

fn forest_combo_vs_sword_rally(db: &arena_engine::CardDb, seed: u64) -> arena_engine::State {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: load_deck_file("oracle/decks/meta-forest-combo.json"),
            deck_b: load_deck_file("oracle/decks/meta-sword-rally.json"),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull A");
    apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull B");
    assert!(matches!(state.phase, Phase::Main));
    state
}

fn trigger_thestae_deck_buff(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    let me = PlayerId::A;
    st.player_mut(me).crests.push(thestae_crest());
    st.player_mut(me).combo = 3;
    end_turn(db, st);
}

fn meta_deck_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

#[test]
fn v1_encode_unchanged_on_thestae_crest_buff() {
    let db = load_db();
    let mut st = forest_combo_vs_sword_rally(&db, 5);
    let me = PlayerId::A;
    trigger_thestae_deck_buff(&db, &mut st);
    let buffed = encode(&st, me);
    for c in &mut st.player_mut(me).deck {
        if c.kind == arena_engine::CardKind::Follower {
            if let Ok(card) = db.card(c.card) {
                c.attack = card.attack();
                c.defense = card.defense();
                c.max_defense = card.defense();
            }
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
    let mut st = forest_combo_vs_sword_rally(&db, 5);
    trigger_thestae_deck_buff(&db, &mut st);
    let me = PlayerId::A;
    let v2 = encode_version(&st, me, EncodingVersion::V2, Some(&db));
    let expected = [17.0, 9.0, 5.0, 3.0, 17.0, 9.0, 5.0, 3.0, 0.0];
    let got: Vec<f32> = expected
        .iter()
        .enumerate()
        .map(|(i, _)| v2.features[V2_EXTRA_OFF + i])
        .collect();
    assert_eq!(
        got,
        expected.to_vec(),
        "deck atk/def buckets + storm + cost reduction"
    );
}

#[test]
fn v2_hand_bonuses_see_drawn_buffed_follower() {
    let db = load_db();
    let mut st = forest_combo_vs_sword_rally(&db, 5);
    trigger_thestae_deck_buff(&db, &mut st);
    let me = PlayerId::A;
    while st.active != me
        || !matches!(st.phase, Phase::Main)
        || st.player(me).turns_taken < 2
    {
        assert!(
            st.winner.is_none() && !matches!(st.phase, Phase::Terminal),
            "game ended before A turn 2"
        );
        let legal = legal_actions(&db, &st);
        assert!(!legal.is_empty(), "no legal actions");
        if legal.iter().any(|a| matches!(a, Action::EndTurn)) {
            end_turn(&db, &mut st);
        } else if matches!(st.phase, Phase::Choice { .. }) {
            choose(&db, &mut st, 0);
        } else {
            apply(&db, &mut st, legal[0].clone()).expect("apply");
        }
    }

    let magachiyo = cid("10914110");
    let drawn = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card == magachiyo)
        .expect("Magachiyo drawn into hand");
    assert_eq!(drawn.attack, 3, "Thestae deck buff +1 atk (printed 2)");
    assert_eq!(drawn.max_defense, 3, "Thestae deck buff +1 def (printed 2)");

    let obs = encode_version(&st, me, EncodingVersion::V2, Some(&db));
    assert_eq!(obs.features[offset("own_hand_atk_bonus_cost_3_4")], 1.0);
    assert_eq!(obs.features[offset("own_hand_def_bonus_cost_3_4")], 1.0);

    assert_eq!(obs.features[offset("own_hand_cost_reduction")], 1.0);
}

#[test]
fn v2_strict_pool_counts_fuse_partners_as_remaining() {
    let db = load_db();
    let mut st = started_decks(&db, 5, &["90071210", "90071220"], &["88001110"]);
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
    assert_eq!(
        st.player(me).hidden_removal_cards,
        vec![partner_card],
        "parallel card-id log"
    );

    let v = vocab(&st);
    let idx = v
        .binary_search(&partner_card)
        .expect("fuse partner is in decklist vocab");

    let v1 = encode(&st, opp);
    let v2 = encode_version(&st, opp, EncodingVersion::V2, Some(&db));
    let v1_pool = v1.features[POOL_OFF + idx];
    let v2_pool = v2.features[POOL_OFF + idx];
    assert_eq!(
        v2_pool,
        v1_pool + 1.0,
        "strict pool adds hidden fuse partner back (v1={v1_pool} v2={v2_pool})"
    );

    for i in 0..POOL_OFF {
        assert_eq!(v1.features[i], v2.features[i], "feat {i}");
    }
}

#[test]
fn v2_pool_and_zone_bonuses_stable_across_open_worlds() {
    let db = load_db();
    let decks: Vec<Vec<arena_engine::CardId>> = meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(&db, d) && d.len() == 40)
        .collect();
    assert_eq!(decks.len(), 16, "sixteen meta decks");

    let mut states_with_hidden = 0usize;
    let mut worlds_checked = 0usize;
    let mut mismatches = 0usize;
    let mut seed = 1u64;

    while states_with_hidden < 500 && seed < 2_000 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        ) else {
            seed += 1;
            continue;
        };
        let mut rng = policy_rng(seed);
        let mut nact = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            for me in [PlayerId::A, PlayerId::B] {
                let opp = me.opponent();
                if state.player(opp).hidden_removals.is_empty() {
                    continue;
                }
                states_with_hidden += 1;
                let root_vocab = vocab(&state);
                let real_v1 = encode_with_vocab(&state, me, &root_vocab, EncodingVersion::V1, None);
                let real_v2 =
                    encode_with_vocab(&state, me, &root_vocab, EncodingVersion::V2, Some(&db));
                let ref_v1_pool = &real_v1.features[POOL_OFF..POOL_OFF + POOL_WIDTH];
                let ref_v2_pool = &real_v2.features[POOL_OFF..POOL_OFF + POOL_WIDTH];
                let ref_v2_extra = &real_v2.features[V2_EXTRA_OFF..V2_EXTRA_OFF + V2_EXTRA_LEN];

                for w in 0..OPEN_WORLDS {
                    worlds_checked += 1;
                    let world = determinize_with(&state, me, seed * 1_000 + w, Info::Open);
                    let world_v1 =
                        encode_with_vocab(&world, me, &root_vocab, EncodingVersion::V1, None);
                    let world_v2 =
                        encode_with_vocab(&world, me, &root_vocab, EncodingVersion::V2, Some(&db));
                    let v1_pool = &world_v1.features[POOL_OFF..POOL_OFF + POOL_WIDTH];
                    let v2_pool = &world_v2.features[POOL_OFF..POOL_OFF + POOL_WIDTH];
                    let v2_extra = &world_v2.features[V2_EXTRA_OFF..V2_EXTRA_OFF + V2_EXTRA_LEN];
                    if v1_pool != ref_v1_pool || v2_pool != ref_v2_pool || v2_extra != ref_v2_extra
                    {
                        mismatches += 1;
                    }
                }
            }

            if state.turn > 30 || nact >= 500 {
                break;
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
            nact += 1;
        }
        seed += 1;
    }

    assert!(
        states_with_hidden >= 100,
        "expected many states with hidden removals, got {states_with_hidden}"
    );
    assert_eq!(
        mismatches, 0,
        "pool/zone mismatch across {worlds_checked} Open worlds \
         ({states_with_hidden} states with hidden removals)"
    );
    eprintln!(
        "v2_pool_open_worlds: states={states_with_hidden} worlds={worlds_checked} mismatches={mismatches}"
    );
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

#[test]
fn model_rejects_truncated_encoding() {
    let zone_w: Vec<Vec<f32>> = (0..5).map(|_| vec![0.0]).collect();
    let json = serde_json::json!({
        "arch": "linear",
        "encoding": 258,
        "feature_len": 563,
        "feat_mean": vec![0.0; 563],
        "feat_std": vec![1.0; 563],
        "vocab": [0u32],
        "zones": [
            {"name": "own_hand", "id_offset": 0, "count": 9},
            {"name": "own_deck", "id_offset": 9, "count": 96, "hist_offset": 353},
            {"name": "opp_board", "id_offset": 105, "count": 5},
            {"name": "own_board", "id_offset": 110, "count": 5},
            {"name": "opp_pool", "id_offset": 115, "count": 96, "hist_offset": 449},
        ],
        "scale": 1.0,
        "linear": {"w": vec![0.0; 563], "zone_w": zone_w, "b": 0.0},
    });
    let err = ValueNet::from_json(&json.to_string()).unwrap_err();
    assert!(err.contains("encoding"), "{err}");
}

fn choose(db: &arena_engine::CardDb, st: &mut arena_engine::State, idx: u32) {
    apply(db, st, arena_engine::Action::Choose(idx as u8)).expect("choose");
}

fn confirm(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    apply(db, st, arena_engine::Action::Confirm).expect("confirm");
}
