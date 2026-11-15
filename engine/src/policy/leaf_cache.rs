//! Per-decision cache for net leaf values (`search_key_id`, me, net ptr, clip).

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::fxhash::FxMap;

pub type LeafValKey = (u64, u8, u64, u32);
pub type LeafValMap = FxMap<LeafValKey, u32>;

thread_local! {
    static TLS: RefCell<Option<Arc<Mutex<LeafValMap>>>> = const { RefCell::new(None) };
}

/// Attach the decision cache to this OS thread for the duration of one decision.
pub fn bind(arc: Arc<Mutex<LeafValMap>>) {
    TLS.with(|c| *c.borrow_mut() = Some(arc));
}

/// Detach after a parallel worker or decision finishes on this thread.
pub fn unbind() {
    TLS.with(|c| c.borrow_mut().take());
}

pub fn with<R>(f: impl FnOnce(&Mutex<LeafValMap>) -> R) -> Option<R> {
    TLS.with(|c| {
        let slot = c.borrow();
        let arc = slot.as_ref()?;
        Some(f(arc))
    })
}

pub fn get(key: &LeafValKey) -> Option<f32> {
    with(|cache| {
        let guard = cache.lock().ok()?;
        guard.get(key).copied().map(f32::from_bits)
    })
    .flatten()
}

pub fn insert(key: LeafValKey, v: f32) {
    let _ = with(|cache| {
        if let Ok(mut guard) = cache.lock() {
            guard.insert(key, v.to_bits());
        }
    });
}
