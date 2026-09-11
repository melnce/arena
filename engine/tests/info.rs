//! Presentation helpers (`engine::info`) — not on the apply / legal hot path.

mod common;

use arena_engine::{board_info, hand_info, PlayerId};
use common::{give_pp, load_db, put_field, put_hand, started};

#[test]
fn depths_of_the_eld_sword_enhance_at_8_pp() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 8, 8);
    put_field(&db, &mut st, PlayerId::B, "88001110");
    let _ = put_hand(&db, &mut st, me, "90024320");
    let info = hand_info(&db, &st, me);
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].id, "90024320");
    assert_eq!(info[0].cost, Some(1));
    assert_eq!(info[0].form.as_deref(), Some("enhance"));
    let enh = info[0]
        .gates
        .iter()
        .find(|g| g.kind == "enhance")
        .expect("enhance gate");
    assert_eq!(enh.need, 1);
    assert_eq!(enh.have, 8);
    assert!(enh.met);
}

#[test]
fn hark_necromancy_progress() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 3, 3);
    let _ = put_hand(&db, &mut st, me, "10753310");

    st.player_mut(me).shadows = 4;
    let short = hand_info(&db, &st, me);
    let necro = short[0]
        .gates
        .iter()
        .find(|g| g.kind == "necromancy")
        .expect("necromancy gate");
    assert_eq!(necro.need, 6);
    assert_eq!(necro.have, 4);
    assert!(!necro.met);

    st.player_mut(me).shadows = 6;
    let ready = hand_info(&db, &st, me);
    let necro = ready[0]
        .gates
        .iter()
        .find(|g| g.kind == "necromancy")
        .expect("necromancy gate");
    assert_eq!(necro.need, 6);
    assert_eq!(necro.have, 6);
    assert!(necro.met);
}

#[test]
fn rally_twenty_at_twelve() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).rally = 12;
    let _ = put_hand(&db, &mut st, me, "10824110");
    let info = hand_info(&db, &st, me);
    let rally = info[0]
        .gates
        .iter()
        .find(|g| g.kind == "rally")
        .expect("rally gate");
    assert_eq!(rally.need, 20);
    assert_eq!(rally.have, 12);
    assert!(!rally.met);
}

#[test]
fn board_info_occupied_slots_only() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001120");
    let info = board_info(&db, &st, me);
    assert_eq!(info.len(), 2);
    assert_eq!(info[0].slot, 0);
    assert_eq!(info[1].slot, 1);
    assert_eq!(info[0].id, "88001110");
}
