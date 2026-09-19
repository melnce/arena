#!/usr/bin/env python3
"""Fetch the current Rotation meta from WBArts and write representative decks.

Re-runnable whenever the meta shifts. Idempotent: unchanged files are left
alone; stale ``meta-*.json`` files from a previous run are removed.

**Legality is ``is_include_rotation``, not ``latest_pack_id``.** WBArts'
``latest_pack_id`` is only the newest set among a deck's own cards. Filtering
on it silently drops current decks built from older sets. The 2026-09-10
snapshot did exactly that and hid Crystal Rune (30 of its 33 lists) and half
of Rally Sword. A list is legal iff every card is in the Cygames catalog and
every card has ``is_include_rotation`` true.

Sources (≤ 2 requests/second, User-Agent set):

- decklists: ``https://sva.hypd.asia/api/decks?format=rotation&sort=new&days=60``
- archetype names: ``https://sva.hypd.asia/api/archetypes``
- card catalog: ``https://shadowverse-wb.com/web/CardList/cardList`` (``Lang: en``)

Window starts 2026-08-27 JST (Revenants of Azvaldt). A list with no card from
set 10009 counts only from the day after release.

Archetype labels are WBArts' own ``local:<id>``, with a child merged into its
``parent_archetype_id`` when one is set. Unlabelled lists are not classified.

Each archetype with ≥ 10 qualifying lists contributes one *real posted list*:
the list with the highest weighted Jaccard similarity to the average list
(mean copies per card over the most recent half of the window when that half
has ≥ 15 lists, else the whole window), tie-breaking on rating.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
import urllib.error
import urllib.request
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any
from zoneinfo import ZoneInfo

def repo_root() -> Path:
    here = Path(__file__).resolve().parent
    for d in (here, *here.parents):
        if (d / "oracle" / "decks").is_dir() and (d / "cards").is_dir():
            return d
    return Path.cwd()

USER_AGENT = "arena-meta-decks/1.0 (+https://github.com/melnce/arena)"
WBA_DECKS = "https://sva.hypd.asia/api/decks?format=rotation&sort=new&days=60&offset={offset}"
WBA_ARCHETYPES = "https://sva.hypd.asia/api/archetypes"
CATALOG = "https://shadowverse-wb.com/web/CardList/cardList?offset={offset}&include_token=1"

JST = ZoneInfo("Asia/Tokyo")
WINDOW_START = datetime(2026, 8, 27, 0, 0, 0, tzinfo=JST)
RELEASE_NEXT_DAY = datetime(2026, 8, 28, 0, 0, 0, tzinfo=JST)
NEW_SET_ID = 10009
MIN_GAP_S = 0.5
ARCHETYPE_FLOOR = 10
RECENT_HALF_MIN = 15

CLASS_SLUG = {
    1: "forest",
    2: "sword",
    3: "rune",
    4: "dragon",
    5: "abyss",
    6: "haven",
    7: "portal",
}

# Filename slugs only, when WBArts has no English name. Provenance still
# records the empty EN field and the JP/CHS names as returned.
SLUG_WHEN_NO_EN = {
    18: "insect",
    35: "enhance",
    37: "ho-chan",
    38: "discard",
    39: "aggro",
    40: "midrange",
    41: "midrange",
    42: "otk",
    43: "almes",
    44: "storm",
}

REAL_STEMS = [
    "abyss-p8rfn",
    "afnm-minatodao",
    "elf-neanisu2",
    "ramp-37772",
    "ramp-claywies",
    "royal-nattui",
    "rune-mach15",
]
SYNTHETIC_STEMS = [
    "abyss-pool",
    "basic-forest",
    "basic-portal",
    "basic-rune",
    "dragon-pool",
    "forest-pool",
    "portal-pool",
    "rune-pool",
    "sword-pool",
]

SIDECAR_STEMS = frozenset({"POOLS", "meta-pool"})


class FetchError(SystemExit):
    """Network refused a required host — do not synthesise lists."""


def _headers(extra: dict[str, str] | None = None) -> dict[str, str]:
    h = {"User-Agent": USER_AGENT, "Accept": "application/json"}
    if extra:
        h.update(extra)
    return h


class RateLimiter:
    def __init__(self, min_gap: float = MIN_GAP_S) -> None:
        self.min_gap = min_gap
        self._last = 0.0

    def wait(self) -> None:
        now = time.monotonic()
        gap = self.min_gap - (now - self._last)
        if self._last and gap > 0:
            time.sleep(gap)
        self._last = time.monotonic()


def fetch_json(url: str, headers: dict[str, str], limiter: RateLimiter) -> Any:
    limiter.wait()
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            raw = resp.read()
            status = resp.status
    except urllib.error.HTTPError as e:
        body = e.read()[:200]
        raise FetchError(
            f"GET {url} → HTTP {e.code}; body starts: {body!r}. "
            f"Network refused this host; not synthesising decklists."
        ) from e
    except urllib.error.URLError as e:
        raise FetchError(
            f"GET {url} failed: {e}. "
            f"Network refused this host; not synthesising decklists."
        ) from e
    if status != 200:
        raise FetchError(
            f"GET {url} → HTTP {status}. "
            f"Network refused this host; not synthesising decklists."
        )
    try:
        return json.loads(raw.decode("utf-8"))
    except json.JSONDecodeError as e:
        raise FetchError(
            f"GET {url} returned non-JSON ({raw[:80]!r}). "
            f"Network refused this host; not synthesising decklists."
        ) from e


def fetch_catalog(limiter: RateLimiter) -> dict[str, dict[str, Any]]:
    """card_id → {name, class, set, rotation} from the live Cygames list."""
    out: dict[str, dict[str, Any]] = {}
    offset = 0
    count: int | None = None
    pages = 0
    while True:
        url = CATALOG.format(offset=offset)
        body = fetch_json(url, _headers({"Lang": "en"}), limiter)
        data = body.get("data") or {}
        if count is None:
            count = int(data.get("count") or 0)
        page = data.get("sort_card_id_list") or []
        details = data.get("card_details") or {}
        for cid, det in details.items():
            common = (det or {}).get("common") or {}
            card_id = str(common.get("card_id") or cid)
            out[card_id] = {
                "name": str(common.get("name") or ""),
                "class": int(common.get("class") or 0),
                "set": int(common.get("card_set_id") or 0),
                "rotation": bool(common.get("is_include_rotation")),
            }
        pages += 1
        offset += len(page)
        print(f"catalog page {pages}: offset={offset}/{count}", flush=True)
        if not page or (count is not None and offset >= count):
            break
    print(f"catalog: {len(out)} cards (api count={count})", flush=True)
    return out


def fetch_archetypes(limiter: RateLimiter) -> dict[int, dict[str, Any]]:
    body = fetch_json(WBA_ARCHETYPES, _headers(), limiter)
    rows = body.get("archetypes") if isinstance(body, dict) else body
    out: dict[int, dict[str, Any]] = {}
    for row in rows or []:
        out[int(row["id"])] = row
    print(f"archetypes: {len(out)}", flush=True)
    return out


def parse_posted_at(raw: str) -> datetime:
    text = raw.replace("Z", "+00:00")
    dt = datetime.fromisoformat(text)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(JST)


def fetch_decks(limiter: RateLimiter) -> list[dict[str, Any]]:
    """Page newest-first until posts are older than the window start."""
    out: list[dict[str, Any]] = []
    offset = 0
    pages = 0
    while True:
        url = WBA_DECKS.format(offset=offset)
        body = fetch_json(url, _headers(), limiter)
        page = body.get("decks") if isinstance(body, dict) else body
        if not page:
            break
        pages += 1
        stop = False
        kept = 0
        for row in page:
            posted_raw = row.get("posted_at") or row.get("created_at") or ""
            try:
                posted = parse_posted_at(str(posted_raw))
            except (TypeError, ValueError):
                continue
            if posted < WINDOW_START:
                stop = True
                continue
            row["_posted_jst"] = posted
            out.append(row)
            kept += 1
        print(
            f"decks page {pages}: offset={offset} kept={kept} total={len(out)}",
            flush=True,
        )
        if stop:
            break
        offset += len(page)
    print(f"decks: {len(out)} posts in window", flush=True)
    return out


def deck_cards(row: dict[str, Any]) -> dict[str, int] | None:
    raw = row.get("cards")
    if not isinstance(raw, dict):
        return None
    out: dict[str, int] = {}
    for k, n in raw.items():
        try:
            count = int(n)
        except (TypeError, ValueError):
            return None
        if count <= 0:
            return None
        out[str(k)] = count
    return out


def local_archetype_id(label: object) -> int | None:
    if not isinstance(label, str) or not label.startswith("local:"):
        return None
    tail = label.split(":", 1)[1]
    if not tail.isdigit():
        return None
    return int(tail)


def merge_archetype_id(aid: int, archetypes: dict[int, dict[str, Any]]) -> int:
    rec = archetypes.get(aid) or {}
    parent = rec.get("parent_archetype_id") or 0
    try:
        parent_id = int(parent)
    except (TypeError, ValueError):
        return aid
    if parent_id and parent_id in archetypes:
        return parent_id
    return aid


def name_en(rec: dict[str, Any]) -> str:
    return str(rec.get("resolved_name_eng") or rec.get("name_eng") or "").strip()


def name_jp(rec: dict[str, Any]) -> str:
    return str(rec.get("resolved_name_jpn") or rec.get("name_jpn") or "").strip()


def slugify(text: str) -> str:
    s = text.lower().strip()
    s = re.sub(r"[^a-z0-9]+", "-", s)
    return s.strip("-")


def archetype_slug(aid: int, rec: dict[str, Any], class_id: int) -> str:
    en = name_en(rec)
    if en:
        slug = slugify(en)
        class_word = CLASS_SLUG.get(class_id, "")
        if class_word and slug.endswith("-" + class_word):
            slug = slug[: -len(class_word) - 1]
        if slug:
            return slug
    return SLUG_WHEN_NO_EN.get(aid) or f"local-{aid}"


def has_new_set(cards: dict[str, int], catalog: dict[str, dict[str, Any]]) -> bool:
    return any(catalog.get(cid, {}).get("set") == NEW_SET_ID for cid in cards)


def qualify(
    row: dict[str, Any],
    catalog: dict[str, dict[str, Any]],
) -> dict[str, int] | None:
    """Return the 40-card map if the list survives every load-bearing filter."""
    if row.get("source") == "user":
        return None
    cards = deck_cards(row)
    if cards is None:
        return None
    class_id = int(row.get("class_id") or 0)
    posted: datetime = row["_posted_jst"]
    if posted < WINDOW_START:
        return None
    if not has_new_set(cards, catalog) and posted < RELEASE_NEXT_DAY:
        return None
    if sum(cards.values()) != 40:
        return None
    for cid, _n in cards.items():
        rec = catalog.get(cid)
        if rec is None:
            return None
        if not rec["rotation"]:
            return None
        if rec["class"] not in (0, class_id):
            return None
    return cards


def mean_copies(lists: list[dict[str, int]]) -> dict[str, float]:
    n = len(lists)
    totals: dict[str, float] = defaultdict(float)
    for cards in lists:
        for cid, c in cards.items():
            totals[cid] += c
    return {cid: tot / n for cid, tot in totals.items()}


def weighted_jaccard(avg: dict[str, float], cards: dict[str, int]) -> float:
    keys = set(avg) | set(cards)
    num = 0.0
    den = 0.0
    for k in keys:
        a = float(avg.get(k, 0.0))
        b = float(cards.get(k, 0))
        num += min(a, b)
        den += max(a, b)
    return num / den if den else 0.0


def discrete_aggregate(avg: dict[str, float]) -> dict[str, int]:
    """Round mean copies to a 40-card multiset (report only; not the deck)."""
    ranked = sorted(avg.items(), key=lambda kv: (-kv[1], kv[0]))
    out: dict[str, int] = {}
    left = 40
    for cid, mean in ranked:
        n = int(round(mean))
        if n <= 0:
            continue
        n = min(n, left, 3)
        if n <= 0:
            continue
        out[cid] = n
        left -= n
        if left <= 0:
            break
    return out


def recent_half_cutoff(now: datetime) -> datetime:
    delta = now - WINDOW_START
    return WINDOW_START + delta / 2


def pick_representative(
    rows: list[dict[str, Any]],
    cutoff: datetime,
) -> tuple[dict[str, Any], float, float]:
    recent = [r for r in rows if r["_posted_jst"] >= cutoff]
    source = recent if len(recent) >= RECENT_HALF_MIN else rows
    avg = mean_copies([r["_cards"] for r in source])
    agg = discrete_aggregate(avg)

    def key(r: dict[str, Any]) -> tuple[float, float, str]:
        sim = weighted_jaccard(avg, r["_cards"])
        rating = float(r.get("rating") or 0)
        return (sim, rating, str(r.get("id") or ""))

    best = max(rows, key=key)
    sim_mean = weighted_jaccard(avg, best["_cards"])
    sim_agg = weighted_jaccard({k: float(v) for k, v in agg.items()}, best["_cards"])
    return best, sim_mean, sim_agg


def authored_ids(repo: Path) -> dict[str, Path]:
    out: dict[str, Path] = {}
    cards = repo / "cards"
    for path in cards.rglob("*.json"):
        if "official" in path.parts:
            continue
        out[path.stem] = path
    return out


def missing_authored(
    cards: dict[str, int],
    files: dict[str, Path],
    catalog: dict[str, dict[str, Any]],
) -> list[tuple[str, str]]:
    miss: list[tuple[str, str]] = []
    for cid in cards:
        if cid not in files:
            miss.append((cid, str((catalog.get(cid) or {}).get("name") or "")))
    return miss


def write_json(path: Path, payload: Any) -> str:
    text = json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
    if path.is_file() and path.read_text(encoding="utf-8") == text:
        return "unchanged"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return "wrote"


def deck_to_json(cards: dict[str, int]) -> dict[str, int]:
    return {cid: cards[cid] for cid in sorted(cards, key=lambda x: (len(x), x))}


def now_jst() -> datetime:
    return datetime.now(JST)


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def run(repo: Path) -> int:
    generated_at = utc_now_iso()
    now = now_jst()
    cutoff = recent_half_cutoff(now)
    limiter = RateLimiter()

    print(f"window: {WINDOW_START.date()} JST → {now.date()} JST", flush=True)
    print(f"recent-half cutoff: {cutoff.isoformat()}", flush=True)

    catalog = fetch_catalog(limiter)
    archetypes = fetch_archetypes(limiter)
    posts = fetch_decks(limiter)

    qualifying: list[dict[str, Any]] = []
    for row in posts:
        cards = qualify(row, catalog)
        if cards is None:
            continue
        row["_cards"] = cards
        qualifying.append(row)

    labelled: dict[int, list[dict[str, Any]]] = defaultdict(list)
    unlabelled = 0
    for row in qualifying:
        aid = local_archetype_id(row.get("archetype"))
        if aid is None or aid not in archetypes:
            unlabelled += 1
            continue
        labelled[merge_archetype_id(aid, archetypes)].append(row)

    counts = {aid: len(rows) for aid, rows in labelled.items()}
    selected_ids = sorted(
        (aid for aid, n in counts.items() if n >= ARCHETYPE_FLOOR),
        key=lambda a: (-counts[a], a),
    )
    missed = sorted(
        ((aid, n) for aid, n in counts.items() if n < ARCHETYPE_FLOOR),
        key=lambda t: (-t[1], t[0]),
    )

    print(f"qualifying lists: {len(qualifying)}", flush=True)
    print(f"unlabelled (no local: id / unknown id): {unlabelled}", flush=True)
    print(
        f"merged archetypes ≥ {ARCHETYPE_FLOOR}: {len(selected_ids)} "
        f"(of {len(counts)} labelled)",
        flush=True,
    )
    if missed:
        preview = missed[:8]
        bits = []
        for aid, n in preview:
            rec = archetypes[aid]
            bits.append(
                f"{name_en(rec) or name_jp(rec) or aid} n={n}"
            )
        print(f"just missed: {', '.join(bits)}", flush=True)

    files = authored_ids(repo)
    decks_dir = repo / "oracle" / "decks"
    n_qual = max(len(qualifying), 1)
    provenance: dict[str, Any] = {
        "generated_at": generated_at,
        "window": {
            "start": WINDOW_START.isoformat(),
            "end": now.isoformat(),
            "tz": "Asia/Tokyo",
        },
        "unlabelled": unlabelled,
        "qualifying": len(qualifying),
        "floor": ARCHETYPE_FLOOR,
        "just_missed": [
            {
                "archetype_id": aid,
                "name_en": name_en(archetypes[aid]),
                "name_jp": name_jp(archetypes[aid]),
                "count": n,
            }
            for aid, n in missed[:8]
        ],
        "skipped_coverage": [],
        "decks": {},
    }

    wanted_stems: set[str] = set()
    used_slugs: Counter[str] = Counter()
    for aid in selected_ids:
        rows = labelled[aid]
        rec = archetypes[aid]
        class_id = int(rec.get("class_id") or rows[0].get("class_id") or 0)
        class_slug = CLASS_SLUG.get(class_id, f"class{class_id}")
        slug = archetype_slug(aid, rec, class_id)
        used_slugs[f"{class_slug}-{slug}"] += 1
        if used_slugs[f"{class_slug}-{slug}"] > 1:
            slug = f"{slug}-{aid}"
        stem = f"meta-{class_slug}-{slug}"
        best, sim_mean, sim_agg = pick_representative(rows, cutoff)
        miss = missing_authored(best["_cards"], files, catalog)
        en = name_en(rec)
        jp = name_jp(rec)
        if miss:
            entry = {
                "archetype_id": aid,
                "name_en": en,
                "name_jp": jp,
                "stem": stem,
                "missing": [{"id": cid, "name": nm} for cid, nm in miss],
            }
            provenance["skipped_coverage"].append(entry)
            print(
                f"skipped {stem} ({en or jp}): missing "
                + ", ".join(f"{cid} {nm}" for cid, nm in miss),
                flush=True,
            )
            continue
        wanted_stems.add(stem)
        path = decks_dir / f"{stem}.json"
        action = write_json(path, deck_to_json(best["_cards"]))
        posted = best["_posted_jst"]
        rec_deck = {
            "archetype_id": aid,
            "name_en": en,
            "name_jp": jp,
            "class": class_slug,
            "class_id": class_id,
            "list_count": len(rows),
            "share": round(len(rows) / n_qual, 6),
            "wbarts_id": best.get("id"),
            "source_url": best.get("source_url") or "",
            "posted_at_jst": posted.isoformat(),
            "date_jst": posted.date().isoformat(),
            "rating": best.get("rating"),
            "streak": best.get("win_count"),
            "similarity_to_average": round(sim_mean, 6),
            "similarity_to_aggregate": round(sim_agg, 6),
        }
        provenance["decks"][stem] = rec_deck
        print(
            f"{action:9} {stem}.json  n={len(rows)} share={rec_deck['share']:.3f} "
            f"sim={sim_mean:.3f} id={best.get('id')} {best.get('source_url')}",
            flush=True,
        )

    stale = []
    for path in sorted(decks_dir.glob("meta-*.json")):
        if path.stem == "meta-pool":
            continue
        if path.stem not in wanted_stems:
            path.unlink()
            stale.append(path.name)
            print(f"removed   {path.name} (no longer selected)", flush=True)

    meta_stems = sorted(wanted_stems)
    pools = {
        "meta": meta_stems,
        "real": list(REAL_STEMS),
        "synthetic": list(SYNTHETIC_STEMS),
        "all": sorted(set(meta_stems) | set(REAL_STEMS) | set(SYNTHETIC_STEMS)),
    }
    action = write_json(decks_dir / "POOLS.json", pools)
    print(f"{action:9} POOLS.json  meta={len(meta_stems)} all={len(pools['all'])}", flush=True)
    action = write_json(decks_dir / "meta-pool.json", provenance)
    print(f"{action:9} meta-pool.json", flush=True)

    print(
        f"done: {len(meta_stems)} meta decks, "
        f"{len(provenance['skipped_coverage'])} skipped for coverage, "
        f"{unlabelled} unlabelled, "
        f"{len(stale)} stale removed",
        flush=True,
    )
    return 0


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--repo",
        default=None,
        help="repo root (default: walk from this file)",
    )
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    repo = Path(args.repo).resolve() if args.repo else repo_root()
    return run(repo)


if __name__ == "__main__":
    raise SystemExit(main())
