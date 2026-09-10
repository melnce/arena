//! Thin WASM wrapper. The JS API is JSON-neutral — see `docs/engine-api.md`.
//!
//! JSON strings (not `serde-wasm-bindgen`) so the client speaks the same
//! NeutralAction / CanonicalState JSON as `arena-trace` / `arena-replay`.

use wasm_bindgen::prelude::*;

mod bundle;
mod inner;
mod ser;

use crate::inner::GameInner;

#[wasm_bindgen]
pub struct Game {
    inner: GameInner,
}

#[wasm_bindgen]
impl Game {
    /// `seed` is a JS number, bigint, or decimal string.
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: JsValue,
        deck_a: String,
        deck_b: String,
        first: String,
    ) -> Result<Game, JsValue> {
        let seed = seed_from_js(&seed)?;
        GameInner::new(seed, &deck_a, &deck_b, &first)
            .map(|inner| Game { inner })
            .map_err(JsValue::from)
    }

    pub fn legal(&self) -> Result<String, JsValue> {
        self.inner.legal().map_err(JsValue::from)
    }

    pub fn apply(&mut self, action: String) -> Result<String, JsValue> {
        self.inner.apply(&action).map_err(JsValue::from)
    }

    pub fn snapshot(&self) -> Result<String, JsValue> {
        self.inner.snapshot().map_err(JsValue::from)
    }

    pub fn full(&self) -> Result<String, JsValue> {
        self.inner.full().map_err(JsValue::from)
    }

    pub fn hash(&self) -> String {
        self.inner.hash()
    }

    pub fn phase(&self) -> String {
        self.inner.phase()
    }

    #[wasm_bindgen(js_name = clone)]
    pub fn js_clone(&self) -> Game {
        Game {
            inner: self.inner.clone(),
        }
    }

    pub fn acting(&self) -> String {
        self.inner.acting()
    }

    pub fn active(&self) -> String {
        self.inner.active()
    }

    pub fn turn(&self) -> u32 {
        self.inner.turn()
    }

    /// `"a"` / `"b"`, or `null` when the match is not over.
    pub fn winner(&self) -> JsValue {
        match self.inner.winner() {
            Some(s) => JsValue::from_str(&s),
            None => JsValue::NULL,
        }
    }

    /// One `NeutralAction` JSON for the acting player.
    #[wasm_bindgen(js_name = botAction)]
    pub fn bot_action(&self, policy: String, seed: JsValue) -> Result<String, JsValue> {
        let seed = seed_from_js(&seed)?;
        self.inner.bot_action(&policy, seed).map_err(JsValue::from)
    }
}

/// JSON array of policy names the client can put in a selector.
#[wasm_bindgen(js_name = botPolicies)]
pub fn bot_policies() -> String {
    crate::inner::bot_policies_json()
}

#[wasm_bindgen(js_name = cardText)]
pub fn card_text(id: String) -> Result<String, JsValue> {
    crate::bundle::card_text(&id).map_err(JsValue::from)
}

#[wasm_bindgen(js_name = bundleInfo)]
pub fn bundle_info() -> String {
    crate::bundle::bundle_info()
}

#[wasm_bindgen]
pub fn version() -> String {
    crate::bundle::version().to_string()
}

fn seed_from_js(seed: &JsValue) -> Result<u64, JsValue> {
    if let Some(n) = seed.as_f64() {
        if n.is_finite() && n >= 0.0 {
            return Ok(n as u64);
        }
    }
    if let Some(s) = seed.as_string() {
        return s
            .parse::<u64>()
            .map_err(|e| JsValue::from_str(&e.to_string()));
    }
    if let Ok(b) = js_sys::BigInt::new(seed) {
        if let Ok(s) = b.to_string(10) {
            let s = String::from(s);
            if let Ok(v) = s.parse::<u64>() {
                return Ok(v);
            }
        }
    }
    Err(JsValue::from_str(
        "seed must be a non-negative number, bigint, or decimal string",
    ))
}

#[cfg(test)]
mod tests {
    use super::inner::GameInner;
    use std::fs;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    fn read_deck(name: &str) -> String {
        fs::read_to_string(repo_root().join("oracle/decks").join(name)).expect("deck")
    }

    #[test]
    fn bundle_starts_basic_forest_vs_rune() {
        let a = read_deck("basic-forest.json");
        let b = read_deck("basic-rune.json");
        let g = GameInner::new(1, &a, &b, "a").expect("new game");
        assert_eq!(g.phase(), "mulligan");
        let legal: serde_json::Value = serde_json::from_str(&g.legal().unwrap()).unwrap();
        assert!(legal.as_array().is_some_and(|a| !a.is_empty()));
        let info: serde_json::Value = serde_json::from_str(&crate::bundle::bundle_info()).unwrap();
        assert!(info["cards"].as_u64().unwrap() > 0);
        assert!(info["bytes"].as_u64().unwrap() > 0);
        assert_eq!(g.acting(), "a");
        assert_eq!(g.active(), "a");
        assert_eq!(g.turn(), 0);
        assert!(g.winner().is_none());
        let names: Vec<String> = serde_json::from_str(&crate::inner::bot_policies_json()).unwrap();
        assert_eq!(names, vec!["random", "first-legal"]);
        let first = g.bot_action("first-legal", 1).unwrap();
        let legal_arr = legal.as_array().unwrap();
        assert_eq!(first, serde_json::to_string(&legal_arr[0]).unwrap());
        let r1 = g.bot_action("random", 7).unwrap();
        let r2 = g.bot_action("random", 7).unwrap();
        assert_eq!(r1, r2);
        assert!(g.bot_action("h0", 1).is_err());
    }
}
