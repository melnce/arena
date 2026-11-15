//! No-op stubs when `h0-prof` is disabled.

#![allow(dead_code)]

use std::time::Duration;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DupKind {
    LeafValue = 0,
    LeafEncode = 1,
    OppLethal = 2,
    Oopen = 3,
}

pub struct ZoneGuard;

impl ZoneGuard {
    #[inline]
    pub fn enter(_zone: Zone) -> Self {
        Self
    }
}

#[inline]
pub fn reset_decision() {}

#[inline]
pub fn dup_probe(_kind: DupKind, _key: impl FnOnce() -> u64) -> bool {
    false
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DecisionProf {
    pub zones: Vec<ZoneRow>,
    pub dups: Vec<DupRow>,
    pub wall_ns: u64,
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

pub fn take_decision(_wall: Duration) -> DecisionProf {
    DecisionProf {
        zones: Vec::new(),
        dups: Vec::new(),
        wall_ns: 0,
    }
}

pub fn merge_decisions(_rows: &[DecisionProf]) -> DecisionProf {
    DecisionProf {
        zones: Vec::new(),
        dups: Vec::new(),
        wall_ns: 0,
    }
}
