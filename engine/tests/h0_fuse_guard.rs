//! H0 fuseguard: no-op fuse predicate and opt-in filtering.

use arena_engine::{fuse_is_noop, AnyPolicy, H0};

mod common;
use common::{cid, load_db};

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

#[test]
fn fuse_is_noop_on_card_data() {
    let db = load_db();
    let sephie = cid("10934110");
    let ecstatic = cid("10933110");
    let gear = cid("90071210");
    assert!(fuse_is_noop(&db, sephie, 0, 8));
    assert!(fuse_is_noop(&db, sephie, 1, 8));
    assert!(!fuse_is_noop(&db, sephie, 2, 8));
    assert!(!fuse_is_noop(&db, sephie, 0, 9));
    assert!(!fuse_is_noop(&db, ecstatic, 0, 8));
    assert!(!fuse_is_noop(&db, gear, 0, 8));
}

#[test]
fn fuseguard_parses_and_round_trips() {
    let h = parse_h0("h0:fuseguard=1");
    assert!(h.fuseguard);
    assert_eq!(AnyPolicy::H0(h).spec(), "h0:fuseguard=1");
    assert!(!parse_h0("h0:fuseguard=0").fuseguard);
    assert!(AnyPolicy::parse_spec("h0:fuseguard=2").is_err());
}
