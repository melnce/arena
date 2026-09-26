//! Learned mulligan: `mull=rule|random|<table>`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use arena_engine::{
    acting_player, deck_fingerprint_player, legal_actions, new_game, play_game, policy_rng, Action,
    AnyPolicy, CardDb, CardId, First, GameConfig, MullMode, OpeningHands, PlayerId, Policy, H0,
};

mod common;
use common::*;

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap() {
        AnyPolicy::H0(h) => h,
        _ => panic!("expected h0"),
    }
}

fn mull_pick(
    db: &CardDb,
    h: &mut H0,
    state: &arena_engine::State,
    rng: &mut arena_engine::Xoshiro256ss,
) -> Action {
    let legal = legal_actions(db, state);
    let idx = h.choose(db, state, &legal, rng);
    legal[idx].clone()
}

fn fixture(name: &str) -> String {
    fixtures_dir()
        .join("mulligan")
        .join(name)
        .display()
        .to_string()
}

#[test]
fn mull_spec_round_trips() {
    assert_eq!(
        AnyPolicy::parse_spec("h0:mull=rule").unwrap().spec(),
        "h0:mull=rule"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:mull=builtin").unwrap().spec(),
        "h0"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:mull=random").unwrap().spec(),
        "h0:mull=random"
    );
    let path = fixture("meta-rune-test-subject.json");
    let spec = format!("h0:mull={path}");
    let parsed = AnyPolicy::parse_spec(&spec).unwrap();
    assert_eq!(parsed.spec(), spec);
    let again = AnyPolicy::parse_spec(&parsed.spec()).unwrap();
    assert_eq!(again, parsed);
    let combo = AnyPolicy::parse_spec("h0:mull=random,nodes=4000").unwrap();
    let reparsed = AnyPolicy::parse_spec(&combo.spec()).unwrap();
    assert_eq!(combo, reparsed);
}

#[test]
fn mull_parse_errors() {
    let missing = "/no/such/mulligan-table.json";
    let e = AnyPolicy::parse_spec(&format!("h0:mull={missing}")).unwrap_err();
    assert!(e.contains(missing), "{e}");
    let e = AnyPolicy::parse_spec(&format!("h0:mull={}", fixture("bad-version.json"))).unwrap_err();
    assert!(e.contains("version"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mull=").unwrap_err();
    assert!(!e.is_empty());
}

#[test]
fn mull_random_same_seed_same_mask() {
    let db = load_db();
    let mut h1 = parse_h0("h0:mull=random");
    let mut h2 = parse_h0("h0:mull=random");
    let st = new_game(
        &db,
        GameConfig {
            seed: 99,
            deck_a: pad_deck(&["10001110"], 40),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let a1 = mull_pick(&db, &mut h1, &st, &mut policy_rng(99));
    let a2 = mull_pick(&db, &mut h2, &st, &mut policy_rng(99));
    assert_eq!(a1, a2);
}

#[test]
fn mull_random_slot_frequencies() {
    let db = load_db();
    let deck = pad_deck(&["10001110", "10011130", "10021120", "10031110"], 40);
    let mut slot_hits = [0u64; 4];
    let n = 2500;
    for seed in 0..n {
        let st = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: deck.clone(),
                deck_b: pad_deck(&["88001110"], 40),
                first: First::A,
                opening_hands: None,
            },
        )
        .unwrap();
        let mut h = parse_h0("h0:mull=random");
        let mut rng = policy_rng(seed);
        let a = mull_pick(&db, &mut h, &st, &mut rng);
        let swap = match a {
            Action::MulliganConfirm { swap } => swap,
            _ => panic!("expected mulligan"),
        };
        for i in 0..4 {
            if swap[i] {
                slot_hits[i] += 1;
            }
        }
    }
    for (i, hits) in slot_hits.iter().enumerate() {
        let f = *hits as f64 / n as f64;
        assert!(
            (0.46..=0.54).contains(&f),
            "slot {i} send-back rate {f} outside [0.46, 0.54]"
        );
    }
}

#[test]
fn mull_table_keeps_sephie() {
    let db = load_db();
    let path = fixture("meta-rune-test-subject.json");
    let spec = format!("h0:mull={path}");
    let sephie = cid("10934110");
    let cheap = cid("10031210");
    let st = new_game(
        &db,
        GameConfig {
            seed: 1,
            deck_a: load_deck_file(&oracle_deck("meta-rune-test-subject")),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: Some(OpeningHands {
                a: vec![sephie, cheap, cheap, cheap],
                b: vec![cid("88001110"); 4],
            }),
        },
    )
    .unwrap();
    let mut h = parse_h0(&spec);
    let a = mull_pick(&db, &mut h, &st, &mut policy_rng(1));
    assert_eq!(
        a,
        Action::MulliganConfirm {
            swap: [false, true, true, true]
        }
    );
    assert_eq!(h.stats.mull_table, 1);
    assert_eq!(h.stats.mull_fallback, 0);
}

#[test]
fn mull_table_missing_deck_uses_rule() {
    let db = load_db();
    let path = fixture("no-deck.json");
    let spec = format!("h0:mull={path}");
    let dear = cid("10011130");
    let cheap = cid("10001110");
    let st = new_game(
        &db,
        GameConfig {
            seed: 23,
            deck_a: pad_deck(
                &["10001110", "10001110", "10011130", "10011130", "88001110"],
                40,
            ),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: Some(OpeningHands {
                a: vec![cheap, cheap, dear, dear],
                b: vec![cid("88001110"); 4],
            }),
        },
    )
    .unwrap();
    let mut h = parse_h0(&spec);
    let a = mull_pick(&db, &mut h, &st, &mut policy_rng(1));
    assert_eq!(
        a,
        Action::MulliganConfirm {
            swap: [false, false, true, true]
        }
    );
    assert_eq!(h.stats.mull_fallback, 1);
}

#[test]
fn mull_table_missing_card_uses_rule_for_that_card() {
    let db = load_db();
    let path = fixture("partial-card.json");
    let spec = format!("h0:mull={path}");
    let sephie = cid("10934110");
    let dear = cid("10834110");
    let st = new_game(
        &db,
        GameConfig {
            seed: 1,
            deck_a: load_deck_file(&oracle_deck("meta-rune-test-subject")),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: Some(OpeningHands {
                a: vec![sephie, dear, dear, dear],
                b: vec![cid("88001110"); 4],
            }),
        },
    )
    .unwrap();
    assert!(st.player(PlayerId::A).hand[1].cost >= 4);
    let mut h = parse_h0(&spec);
    let a = mull_pick(&db, &mut h, &st, &mut policy_rng(1));
    // Sephie kept (table); cost-4 card missing from table → rule send back.
    assert_eq!(
        a,
        Action::MulliganConfirm {
            swap: [false, true, true, true]
        }
    );
}

#[test]
fn deck_fingerprint_matches_deck_file() {
    let db = load_db();
    let deck = load_deck_file(&oracle_deck("meta-rune-test-subject"));
    let mut counts: BTreeMap<CardId, u32> = BTreeMap::new();
    for id in &deck {
        counts.entry(*id).and_modify(|n| *n += 1).or_insert(1);
    }
    let fp_file = arena_engine::deck_fingerprint_counts(&counts);
    let st = new_game(
        &db,
        GameConfig {
            seed: 1,
            deck_a: deck.clone(),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let me = acting_player(&st);
    let fp_live = deck_fingerprint_player(st.player(me));
    assert_eq!(fp_file, fp_live);
}

fn oracle_deck(stem: &str) -> PathBuf {
    repo_root()
        .join("oracle/decks")
        .join(format!("{stem}.json"))
}

fn load_deck_file(path: &PathBuf) -> Vec<CardId> {
    let text = std::fs::read_to_string(path).unwrap();
    let map: BTreeMap<String, u32> = serde_json::from_str(&text).unwrap();
    let mut ids = Vec::new();
    for (k, n) in map {
        for _ in 0..n {
            ids.push(cid(&k));
        }
    }
    ids
}

#[test]
fn mull_builtin_equals_default_h0() {
    let db = load_db();
    let deck = pad_deck(&["10001110", "10011130"], 40);
    let st = new_game(
        &db,
        GameConfig {
            seed: 5,
            deck_a: deck.clone(),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let mut def = H0::default();
    let mut builtin = parse_h0("h0:mull=builtin");
    let a = mull_pick(&db, &mut def, &st, &mut policy_rng(5));
    let b = mull_pick(&db, &mut builtin, &st, &mut policy_rng(5));
    assert_eq!(a, b);
    assert_eq!(def.mull, MullMode::Table);
    assert!(def.mull_table.is_some());
    assert!(def.mull_path.is_none());
}

#[test]
fn mull_rule_differs_from_default_on_meta_deck() {
    let db = load_db();
    let sephie = cid("10934110");
    let cheap = cid("10031210");
    let st = new_game(
        &db,
        GameConfig {
            seed: 1,
            deck_a: load_deck_file(&oracle_deck("meta-rune-test-subject")),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: Some(OpeningHands {
                a: vec![sephie, cheap, cheap, cheap],
                b: vec![cid("88001110"); 4],
            }),
        },
    )
    .unwrap();
    let mut def = H0::default();
    let mut rule = parse_h0("h0:mull=rule");
    let def_pick = mull_pick(&db, &mut def, &st, &mut policy_rng(1));
    let rule_pick = mull_pick(&db, &mut rule, &st, &mut policy_rng(1));
    assert_ne!(def_pick, rule_pick);
    assert_eq!(def.stats.mull_table, 1);
}

#[test]
fn play_game_records_mulligans() {
    let db = load_db();
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 42,
            deck_a: pad_deck(&["10001110"], 40),
            deck_b: pad_deck(&["10021120"], 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let mut a = AnyPolicy::parse_spec("random").unwrap();
    let mut b = AnyPolicy::parse_spec("random").unwrap();
    let out = play_game(&db, &mut state, &mut a, &mut b, &mut policy_rng(42));
    assert!(out.mulligans[0].is_some());
    assert!(out.mulligans[1].is_some());
    for rec in out.mulligans.iter().flatten() {
        assert!(!rec.hand.is_empty());
        assert_eq!(rec.hand.len(), rec.swap.iter().take(4).count());
    }
}
