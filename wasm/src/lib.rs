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

    /// `HandCardInfo[]` JSON for `player` (`"a"` / `"b"`).
    #[wasm_bindgen(js_name = handInfo)]
    pub fn hand_info(&self, player: String) -> Result<String, JsValue> {
        self.inner.hand_info(&player).map_err(JsValue::from)
    }

    /// `BoardCardInfo[]` JSON for `player` (`"a"` / `"b"`). Occupied slots only.
    #[wasm_bindgen(js_name = boardInfo)]
    pub fn board_info(&self, player: String) -> Result<String, JsValue> {
        self.inner.board_info(&player).map_err(JsValue::from)
    }

    /// `PlayerInfo` JSON for `player` (`"a"` / `"b"`): evolve / super unlock.
    #[wasm_bindgen(js_name = playerInfo)]
    pub fn player_info(&self, player: String) -> Result<String, JsValue> {
        self.inner.player_info(&player).map_err(JsValue::from)
    }

    /// `ModeChoiceInfo` JSON while a mode choice is open; `null` otherwise.
    #[wasm_bindgen(js_name = modeChoiceInfo)]
    pub fn mode_choice_info(&self) -> Result<String, JsValue> {
        self.inner.mode_choice_info().map_err(JsValue::from)
    }

    /// Replace the live RNG. Canonical `hash` is unchanged.
    pub fn reseed(&mut self, seed: JsValue) -> Result<(), JsValue> {
        let seed = seed_from_js(&seed)?;
        self.inner.reseed(seed);
        Ok(())
    }

    #[wasm_bindgen(js_name = debugGrantCantAttackLeader)]
    pub fn debug_grant_cant_attack_leader(
        &mut self,
        player: String,
        slot: u8,
    ) -> Result<(), JsValue> {
        self.inner
            .debug_grant_cant_attack_leader(&player, slot)
            .map_err(JsValue::from)
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
        assert_eq!(names, vec!["random", "first-legal", "h0"]);
        let first = g.bot_action("first-legal", 1).unwrap();
        let legal_arr = legal.as_array().unwrap();
        assert_eq!(first, serde_json::to_string(&legal_arr[0]).unwrap());
        let r1 = g.bot_action("random", 7).unwrap();
        let r2 = g.bot_action("random", 7).unwrap();
        assert_eq!(r1, r2);
        let h1 = g.bot_action("h0", 1).unwrap();
        let h2 = g.bot_action("h0", 1).unwrap();
        assert_eq!(h1, h2);
        let err = g.bot_action("no-such-policy", 1).unwrap_err();
        assert!(err.contains("unknown policy no-such-policy"), "{err}");
    }

    mod event_json {
        use super::GameInner;
        use arena_engine::CardInstance;
        use arena_engine::ids::PlayerId;
        use arena_engine::{
            apply, new_game, Action, CardDb, CardId, First, GameConfig, Phase, State,
        };

        fn pad_deck(ids: &[&str], n: usize) -> Vec<CardId> {
            let mut v: Vec<CardId> = ids.iter().map(|s| CardId::parse(s).unwrap()).collect();
            let pad = CardId::parse("10001110").unwrap();
            while v.len() < n {
                v.push(pad);
            }
            v
        }

        fn started(db: &CardDb, seed: u64) -> State {
            let mut state = new_game(
                db,
                GameConfig {
                    seed,
                    deck_a: pad_deck(&["10001110"], 40),
                    deck_b: pad_deck(&["10001110"], 40),
                    first: First::A,
                    opening_hands: None,
                },
            )
            .expect("new_game");
            apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).unwrap();
            apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).unwrap();
            assert!(matches!(state.phase, Phase::Main));
            state
        }

        fn give_pp(state: &mut State, who: PlayerId, pp: i32, pp_max: i32) {
            let p = state.player_mut(who);
            p.pp_max = pp_max;
            p.pp = pp;
        }

        fn put_hand(db: &CardDb, state: &mut State, who: PlayerId, id: &str) -> u8 {
            let card = db.card(CardId::parse(id).unwrap()).unwrap();
            let inst = CardInstance::from_card(card, state.alloc_id());
            let pos = state.player(who).hand.len() as u8;
            state.player_mut(who).hand.push(inst);
            pos
        }

        fn put_field(db: &CardDb, state: &mut State, who: PlayerId, id: &str) -> u8 {
            let card = db.card(CardId::parse(id).unwrap()).unwrap();
            let mut inst = CardInstance::from_card(card, state.alloc_id());
            inst.flags.summoning_sick = false;
            let slot = state.player(who).first_empty_slot().unwrap();
            state.player_mut(who).field[slot as usize] = Some(inst);
            slot
        }

        fn slot_id(st: &State, p: PlayerId, slot: u8) -> u32 {
            st.field_inst(p, slot).expect("field").id
        }

        fn parse_events(json: &str) -> Vec<Value> {
            serde_json::from_str(json).expect("events json")
        }

        fn event_kind(ev: &Value) -> &str {
            ev.as_object()
                .and_then(|o| o.keys().next().map(String::as_str))
                .expect("event object")
        }

        fn slot_target_has_id(target: &Value) {
            assert!(target.get("slot").is_some());
            assert!(target.get("player").is_some());
            let id = target.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            assert!(id > 0, "slot target missing id: {target}");
        }

        fn check_event_contract(events: &[Value]) {
            let mut last_resolve: Option<usize> = None;
            for (i, ev) in events.iter().enumerate() {
                let kind = event_kind(ev);
                if kind == "resolve" {
                    last_resolve = Some(i);
                }
                match kind {
                    "damage" | "restore" => {
                        let body = &ev[kind];
                        let target = &body["target"];
                        if target.get("slot").is_some() {
                            assert!(last_resolve.is_some(), "missing resolve before {kind} at {i}");
                            slot_target_has_id(target);
                        }
                    }
                    "random_pick" => {
                        assert!(last_resolve.is_some(), "missing resolve before random_pick at {i}");
                        let target = &ev["random_pick"]["target"];
                        if target.get("slot").is_some() {
                            slot_target_has_id(target);
                        }
                    }
                    "destroy" => {
                        let body = &ev["destroy"];
                        assert!(body["id"].as_u64().unwrap_or(0) > 0, "destroy without id at {i}");
                        assert!(body["player"].is_string());
                    }
                    _ => {}
                }
            }
        }

        use serde_json::Value;

        fn play_json(st: &State, who: PlayerId, hand: u8) -> String {
            let card = st.player(who).hand[hand as usize].card.as_str();
            format!(
                r#"{{"play":{{"player":"{}","hand_pos":{hand},"card":"{card}"}}}}"#,
                who.as_str()
            )
        }

        #[test]
        fn damage_slot_ids_via_apply() {
            let db = super::super::bundle::card_db();
            let mut st = started(db, 43);
            let me = PlayerId::A;
            let opp = PlayerId::B;
            let f0 = put_field(db, &mut st, opp, "10001110");
            let f1 = put_field(db, &mut st, opp, "10011110");
            let id0 = slot_id(&st, opp, f0);
            let id1 = slot_id(&st, opp, f1);
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let h = put_hand(db, &mut st, me, "10753310");
            let action = play_json(&st, me, h);
            let mut game = GameInner::from_state_for_test(st);
            let events = parse_events(&game.apply(&action).unwrap());
            check_event_contract(&events);
            let mut ids = Vec::new();
            for ev in &events {
                if event_kind(ev) != "damage" {
                    continue;
                }
                let target = &ev["damage"]["target"];
                if target.get("slot").is_some() {
                    slot_target_has_id(target);
                    ids.push(target["id"].as_u64().unwrap() as u32);
                }
            }
            assert!(ids.len() >= 2, "expected multiple slot damages: {ids:?}");
            assert!(ids.contains(&id0));
            assert!(ids.contains(&id1));
        }

        #[test]
        fn destroy_player_and_id_via_apply() {
            let db = super::super::bundle::card_db();
            let mut st = started(db, 59);
            let opp = PlayerId::B;
            let slot = put_field(db, &mut st, opp, "10011210");
            let uid = slot_id(&st, opp, slot);
            if let Some(c) = st.field_inst_mut(opp, slot) {
                c.countdown = Some(1);
            }
            let mut game = GameInner::from_state_for_test(st);
            let events = parse_events(
                &game
                    .apply(r#"{"end_turn":{"player":"a"}}"#)
                    .unwrap(),
            );
            check_event_contract(&events);
            let destroy = events
                .iter()
                .find(|e| event_kind(e) == "destroy")
                .map(|e| &e["destroy"])
                .expect("countdown destroy");
            assert_eq!(destroy["player"], "b");
            assert_eq!(destroy["id"].as_u64().unwrap() as u32, uid);
        }

        #[test]
        fn random_pick_target_via_apply() {
            let db = super::super::bundle::card_db();
            let mut st = started(db, 44);
            let me = PlayerId::A;
            let opp = PlayerId::B;
            put_field(db, &mut st, me, "10011210");
            put_field(db, &mut st, opp, "10001110");
            put_field(db, &mut st, opp, "10011110");
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let h = put_hand(db, &mut st, me, "90011110");
            let action = play_json(&st, me, h);
            let mut game = GameInner::from_state_for_test(st);
            let events = parse_events(&game.apply(&action).unwrap());
            check_event_contract(&events);
            let picks: Vec<&Value> = events
                .iter()
                .filter(|e| {
                    event_kind(e) == "random_pick"
                        && e["random_pick"]["what"] == "random_target"
                })
                .map(|e| &e["random_pick"]["target"])
                .collect();
            assert_eq!(picks.len(), 1);
            slot_target_has_id(picks[0]);
            let dmg_id = events.iter().find_map(|e| {
                if event_kind(e) != "damage" {
                    return None;
                }
                let target = &e["damage"]["target"];
                target.get("id").cloned()
            });
            assert_eq!(picks[0]["id"], dmg_id.unwrap());
        }

        #[test]
        fn resolve_spell_source_before_damage_via_apply() {
            let db = super::super::bundle::card_db();
            let mut st = started(db, 48);
            let me = PlayerId::A;
            let opp = PlayerId::B;
            put_field(db, &mut st, opp, "10001110");
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let h = put_hand(db, &mut st, me, "10753310");
            let action = play_json(&st, me, h);
            let mut game = GameInner::from_state_for_test(st);
            let events = parse_events(&game.apply(&action).unwrap());
            check_event_contract(&events);
            let dmg_idx = events
                .iter()
                .position(|e| event_kind(e) == "damage")
                .expect("damage");
            let resolve_idx = events
                .iter()
                .position(|e| {
                    event_kind(e) == "resolve"
                        && e["resolve"]["source"]["spell"]["card"] == "10753310"
                })
                .expect("spell resolve");
            assert!(resolve_idx < dmg_idx);
        }

        #[test]
        fn choice_offered_spell_source_via_apply() {
            let db = super::super::bundle::card_db();
            let mut st = started(db, 56);
            let me = PlayerId::A;
            put_field(db, &mut st, PlayerId::B, "10001110");
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let h = put_hand(db, &mut st, me, "10671310");
            let action = play_json(&st, me, h);
            let mut game = GameInner::from_state_for_test(st);
            let events = parse_events(&game.apply(&action).unwrap());
            check_event_contract(&events);
            let offered = events
                .iter()
                .find(|e| event_kind(e) == "choice_offered")
                .map(|e| &e["choice_offered"])
                .expect("choice_offered");
            assert!(offered["source"]["spell"].is_object());
            assert_eq!(offered["source"]["spell"]["card"], "10671310");
        }

        #[test]
        fn restore_slot_ids_via_apply() {
            let db = super::super::bundle::card_db();
            let mut st = started(db, 61);
            let me = PlayerId::A;
            let ally = put_field(db, &mut st, me, "10001110");
            let ally_id = slot_id(&st, me, ally);
            if let Some(c) = st.field_inst_mut(me, ally) {
                c.defense = 1;
            }
            let kou = put_field(db, &mut st, me, "10411110");
            if let Some(c) = st.field_inst_mut(me, kou) {
                c.flags.attacks_left = 2;
                c.flags.attacked_this_turn = false;
            }
            st.phase = Phase::Main;
            st.active = me;
            let mut game = GameInner::from_state_for_test(st);
            let events = parse_events(
                &game
                    .apply(&format!(
                        r#"{{"attack":{{"player":"a","attacker_slot":{kou},"target":"leader"}}}}"#
                    ))
                    .unwrap(),
            );
            check_event_contract(&events);
            let restores: Vec<&Value> = events
                .iter()
                .filter(|e| {
                    event_kind(e) == "restore"
                        && e["restore"]["target"].get("slot").is_some()
                })
                .map(|e| &e["restore"]["target"])
                .collect();
            assert!(!restores.is_empty(), "expected slot restore events");
            assert!(restores.iter().any(|t| t["id"].as_u64() == Some(ally_id as u64)));
        }
    }
}
