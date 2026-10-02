//! Bot visibility of artifact enter counts: determinize, search_key, H0 explain.

use arena_engine::determinize::{determinize_with, Info};
use arena_engine::policy::Policy;
use arena_engine::{hash, legal_actions, policy_rng, search_key, Action, AnyPolicy, PlayerId, H0};

mod common;
use common::{cid, give_pp, load_db, put_field, put_hand, started};

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn warp_slash_base(db: &arena_engine::CardDb) -> (arena_engine::State, u8) {
    let mut st = started(db, 20261002);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(db, &mut st, opp, "88001110");
    put_field(db, &mut st, opp, "88001120");
    for slot in 0..2 {
        if let Some(f) = st.field_inst_mut(opp, slot) {
            f.defense = 3;
            f.max_defense = 3;
        }
    }
    give_pp(&mut st, me, 3, 10);
    st.player_mut(me).hand.clear();
    let hand = put_hand(db, &mut st, me, "10773310");
    (st, hand)
}

fn with_artifact_k(st: &arena_engine::State, artifact_k: usize) -> arena_engine::State {
    let mut out = st.clone();
    let me = PlayerId::A;
    out.player_mut(me).enter_counts.clear();
    const IDS: [&str; 3] = ["90071130", "90071140", "90071150"];
    for id in IDS.iter().take(artifact_k.min(IDS.len())) {
        out.player_mut(me).enter_counts.insert(cid(id), 1);
    }
    out
}

#[test]
fn determinize_preserves_enter_counts_all_info_modes() {
    let db = load_db();
    let (base, _) = warp_slash_base(&db);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let mut st = with_artifact_k(&base, 3);
    st.player_mut(opp)
        .enter_counts
        .insert(cid("90071160"), 1);
    st.player_mut(opp)
        .enter_counts
        .insert(cid("90073110"), 2);
    for info in [Info::Open, Info::Fair, Info::Draws, Info::All] {
        for seed in [1u64, 99, 4242] {
            let d = determinize_with(&st, me, seed, info);
            assert_eq!(
                d.player(me).enter_counts,
                st.player(me).enter_counts,
                "{info:?} seed={seed} actor enter_counts"
            );
            assert_eq!(
                d.player(opp).enter_counts,
                st.player(opp).enter_counts,
                "{info:?} seed={seed} opponent enter_counts"
            );
        }
    }
}

#[test]
fn search_key_differs_on_enter_counts() {
    let db = load_db();
    let (base, _) = warp_slash_base(&db);
    let a = with_artifact_k(&base, 0);
    let b = with_artifact_k(&base, 3);
    assert_eq!(hash(&a), hash(&base), "k=0 differs only in enter_counts");
    assert_eq!(search_key(&a), search_key(&base));
    assert_ne!(search_key(&a), search_key(&b));
}

#[test]
fn h0_values_warp_slash_higher_with_three_artifacts() {
    let db = load_db();
    let (base, warp_hand) = warp_slash_base(&db);
    let st0 = with_artifact_k(&base, 0);
    let st3 = with_artifact_k(&base, 3);
    assert_eq!(hash(&st0), hash(&base));
    assert_eq!(search_key(&st0), search_key(&base));
    let legal0 = legal_actions(&db, &st0);
    let legal3 = legal_actions(&db, &st3);
    let play0 = legal0
        .iter()
        .position(|a| matches!(a, Action::Play { hand } if *hand == warp_hand))
        .expect("play warp slash legal at 0");
    let play3 = legal3
        .iter()
        .position(|a| matches!(a, Action::Play { hand } if *hand == warp_hand))
        .expect("play warp slash legal at 3");

    let seed = 20261002u64;
    let mut h0 = parse_h0("h0");
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let chosen3 = h0.choose(&db, &st3, &legal3, &mut rng);
    let rec3 = h0.take_explain().expect("explain at 3");
    let cand3 = rec3
        .candidates
        .iter()
        .find(|c| c.legal_index == play3)
        .expect("warp slash candidate at 3");
    let val3 = cand3.root_agg;

    let mut h0 = parse_h0("h0");
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let chosen0 = h0.choose(&db, &st0, &legal0, &mut rng);
    let rec0 = h0.take_explain().expect("explain at 0");
    let cand0 = rec0
        .candidates
        .iter()
        .find(|c| c.legal_index == play0)
        .expect("warp slash candidate at 0");
    let val0 = cand0.root_agg;

    eprintln!(
        "artifacts=3: chosen={chosen3} play_warp_root={val3:.4}; artifacts=0: chosen={chosen0} play_warp_root={val0:.4}"
    );
    assert!(
        val3 > val0,
        "play Warp Slash root value at 3 artifacts ({val3}) should exceed 0 ({val0})"
    );
}
