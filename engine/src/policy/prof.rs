//! Off-by-default wall-time and duplicate-lookup counters for H0 (`h0-prof` feature).

use std::collections::hash_map::Entry;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::fxhash::FxMap;

/// Wall-time bucket (one per decision, reset in [`reset_decision`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Zone {
    WorldSetup = 0,
    OwnSearch = 1,
    LeafEncode = 2,
    LeafForward = 3,
    OrderNet = 4,
    OppLethal = 5,
    Oopen = 6,
    Holdback = 7,
    Tkill = 8,
    Tkroll = 9,
    Fuseguard = 10,
}

pub const ZONE_COUNT: usize = 11;

pub static ZONE_NAMES: [&str; ZONE_COUNT] = [
    "world_setup",
    "own_search",
    "leaf_encode",
    "leaf_forward",
    "order_net",
    "opp_lethal",
    "oopen",
    "holdback",
    "tkill",
    "tkroll",
    "fuseguard",
];

/// Duplicate probe kinds (within one decision).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DupKind {
    LeafValue = 0,
    LeafEncode = 1,
    OppLethal = 2,
    Oopen = 3,
}

pub const DUP_KIND_COUNT: usize = 4;

pub static DUP_KIND_NAMES: [&str; DUP_KIND_COUNT] =
    ["leaf_value", "leaf_encode", "opp_lethal", "oopen"];

struct ZoneAcc {
    ns: AtomicU64,
    calls: AtomicU64,
}

impl ZoneAcc {
    const fn new() -> Self {
        Self {
            ns: AtomicU64::new(0),
            calls: AtomicU64::new(0),
        }
    }

    fn add(&self, d: Duration) {
        self.ns.fetch_add(d.as_nanos() as u64, Ordering::Relaxed);
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
}

struct DupAcc {
    lookups: AtomicU64,
    repeats: AtomicU64,
}

impl DupAcc {
    const fn new() -> Self {
        Self {
            lookups: AtomicU64::new(0),
            repeats: AtomicU64::new(0),
        }
    }
}

struct ProfInner {
    zones: [ZoneAcc; ZONE_COUNT],
    dup_keys: Mutex<FxMap<(u8, u64), ()>>,
    dups: [DupAcc; DUP_KIND_COUNT],
}

impl ProfInner {
    fn new() -> Self {
        Self {
            zones: std::array::from_fn(|_| ZoneAcc::new()),
            dup_keys: Mutex::new(FxMap::default()),
            dups: std::array::from_fn(|_| DupAcc::new()),
        }
    }

    fn reset(&self) {
        for z in &self.zones {
            z.ns.store(0, Ordering::Relaxed);
            z.calls.store(0, Ordering::Relaxed);
        }
        for d in &self.dups {
            d.lookups.store(0, Ordering::Relaxed);
            d.repeats.store(0, Ordering::Relaxed);
        }
        self.dup_keys.lock().unwrap().clear();
    }
}

static PROF: OnceLock<ProfInner> = OnceLock::new();

fn prof() -> &'static ProfInner {
    PROF.get_or_init(ProfInner::new)
}

pub struct ZoneGuard {
    zone: Zone,
    start: Instant,
}

impl ZoneGuard {
    #[inline]
    pub fn enter(zone: Zone) -> Self {
        Self {
            zone,
            start: Instant::now(),
        }
    }
}

impl Drop for ZoneGuard {
    fn drop(&mut self) {
        prof().zones[self.zone as usize].add(self.start.elapsed());
    }
}

#[inline]
pub fn reset_decision() {
    prof().reset();
}

/// Record a lookup; returns `true` when this key was seen before in this decision.
#[inline]
pub fn dup_probe(kind: DupKind, key: impl FnOnce() -> u64) -> bool {
    let key = key();
    let tag = kind as u8;
    let p = prof();
    let d = &p.dups[kind as usize];
    d.lookups.fetch_add(1, Ordering::Relaxed);
    let mut map = p.dup_keys.lock().unwrap();
    match map.entry((tag, key)) {
        Entry::Occupied(_) => {
            d.repeats.fetch_add(1, Ordering::Relaxed);
            true
        }
        Entry::Vacant(e) => {
            e.insert(());
            false
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ZoneRow {
    pub name: &'static str,
    pub ns: u64,
    pub calls: u64,
    pub share: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DupRow {
    pub name: &'static str,
    pub lookups: u64,
    pub repeats: u64,
    pub repeat_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DecisionProf {
    pub zones: Vec<ZoneRow>,
    pub dups: Vec<DupRow>,
    pub wall_ns: u64,
}

pub fn take_decision(wall: Duration) -> DecisionProf {
    let p = prof();
    let wall_ns = wall.as_nanos() as u64;
    let mut zone_ns = [0u64; ZONE_COUNT];
    let mut zone_calls = [0u64; ZONE_COUNT];
    for (i, z) in p.zones.iter().enumerate() {
        zone_ns[i] = z.ns.load(Ordering::Relaxed);
        zone_calls[i] = z.calls.load(Ordering::Relaxed);
    }
    let denom = wall_ns.max(1) as f64;
    let zones: Vec<ZoneRow> = (0..ZONE_COUNT)
        .map(|i| ZoneRow {
            name: ZONE_NAMES[i],
            ns: zone_ns[i],
            calls: zone_calls[i],
            share: zone_ns[i] as f64 / denom,
        })
        .collect();
    let dups: Vec<DupRow> = (0..DUP_KIND_COUNT)
        .map(|i| {
            let lookups = p.dups[i].lookups.load(Ordering::Relaxed);
            let repeats = p.dups[i].repeats.load(Ordering::Relaxed);
            DupRow {
                name: DUP_KIND_NAMES[i],
                lookups,
                repeats,
                repeat_rate: if lookups > 0 {
                    repeats as f64 / lookups as f64
                } else {
                    0.0
                },
            }
        })
        .collect();
    DecisionProf {
        zones,
        dups,
        wall_ns,
    }
}

pub fn merge_decisions(rows: &[DecisionProf]) -> DecisionProf {
    let wall_ns: u64 = rows.iter().map(|r| r.wall_ns).sum();
    let mut zone_ns = [0u64; ZONE_COUNT];
    let mut zone_calls = [0u64; ZONE_COUNT];
    let mut dup_lookups = [0u64; DUP_KIND_COUNT];
    let mut dup_repeats = [0u64; DUP_KIND_COUNT];
    for r in rows {
        for (i, z) in r.zones.iter().enumerate() {
            zone_ns[i] += z.ns;
            zone_calls[i] += z.calls;
        }
        for (i, d) in r.dups.iter().enumerate() {
            dup_lookups[i] += d.lookups;
            dup_repeats[i] += d.repeats;
        }
    }
    let denom = wall_ns.max(1) as f64;
    let zones: Vec<ZoneRow> = (0..ZONE_COUNT)
        .map(|i| ZoneRow {
            name: ZONE_NAMES[i],
            ns: zone_ns[i],
            calls: zone_calls[i],
            share: zone_ns[i] as f64 / denom,
        })
        .collect();
    let dups: Vec<DupRow> = (0..DUP_KIND_COUNT)
        .map(|i| DupRow {
            name: DUP_KIND_NAMES[i],
            lookups: dup_lookups[i],
            repeats: dup_repeats[i],
            repeat_rate: if dup_lookups[i] > 0 {
                dup_repeats[i] as f64 / dup_lookups[i] as f64
            } else {
                0.0
            },
        })
        .collect();
    DecisionProf {
        zones,
        dups,
        wall_ns,
    }
}
