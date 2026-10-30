//! Recorded `play_game` traces match H0 explain paths and replay.

mod common;

use arena_engine::policy::Policy;
use arena_engine::{
    acting_player, apply, legal_actions, new_game, play_game_record, policy_rng, AnyPolicy, CardId,
    First, GameConfig, PlayerId,
};

use common::{load_db, load_deck_file, repo_root};

fn deck_pairs() -> Vec<(Vec<CardId>, Vec<CardId>)> {
    let root = repo_root();
    let paths = [
        "oracle/decks/basic-forest.json",
        "oracle/decks/basic-rune.json",
        "oracle/decks/meta-abyss-aggro.json",
        "oracle/decks/meta-haven-evo.json",
        "oracle/decks/meta-dragon-ramp.json",
        "oracle/decks/meta-forest-combo.json",
        "oracle/decks/meta-portal-af.json",
        "oracle/decks/meta-portal-cutthroat.json",
    ];
    paths
        .chunks(2)
        .map(|c| {
            (
                load_deck_file(root.join(c[0])),
                load_deck_file(root.join(c[1])),
            )
        })
        .collect()
}

#[test]
fn recorded_paths_match_explain_and_replay() {
    let db = load_db();
    let specs = [
        "h0:nodes=300,mull=leaf",
        "h0:nodes=300,deal=block",
        "h0-fast",
        "h0:nodes=500",
    ];
    let seeds = [1u64, 7, 42, 99];
    for spec in specs {
        for seed in seeds {
            for (deck_a, deck_b) in deck_pairs() {
                let mut state = new_game(
                    &db,
                    GameConfig {
                        seed,
                        deck_a: deck_a.clone(),
                        deck_b: deck_b.clone(),
                        first: First::A,
                        opening_hands: None,
                    },
                )
                .unwrap();
                let mut a = AnyPolicy::parse_spec(spec).unwrap();
                let mut b = AnyPolicy::parse_spec("random").unwrap();
                let mut rng = policy_rng(seed);
                let out = play_game_record(&db, &mut state, &mut a, &mut b, &mut rng, true);
                assert!(!out.chosen.is_empty());
                assert_eq!(out.chosen.len(), out.paths.len());
                assert_eq!(out.chosen.len(), out.values.len());

                let mut replay = new_game(
                    &db,
                    GameConfig {
                        seed,
                        deck_a,
                        deck_b,
                        first: First::A,
                        opening_hands: None,
                    },
                )
                .unwrap();
                let mut rr = policy_rng(seed);
                let mut pol_a = AnyPolicy::parse_spec(spec).unwrap();
                let mut pol_b = AnyPolicy::parse_spec("random").unwrap();
                for (step, &chosen) in out.chosen.iter().enumerate() {
                    let legal = legal_actions(&db, &replay);
                    assert!(!legal.is_empty());
                    let idx = chosen as usize;
                    assert!(idx < legal.len());
                    let pick = match acting_player(&replay) {
                        PlayerId::A => {
                            if let AnyPolicy::H0(ref mut h0) = pol_a {
                                h0.arm_explain();
                            }
                            let p = pol_a.choose(&db, &replay, &legal, &mut rr);
                            if let AnyPolicy::H0(ref mut h0) = pol_a {
                                let rec = h0.take_explain().expect("explain");
                                assert_eq!(
                                    out.paths[step],
                                    rec.path.code(),
                                    "seed={seed} spec={spec} step={step}"
                                );
                            }
                            p
                        }
                        PlayerId::B => pol_b.choose(&db, &replay, &legal, &mut rr),
                    };
                    assert_eq!(pick, idx);
                    apply(&db, &mut replay, legal[idx].clone()).unwrap();
                }
                assert_eq!(replay.winner, out.winner);
                assert_eq!(replay.turn, out.turns);
                assert_eq!(out.chosen.len() as u32, out.actions);
                assert_eq!(arena_engine::hash(&replay), out.final_hash);
            }
        }
    }
}
