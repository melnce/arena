//! The rustc / Firefox "Fx" hash for in-process lookup tables keyed by small integers or short strings.

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;

/// A `HashMap` with [`FxHasher`]. Only for lookups: nothing may depend on its iteration order.
pub type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
/// A `HashSet` with [`FxHasher`]. Only for lookups: nothing may depend on its iteration order.
pub type FxSet<K> = HashSet<K, BuildHasherDefault<FxHasher>>;

/// One rotate, xor and multiply per word; SipHash's protection against chosen keys buys nothing here.
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    fn add(&mut self, w: u64) {
        self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

impl std::hash::Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.add(u64::from(b));
        }
    }
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}
