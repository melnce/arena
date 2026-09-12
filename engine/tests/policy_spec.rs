//! Policy spec parser: strict tokens, H0 knobs, canonical `spec()`.

use arena_engine::{AnyPolicy, H0};

#[test]
fn parse_h0_params_match_fast_fields() {
    let AnyPolicy::H0(got) = AnyPolicy::parse_spec("h0:depth=2,beam=2,k=1,nodes=80").unwrap()
    else {
        panic!("expected H0");
    };
    let want = H0::fast();
    assert_eq!(got.depth, want.depth);
    assert_eq!(got.beam, want.beam);
    assert_eq!(got.node_cap, want.node_cap);
    // `H0::fast()` stores `determinizations = 0`; `k()` is `max(1, …)`.
    // The spec writes `k=1`, which is that same K.
    assert_eq!(
        got.determinizations.max(1),
        want.determinizations.max(1),
        "k / determinizations"
    );
    assert_eq!(got.depth, 2);
    assert_eq!(got.beam, 2);
    assert_eq!(got.node_cap, 80);
}

#[test]
fn spec_round_trips_five_forms() {
    for s in [
        "random",
        "first-legal",
        "h0",
        "h0-fast",
        "h0:depth=6,beam=4,k=4,nodes=2000",
    ] {
        let parsed = AnyPolicy::parse_spec(s).unwrap_or_else(|e| panic!("{s}: {e}"));
        let again = AnyPolicy::parse_spec(&parsed.spec())
            .unwrap_or_else(|e| panic!("{}: {e}", parsed.spec()));
        assert_eq!(parsed, again, "{s} → {}", parsed.spec());
        assert_eq!(
            AnyPolicy::parse_spec(&again.spec()).unwrap(),
            parsed,
            "second hop {s}"
        );
    }
}

#[test]
fn parse_spec_errors_name_the_token() {
    let e = AnyPolicy::parse_spec("h0:depht=2").unwrap_err();
    assert!(e.contains("depht"), "{e}");
    let e = AnyPolicy::parse_spec("h0:depth=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
    let e = AnyPolicy::parse_spec("hzero").unwrap_err();
    assert!(e.contains("hzero"), "{e}");
}

#[test]
fn parse_no_longer_falls_back_to_random() {
    let err = std::panic::catch_unwind(|| AnyPolicy::parse("hzero"));
    assert!(err.is_err(), "parse must panic on unknown names");
}
