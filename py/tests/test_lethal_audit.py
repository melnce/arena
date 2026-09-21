"""Forced-lethal PyO3 export and the offline audit runner."""

from __future__ import annotations

import importlib.util
import json
import sys
from collections import defaultdict
from pathlib import Path

import pytest

pytest.importorskip("arena")

_PY = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("arena_lethal_audit", _PY / "lethal_audit.py")
assert _spec and _spec.loader
audit = importlib.util.module_from_spec(_spec)
sys.modules["arena_lethal_audit"] = audit
_spec.loader.exec_module(audit)


def _decks(root: Path) -> tuple[dict[str, int], dict[str, int]]:
    forest = json.loads((root / "oracle" / "decks" / "basic-forest.json").read_text())
    rune = json.loads((root / "oracle" / "decks" / "basic-rune.json").read_text())
    return {str(k): int(v) for k, v in forest.items()}, {str(k): int(v) for k, v in rune.items()}


def test_pool_and_decks_conflict() -> None:
    with pytest.raises(SystemExit) as exc:
        audit.parse_args(["--pool", "meta", "--decks", "basic-forest"])
    assert "--pool" in str(exc.value)
    assert "--decks" in str(exc.value)


def test_default_pool_is_meta() -> None:
    args = audit.parse_args([])
    assert args.pool == "meta"
    assert args.decks is None


def test_rate_block_never_divides_by_unknowns() -> None:
    bucket = audit.empty_bucket()
    bucket["lethal"] = 2
    bucket["none"] = 8
    bucket["unknown"] = 5
    bucket["hits"] = 2
    block = audit.rate_block(bucket)
    assert block["decisions_with_a_verdict"] == 10
    assert block["rate"] == pytest.approx(0.2)
    assert block["unknown"] == 5
    assert block["unknown_rate"] == pytest.approx(5 / 15)
    assert block["asked"] == 15
    # The headline rate is n / decided, not n / (decided + unknown).
    assert block["rate"] != pytest.approx(2 / 15)


def test_bot_converts_this_turn() -> None:
    bot_win = [
        {"play": {"player": "b", "hand_pos": 0, "card": "x"}},
        {"attack": {"player": "b", "attacker_slot": 0, "target": "leader"}},
    ]
    assert audit.bot_converts_this_turn(bot_win, 0, "b", "b") is True
    ended = [
        {"play": {"player": "b", "hand_pos": 0, "card": "x"}},
        {"end_turn": {"player": "b"}},
        {"play": {"player": "a", "hand_pos": 0, "card": "y"}},
    ]
    assert audit.bot_converts_this_turn(ended, 0, "b", "a") is False
    assert audit.bot_converts_this_turn(ended, 0, "b", "b") is False


def test_forced_lethal_export_shape(db, root: Path) -> None:
    import arena

    deck_a, deck_b = _decks(root)
    game = arena.Game(db, 1, deck_a, deck_b, "a")
    # Skip both mulligans so we are on a real turn.
    for _ in range(4):
        if game.phase != "mulligan":
            break
        game.apply(game.bot_action("first-legal", 1))
    v0 = arena.forced_lethal(game, 0)
    assert v0["verdict"] in {"lethal", "none", "unknown"}
    assert "nodes" in v0
    assert "is_lethal" not in v0
    # A live turn with expandable actions and budget 0 is Unknown, not None.
    if v0["verdict"] == "unknown":
        assert v0["nodes"] == 0
    v1 = arena.forced_lethal(game, 2_000)
    assert v1["verdict"] in {"lethal", "none", "unknown"}
    if v1["verdict"] == "lethal":
        assert isinstance(v1["line"], list)
        assert "rng_dependent" in v1
    else:
        assert "line" not in v1
        assert "rng_dependent" not in v1
    v2 = arena.forced_lethal(game, 2_000)
    assert v1 == v2


def test_audit_game_counts_over_verdicts(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    rec = audit.play_one(db, 1, deck_a, deck_b, "a", "h0-fast", "h0-fast")
    assert rec["actions"], "expected a recorded prefix"
    row = audit.audit_game(db, rec, budget=80, seats=("b",))
    missed = audit.rate_block(row["missed"])
    handed = audit.rate_block(row["handed"])
    assert missed["asked"] == missed["decisions_with_a_verdict"] + missed["unknown"]
    assert handed["asked"] == handed["decisions_with_a_verdict"] + handed["unknown"]
    if missed["decisions_with_a_verdict"]:
        assert missed["rate"] == pytest.approx(
            missed["n"] / missed["decisions_with_a_verdict"]
        )
    # A boolean would have hidden Unknown. The runner must keep the split.
    assert "unknown_rate" in missed
    assert "unknown_rate" in handed


def test_decisions_flag_defaults_on() -> None:
    assert audit.parse_args([]).decisions is True
    assert audit.parse_args(["--no-decisions"]).decisions is False
    assert audit.parse_args(["--decisions"]).decisions is True
    assert audit.parse_args([]).by_deficit is False
    assert audit.parse_args(["--by-deficit"]).by_deficit is True


def test_side_state_reads_snapshot_keys() -> None:
    # Verified against game.snapshot() on a live h0-fast prefix: player life
    # is `leader_defense` (no `defense` key); field slots have no `kind`.
    player = {
        "leader_defense": 17,
        "field": [
            None,
            {
                "attack": 3,
                "defense": 2,
                "max_defense": 4,
                "countdown": None,
            },
            {
                "attack": 0,
                "defense": 0,
                "max_defense": 0,
                "countdown": None,
                "traits": ["aura"],
            },
            {
                "attack": 1,
                "defense": 1,
                "max_defense": 1,
            },
            None,
        ],
    }
    side = audit.side_state(player)
    assert side["leader_defense"] == 17
    # Earth Sigil-style 0/0 is not a follower.
    assert side["followers"] == 2
    assert side["board_attack"] == 4


def test_deficit_buckets_partition_integers() -> None:
    assert audit.deficit_bucket(-6) == "<= -5"
    assert audit.deficit_bucket(-5) == "<= -5"
    assert audit.deficit_bucket(-4) == "-4..-1"
    assert audit.deficit_bucket(-1) == "-4..-1"
    assert audit.deficit_bucket(0) == "0"
    assert audit.deficit_bucket(1) == "1..4"
    assert audit.deficit_bucket(4) == "1..4"
    assert audit.deficit_bucket(5) == ">= 5"
    assert audit.deficit_bucket(12) == ">= 5"
    assert set(audit.DEFICIT_BUCKETS) == {
        "<= -5",
        "-4..-1",
        "0",
        "1..4",
        ">= 5",
    }


def test_board_at_decision_deficit_is_opp_minus_auditee() -> None:
    snap = {
        "players": {
            "a": {
                "leader_defense": 12,
                "field": [{"attack": 2, "max_defense": 2}],
            },
            "b": {
                "leader_defense": 18,
                "field": [
                    {"attack": 5, "max_defense": 5},
                    {"attack": 1, "max_defense": 3},
                ],
            },
        }
    }
    rec = audit.board_at_decision(snap, "a")
    assert rec["auditee"] == "a"
    assert rec["defense_deficit"] == 6
    assert rec["board_attack_deficit"] == 4
    assert rec["auditee_state"]["followers"] == 1
    assert rec["opp_state"]["followers"] == 2


def test_audit_game_records_snapshot_state(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    rec = audit.play_one(db, 1, deck_a, deck_b, "a", "h0-fast", "h0-fast")
    row = audit.audit_game(db, rec, budget=80, seats=("b",))
    assert row["decisions"], "expected audited decisions"
    for d in row["decisions"]:
        assert d["auditee"] == "b"
        assert "leader_defense" in d["auditee_state"]
        assert "followers" in d["auditee_state"]
        assert "board_attack" in d["auditee_state"]
        assert d["defense_deficit"] == (
            d["opp_state"]["leader_defense"] - d["auditee_state"]["leader_defense"]
        )
        assert d["board_attack_deficit"] == (
            d["opp_state"]["board_attack"] - d["auditee_state"]["board_attack"]
        )
        if d["kind"] == "handed_lethal":
            # After EndTurn the solver is asked from the opponent's seat.
            assert d["acting"] == "a"
            assert d["auditee"] == "b"


def _rate_view(summary: dict) -> dict:
    """Rate blocks only — timing and the decisions list are not the contract."""
    records = []
    for rec in summary["records"]:
        records.append(
            {
                "game_id": rec["game_id"],
                "bot_deck": rec["bot_deck"],
                "winner": rec["winner"],
                "missed_lethal": rec["missed_lethal"],
                "handed_lethal": rec["handed_lethal"],
            }
        )
    return {
        "missed_lethal": summary["missed_lethal"],
        "handed_lethal": summary["handed_lethal"],
        "per_deck": summary["per_deck"],
        "records": records,
    }


# Captured on origin/main (af91ee0) before this change, same argv.
_MAIN_RATE_BLOCKS = {
    "handed_lethal": {
        "asked": 14,
        "decisions_with_a_verdict": 6,
        "lethal": 2,
        "n": 2,
        "nodes": 687,
        "none": 4,
        "rate": 0.3333333333333333,
        "rng_dependent": 0,
        "unknown": 8,
        "unknown_rate": 0.5714285714285714,
    },
    "missed_lethal": {
        "asked": 69,
        "decisions_with_a_verdict": 41,
        "lethal": 0,
        "n": 0,
        "nodes": 2762,
        "none": 41,
        "rate": 0.0,
        "rng_dependent": 0,
        "unknown": 28,
        "unknown_rate": 0.4057971014492754,
    },
}


def _run_audit_cli(out: Path, extra: list[str]) -> dict:
    argv = [
        "--games",
        "2",
        "--seed",
        "17",
        "--decks",
        "basic-forest",
        "basic-rune",
        "--policy",
        "h0-fast",
        "--budget",
        "80",
        "--bot-seat",
        "b",
        "--out",
        str(out),
        *extra,
    ]
    assert audit.main(argv) == 0
    return json.loads(out.read_text())


def _legacy_schedule(
    names: list[str], games: int, first_mode: str
) -> list[tuple[str, str, str]]:
    """Pre-fix schedule: alternate used the global game index ``n``."""
    pairs = [(da, dbk) for da in names for dbk in names]
    if not pairs:
        return []
    out: list[tuple[str, str, str]] = []
    for n in range(max(1, games)):
        da, dbk = pairs[n % len(pairs)]
        first = audit.first_for(first_mode, n)
        out.append((da, dbk, first))
    return out


def test_decisions_off_rate_blocks_byte_identical(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setattr(audit, "schedule", _legacy_schedule)
    off = _run_audit_cli(tmp_path / "off.json", ["--no-decisions"])
    on = _run_audit_cli(tmp_path / "on.json", ["--decisions"])
    assert "decisions" not in off["records"][0]
    assert "decisions" in on["records"][0]
    assert on["records"][0]["decisions"], "default-on records should carry decisions"
    assert _rate_view(off) == _rate_view(on)
    # Pinned against origin/main: additive, so the owner's rate blocks hold.
    assert off["missed_lethal"] == _MAIN_RATE_BLOCKS["missed_lethal"]
    assert off["handed_lethal"] == _MAIN_RATE_BLOCKS["handed_lethal"]
    record_keys = set(off["records"][0])
    assert record_keys == {
        "bot_deck",
        "deck_a",
        "deck_b",
        "first",
        "game_id",
        "handed_lethal",
        "missed_lethal",
        "seed",
        "winner",
    }


def test_both_seat_per_deck_equals_sum_of_a_and_b(tmp_path: Path) -> None:
    def run(seat: str) -> dict:
        out = tmp_path / f"{seat}.json"
        rc = audit.main(
            [
                "--games",
                "4",
                "--seed",
                "23",
                "--decks",
                "basic-forest",
                "basic-rune",
                "--policy",
                "h0-fast",
                "--budget",
                "80",
                "--bot-seat",
                seat,
                "--no-decisions",
                "--out",
                str(out),
            ]
        )
        assert rc == 0
        return json.loads(out.read_text())

    a = run("a")
    b = run("b")
    both = run("both")
    assert set(both["per_deck"]) == set(a["per_deck"]) | set(b["per_deck"])
    assert all("/" not in name for name in both["per_deck"])
    int_keys = (
        "n",
        "decisions_with_a_verdict",
        "unknown",
        "asked",
        "lethal",
        "none",
        "rng_dependent",
        "nodes",
    )
    for deck in both["per_deck"]:
        for kind in ("missed_lethal", "handed_lethal"):
            got = both["per_deck"][deck][kind]
            left = a["per_deck"].get(deck, {}).get(kind)
            right = b["per_deck"].get(deck, {}).get(kind)
            for key in int_keys:
                s = int(left[key] if left else 0) + int(right[key] if right else 0)
                assert got[key] == s, f"{deck} {kind} {key}: {got[key]} != {s}"


def _meta_names(root: Path) -> list[str]:
    _, decks = audit.resolve_selected_decks(root / "oracle" / "decks", "meta", None)
    return list(decks.keys())


def test_alternate_schedule_balanced_per_pair(root: Path) -> None:
    names = _meta_names(root)
    assert len(names) == 16
    sched = audit.schedule(names, 512, "alternate")
    assert len(sched) == 512
    pair_first: dict[tuple[str, str], list[str]] = defaultdict(list)
    for da, dbk, first in sched:
        pair_first[(da, dbk)].append(first)
    for key, firsts in pair_first.items():
        assert firsts.count("a") == 1, key
        assert firsts.count("b") == 1, key
    deck_first: dict[str, int] = defaultdict(int)
    deck_games: dict[str, int] = defaultdict(int)
    for da, dbk, first in sched:
        deck_games[da] += 1
        deck_games[dbk] += 1
        if first == "a":
            deck_first[da] += 1
        else:
            deck_first[dbk] += 1
    for deck in names:
        assert deck_first[deck] == deck_games[deck] // 2, deck


def test_legacy_alternate_schedule_unbalanced(root: Path) -> None:
    names = _meta_names(root)
    sched = _legacy_schedule(names, 512, "alternate")
    pair_first: dict[tuple[str, str], list[str]] = defaultdict(list)
    for da, dbk, first in sched:
        pair_first[(da, dbk)].append(first)
    unbalanced = sum(
        1 for firsts in pair_first.values() if firsts.count("a") != firsts.count("b")
    )
    assert unbalanced > 0


def test_first_modes_a_b_coin_unchanged(root: Path) -> None:
    names = _meta_names(root)
    for mode in ("a", "b", "coin"):
        old = _legacy_schedule(names, 600, mode)
        new = audit.schedule(names, 600, mode)
        assert old == new


def test_unbalanced_stderr_line(root: Path, capsys) -> None:
    names = _meta_names(root)
    assert audit.count_unbalanced_pairs(names, 300, "alternate") == 212
    assert audit.count_unbalanced_pairs(names, 512, "alternate") == 0
    audit.main(["--pool", "meta", "--games", "300", "--no-decisions", "--budget", "1"])
    err = capsys.readouterr().err
    assert "212 ordered pair(s) unbalanced" in err
    audit.main(["--pool", "meta", "--games", "512", "--no-decisions", "--budget", "1"])
    err = capsys.readouterr().err
    assert "unbalanced" not in err


def test_self_describing_output(tmp_path: Path) -> None:
    out = tmp_path / "desc.json"
    audit.main(
        [
            "--decks",
            "basic-forest",
            "basic-rune",
            "--games",
            "8",
            "--seed",
            "99",
            "--first",
            "alternate",
            "--policy",
            "h0-fast",
            "--budget",
            "80",
            "--no-decisions",
            "--out",
            str(out),
        ]
    )
    summary = json.loads(out.read_text())
    assert summary["seed"] == 99
    assert summary["first"] == "alternate"
    assert summary["games"] == 8
    assert summary["first_balanced_per_pair"] is True
    rec = summary["records"][0]
    assert rec["seed"] == 99
    assert rec["first"] in ("a", "b")
    assert "deck_a" in rec and "deck_b" in rec
    matchup = next(
        r
        for r in summary["records"]
        if r["deck_a"] == "basic-forest" and r["deck_b"] == "basic-rune"
    )
    assert matchup["first"] in ("a", "b")
    assert matchup["seed"] == 100

    out_a = tmp_path / "a.json"
    audit.main(
        [
            "--decks",
            "basic-forest",
            "basic-rune",
            "--games",
            "2",
            "--seed",
            "99",
            "--first",
            "a",
            "--policy",
            "h0-fast",
            "--budget",
            "80",
            "--no-decisions",
            "--out",
            str(out_a),
        ]
    )
    summary_a = json.loads(out_a.read_text())
    assert summary_a["first_balanced_per_pair"] is False


def test_resolve_first_coin_matches_fixed_modes(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    for mode in ("a", "b"):
        rec = audit.play_one(db, 5, deck_a, deck_b, mode, "h0-fast", "h0-fast")
        assert audit.resolve_first(rec) == mode
    coin_rec = audit.play_one(db, 6, deck_a, deck_b, "coin", "h0-fast", "h0-fast")
    assert audit.resolve_first(coin_rec) in ("a", "b")


def test_line_len_on_lethal_verdict(db, root: Path) -> None:
    import arena

    deck_a, deck_b = _decks(root)
    game = arena.Game(db, 1, deck_a, deck_b, "a")
    for _ in range(4):
        if game.phase != "mulligan":
            break
        game.apply(game.bot_action("first-legal", 1))
    verdict = arena.forced_lethal(game, 2_000)
    if verdict["verdict"] != "lethal":
        pytest.skip("fixture position did not return lethal at budget 2000")
    rec = audit.play_one(db, 1, deck_a, deck_b, "a", "h0-fast", "h0-fast")
    row = audit.audit_game(db, rec, budget=2_000, seats=("a", "b"))
    lethal = [
        d
        for d in row["decisions"]
        if d.get("verdict") == "lethal"
        and d.get("kind") in ("missed_lethal", "handed_lethal")
    ]
    assert lethal, "expected a lethal verdict in audited decisions"
    d = lethal[0]
    assert "line" in d
    assert d["line_len"] == len(d["line"])
    none = [x for x in row["decisions"] if x.get("verdict") == "none"]
    assert none, "expected a none verdict for contrast"
    assert "line" not in none[0]
    assert none[0]["line_len"] is None


def test_by_deficit_report_populated(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    rec = audit.play_one(db, 3, deck_a, deck_b, "a", "h0-fast", "h0-fast")
    rec["deck_a_name"] = "basic-forest"
    rec["deck_b_name"] = "basic-rune"
    rec["bot_deck"] = "basic-rune"
    row = audit.audit_game(db, rec, budget=80, seats=("b",))
    row["deck_a_name"] = "basic-forest"
    row["deck_b_name"] = "basic-rune"
    row["bot_deck"] = "basic-rune"
    report = audit.by_deficit_report([row])
    assert report["kind"] == "handed_lethal"
    assert report["on"] == "defense_deficit"
    assert report["buckets"] == list(audit.DEFICIT_BUCKETS)
    assert "basic-rune" in report["per_deck"]
    deck = report["per_deck"]["basic-rune"]
    assert deck["end_turn_decisions"] >= 1
    share_sum = sum(v or 0.0 for v in deck["bucket_share"].values())
    assert share_sum == pytest.approx(1.0)
    for label in audit.DEFICIT_BUCKETS:
        block = deck["handed_lethal"][label]
        assert "n" in block
        assert "decisions_with_a_verdict" in block
        assert "rate" in block
        assert "unknown_rate" in block
    text = audit.format_by_deficit(report)
    assert "handed_lethal by defense deficit" in text
    assert "basic-rune" in text
