//! H0 fusemacro: completions, no leaf inside partner choice, one ply, bench identity.

use arena_engine::policy::fuse_completion_partner_sets;
use arena_engine::{
    apply, determinize::determinize_with, from_neutral, legal_actions, new_game, policy_rng,
    Action, AnyPolicy, CardDb, ChoiceNode, ChoosePath, First, Phase, PlayerId, Policy, PvEnd, H0,
};

mod common;
use common::*;

const SEPHIE: &str = "10934110";
const META_DECKS: &[&str] = &[
    "meta-abyss-aggro",
    "meta-sword-rally",
    "meta-haven-evo",
    "meta-dragon-ramp",
    "meta-portal-af",
    "meta-forest-combo",
];

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn is_own_fuse_partners(state: &arena_engine::State, me: PlayerId) -> bool {
    matches!(
        state.phase,
        Phase::Choice {
            player,
            node: ChoiceNode::FusePartners { .. },
            ..
        } if player == me
    )
}

fn sephie_fuse_host(db: &CardDb, state: &arena_engine::State, me: PlayerId) -> Option<u8> {
    if !matches!(state.phase, Phase::Main) || state.active != me {
        return None;
    }
    legal_actions(db, state).iter().find_map(|a| {
        if let Action::Fuse { host } = a {
            state
                .player(me)
                .hand
                .get(*host as usize)
                .filter(|c| c.card.as_str() == SEPHIE)
                .map(|_| *host)
        } else {
            None
        }
    })
}

fn collect_sephie_fuse_positions(db: &CardDb, games: u32) -> Vec<arena_engine::State> {
    // Full h0 self-play is slow; a handful of games still yields enough fuse lines.
    let deck_a = load_deck_file("oracle/decks/meta-rune-test-subject.json");
    let mut out = Vec::new();
    for g in 0..games {
        let deck_b = load_deck_file(format!("oracle/decks/{}.json", META_DECKS[g as usize % 6]));
        let first = if g % 2 == 0 { First::A } else { First::B };
        let Ok(mut state) = new_game(
            db,
            arena_engine::GameConfig {
                seed: 40_000 + g as u64,
                deck_a: deck_a.clone(),
                deck_b,
                first,
                opening_hands: None,
            },
        ) else {
            continue;
        };
        let mut h0 = parse_h0("h0-fast");
        let mut rng = policy_rng(40_000 + g as u64);
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.active == PlayerId::A
                && matches!(state.phase, Phase::Main)
                && state.player(PlayerId::A).usable_pp() < 2
                && sephie_fuse_host(db, &state, PlayerId::A).is_some()
            {
                out.push(state.clone());
            }
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = h0.choose(db, &state, &legal, &mut rng);
            if apply(db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
    }
    out
}

#[test]
fn fuse_completions_sephie_three_singles() {
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    clear_hand(&mut st, me);
    let host = put_hand(&db, &mut st, me, SEPHIE);
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "10932310");
    put_hand(&db, &mut st, me, "10513110");
    put_hand(&db, &mut st, me, "88001110");
    apply(&db, &mut st, Action::Fuse { host }).expect("fuse");
    let sets = fuse_completion_partner_sets(&db, &st, me);
    assert_eq!(sets.len(), 3, "three [Choose, Confirm] completions");
    assert!(sets.iter().all(|s| s.len() == 1));
}

#[test]
fn fuse_completions_recipe_host_singles_and_pairs() {
    let db = load_db();
    let mut st = started(&db, 12);
    let me = PlayerId::A;
    clear_hand(&mut st, me);
    let host = put_hand(&db, &mut st, me, "90072110");
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    put_hand(&db, &mut st, me, "90071130");
    apply(&db, &mut st, Action::Fuse { host }).expect("fuse");
    let sets = fuse_completion_partner_sets(&db, &st, me);
    assert_eq!(sets.len(), 6);
    assert_eq!(sets.iter().filter(|s| s.len() == 1).count(), 3);
    assert_eq!(sets.iter().filter(|s| s.len() == 2).count(), 3);
}

#[test]
fn fuse_completions_non_recipe_picked_only_confirm() {
    let db = load_db();
    let mut st = started(&db, 13);
    let me = PlayerId::A;
    clear_hand(&mut st, me);
    let host = put_hand(&db, &mut st, me, SEPHIE);
    put_hand(&db, &mut st, me, "88001110");
    apply(&db, &mut st, Action::Fuse { host }).expect("fuse");
    apply(&db, &mut st, Action::Choose(0)).expect("choose");
    let sets = fuse_completion_partner_sets(&db, &st, me);
    assert_eq!(sets.len(), 1);
    assert!(sets[0].is_empty());
}

fn replay_world(
    db: &CardDb,
    root: &arena_engine::State,
    world: &arena_engine::WorldRecord,
) -> arena_engine::State {
    let mut s = root.clone();
    for neu in &world.pv {
        let a = from_neutral(&s, neu).expect("pv action maps");
        apply(db, &mut s, a).expect("pv replay legal");
    }
    s
}

#[test]
fn no_leaf_inside_partner_choice_with_fusemacro() {
    let db = load_db();
    let positions = collect_sephie_fuse_positions(&db, 8);
    assert!(
        !positions.is_empty(),
        "need Sephie fuse positions from meta-rune-test-subject games"
    );
    let me = PlayerId::A;
    let mut saw_mid_choice_off = false;
    for (i, state) in positions.iter().enumerate() {
        let legal = legal_actions(&db, state);
        let seed = 50_000u64 + i as u64;
        for spec in ["h0:fusemacro=1", "h0:value=v0,fusemacro=1"] {
            let h0_cfg = parse_h0(spec);
            let mut h0 = parse_h0(spec);
            h0.arm_explain();
            let mut rng = policy_rng(seed);
            let _ = h0.choose(&db, state, &legal, &mut rng);
            let rec = h0.take_explain().expect("explain");
            if !matches!(rec.path, ChoosePath::Search | ChoosePath::Unscored) {
                continue;
            }
            let roots = {
                let mut rng_r = policy_rng(seed);
                (0..rec.k)
                    .map(|_| determinize_with(state, me, rng_r.next_u64(), h0_cfg.info))
                    .collect::<Vec<_>>()
            };
            for cand in &rec.candidates {
                for world in &cand.worlds {
                    if world.skipped || world.pv.is_empty() {
                        continue;
                    }
                    let final_state = replay_world(&db, &roots[world.r as usize], world);
                    assert!(
                        !is_own_fuse_partners(&final_state, me),
                        "{spec} scored inside FusePartners at pos {i} world {}",
                        world.r
                    );
                }
            }
        }
        let mut h0 = parse_h0("h0");
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let _ = h0.choose(&db, state, &legal, &mut rng);
        let rec = h0.take_explain().expect("explain");
        if matches!(rec.path, ChoosePath::Search | ChoosePath::Unscored) {
            let roots = {
                let mut rng_r = policy_rng(seed);
                (0..rec.k)
                    .map(|_| determinize_with(state, me, rng_r.next_u64(), h0.info))
                    .collect::<Vec<_>>()
            };
            for cand in &rec.candidates {
                for world in &cand.worlds {
                    if world.skipped || world.pv.is_empty() {
                        continue;
                    }
                    let final_state = replay_world(&db, &roots[world.r as usize], world);
                    if is_own_fuse_partners(&final_state, me) {
                        saw_mid_choice_off = true;
                    }
                }
            }
        }
    }
    assert!(
        saw_mid_choice_off,
        "fusemacro=0 should score at least one world inside FusePartners"
    );
}

#[test]
fn fusemacro_one_ply_same_depth_as_play() {
    let db = load_db();
    let mut st = started(&db, 20);
    let me = PlayerId::A;
    clear_hand(&mut st, me);
    let sephie = put_hand(&db, &mut st, me, SEPHIE);
    put_hand(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 0, 10);
    let legal = legal_actions(&db, &st);
    let mut h0 = parse_h0("h0:fusemacro=1,depth=3,nodes=8000,k=1,tt=0,value=v0,olethal=0,osteps=0");
    h0.arm_explain();
    let mut rng = policy_rng(60_001);
    let _ = h0.choose(&db, &st, &legal, &mut rng);
    let rec = h0.take_explain().expect("explain");
    assert_eq!(rec.path, ChoosePath::Search);
    let fuse_pv_len = rec
        .candidates
        .iter()
        .find(|c| matches!(c.action, arena_engine::NeutralAction::Fuse { host_pos, .. } if host_pos == sephie))
        .and_then(|c| c.worlds.first())
        .filter(|w| !w.skipped && w.end == Some(PvEnd::Depth))
        .map(|w| w.pv_len);
    let play_pv_len = rec
        .candidates
        .iter()
        .find(|c| matches!(c.action, arena_engine::NeutralAction::Play { .. }))
        .and_then(|c| c.worlds.first())
        .filter(|w| !w.skipped && w.end == Some(PvEnd::Depth))
        .map(|w| w.pv_len);
    assert_eq!(
        fuse_pv_len, play_pv_len,
        "fuse macro line should reach the same PV depth as play"
    );
    let fuse_pv = rec
        .candidates
        .iter()
        .find(|c| matches!(c.action, arena_engine::NeutralAction::Fuse { .. }))
        .and_then(|c| c.worlds.first())
        .map(|w| w.pv.len())
        .unwrap_or(0);
    assert!(
        fuse_pv >= 3,
        "fuse PV should include Fuse, Choose, Confirm before the next action"
    );
}

/// Demonstration table for the PR (not asserted). Run with:
/// `cargo test --release -p arena-engine --test h0_fusemacro show_it -- --ignored --nocapture`
#[test]
#[ignore]
fn show_it_sephie_fuse_reasks() {
    let db = load_db();
    let deck_a = load_deck_file("oracle/decks/meta-rune-test-subject.json");
    let specs = [
        ("h0", "default h0"),
        ("h0:fusemacro=1", "h0:fusemacro=1"),
        ("h0:value=v0,osteps=0,olethal=0,nodes=200000", "v0 wide"),
        (
            "h0:value=v0,osteps=0,olethal=0,nodes=200000,fusemacro=1",
            "v0 wide + fusemacro",
        ),
    ];
    let mut decisions = 0u32;
    let mut still = [0u32; 4];
    eprintln!("| game | seed | still_fuses default | fusemacro | v0 wide | v0+fusemacro | fuse raw/end/pv | best other root_agg |");
    eprintln!("| --- | --- | --- | --- | --- | --- | --- | --- |");
    for g in 0..60u32 {
        let seed = 900u64 + g as u64;
        let deck_b = load_deck_file(format!("oracle/decks/{}.json", META_DECKS[g as usize % 6]));
        let first = if g % 2 == 0 { First::A } else { First::B };
        let Ok(mut state) = new_game(
            &db,
            arena_engine::GameConfig {
                seed,
                deck_a: deck_a.clone(),
                deck_b,
                first,
                opening_hands: None,
            },
        ) else {
            continue;
        };
        let mut rng = policy_rng(seed);
        let mut h0 = parse_h0("h0");
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            let is_decision = state.active == PlayerId::A
                && matches!(state.phase, Phase::Main)
                && state.player(PlayerId::A).usable_pp() < 2
                && sephie_fuse_host(&db, &state, PlayerId::A).is_some();
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = if is_decision {
                let pick = h0.choose(&db, &state, &legal, &mut rng);
                if matches!(legal[pick], Action::Fuse { .. }) {
                    decisions += 1;
                    let mut row = format!("| {g} | {seed} | yes | ");
                    for (si, (spec, _)) in specs.iter().enumerate().skip(1) {
                        let mut bot = parse_h0(spec);
                        bot.arm_explain();
                        let mut r = policy_rng(seed ^ 0xABCD);
                        let re = bot.choose(&db, &state, &legal, &mut r);
                        let fuses = matches!(legal[re], Action::Fuse { .. });
                        still[si] += u32::from(fuses);
                        row.push_str(if fuses { "yes | " } else { "no | " });
                        if si == 1 && fuses {
                            let rec = bot.take_explain().expect("explain");
                            if let Some(cand) = rec.candidates.iter().find(|c| {
                                matches!(c.action, arena_engine::NeutralAction::Fuse { .. })
                            }) {
                                if let Some(w) = cand.worlds.first() {
                                    row.push_str(&format!(
                                        "raw={:.1} end={:?} pv_len={} | ",
                                        w.raw, w.end, w.pv_len
                                    ));
                                }
                                let best_other = rec
                                    .candidates
                                    .iter()
                                    .filter(|c| {
                                        !matches!(
                                            c.action,
                                            arena_engine::NeutralAction::Fuse { .. }
                                        )
                                    })
                                    .map(|c| c.root_agg)
                                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                                    .unwrap_or(0.0);
                                row.push_str(&format!("{:.1} |", best_other));
                            }
                        }
                    }
                    eprintln!("{row}");
                }
                pick
            } else {
                h0.choose(&db, &state, &legal, &mut rng)
            };
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
    }
    eprintln!(
        "decisions={} still_fuses: default=all; fusemacro={}; v0={}; v0+fusemacro={}",
        decisions, still[1], still[2], still[3]
    );
}
