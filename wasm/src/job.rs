//! Stepped H0 bot decisions for multi-worker wasm clients.

use arena_engine::{
    by_name, kill_det, kill_roll, legal_actions, policy_rng, run_job_choose, to_neutral, world,
    H0Job, JobStep, KillDetOutcome, KillRollOutcome, WorldOutcome,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::bundle::card_db;
use crate::inner::GameInner;
use crate::Game;

#[derive(Serialize)]
struct StepJson {
    step: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    roll: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    budgets: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
}

fn step_to_json(game: &GameInner, step: JobStep) -> Result<String, String> {
    let db = card_db();
    let legal = legal_actions(db, game.state());
    match step {
        JobStep::Done { index, value } => {
            let idx = index as usize;
            if idx >= legal.len() {
                return Err(format!("job chose {idx} past legal_len={}", legal.len()));
            }
            let action = serde_json::to_value(to_neutral(game.state(), &legal[idx]))
                .map_err(|e| e.to_string())?;
            let value = value.map(serde_json::Value::from);
            serde_json::to_string(&StepJson {
                step: "done",
                roll: None,
                budgets: None,
                action: Some(action),
                value,
            })
            .map_err(|e| e.to_string())
        }
        JobStep::Kill { roll } => serde_json::to_string(&StepJson {
            step: "kill",
            roll: Some(roll),
            budgets: None,
            action: None,
            value: None,
        })
        .map_err(|e| e.to_string()),
        JobStep::AwaitRoll => serde_json::to_string(&StepJson {
            step: "await_roll",
            roll: None,
            budgets: None,
            action: None,
            value: None,
        })
        .map_err(|e| e.to_string()),
        JobStep::Worlds { budgets } => serde_json::to_string(&StepJson {
            step: "worlds",
            roll: None,
            budgets: Some(budgets),
            action: None,
            value: None,
        })
        .map_err(|e| e.to_string()),
    }
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(js_name = h0JobStart)]
    pub fn h0_job_start(&self, policy: String, seed: JsValue) -> Result<String, JsValue> {
        let seed = crate::seed_from_js(&seed)?;
        let db = card_db();
        self.inner.clear_h0_job();
        let (job, step) =
            H0Job::start(db, self.inner.state(), &policy, seed).map_err(JsValue::from)?;
        *self.inner.h0_job_mut() = Some(job);
        step_to_json(&self.inner, step).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = h0KillDet)]
    pub fn h0_kill_det(&self, policy: String, seed: JsValue) -> Result<String, JsValue> {
        let seed = crate::seed_from_js(&seed)?;
        let out = kill_det(card_db(), self.inner.state(), &policy, seed).map_err(JsValue::from)?;
        serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = h0KillRoll)]
    pub fn h0_kill_roll(&self, policy: String, seed: JsValue) -> Result<String, JsValue> {
        let seed = crate::seed_from_js(&seed)?;
        let out = kill_roll(card_db(), self.inner.state(), &policy, seed).map_err(JsValue::from)?;
        serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = h0JobKillDet)]
    pub fn h0_job_kill_det(&self, outcome_json: String) -> Result<String, JsValue> {
        let outcome: KillDetOutcome =
            serde_json::from_str(&outcome_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let db = card_db();
        let step = {
            let mut guard = self.inner.h0_job_mut();
            let job = guard
                .as_mut()
                .ok_or_else(|| JsValue::from_str("no h0 job in progress; call h0JobStart first"))?;
            job.provide_kill_det(db, outcome).map_err(JsValue::from)?
        };
        step_to_json(&self.inner, step).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = h0JobKillRoll)]
    pub fn h0_job_kill_roll(&self, outcome_json: String) -> Result<String, JsValue> {
        let outcome: KillRollOutcome =
            serde_json::from_str(&outcome_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let db = card_db();
        let step = {
            let mut guard = self.inner.h0_job_mut();
            let job = guard
                .as_mut()
                .ok_or_else(|| JsValue::from_str("no h0 job in progress; call h0JobStart first"))?;
            job.provide_kill_roll(db, outcome).map_err(JsValue::from)?
        };
        step_to_json(&self.inner, step).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = h0JobWorld)]
    pub fn h0_job_world(
        &self,
        policy: String,
        seed: JsValue,
        r: u32,
        budget: u32,
    ) -> Result<String, JsValue> {
        let seed = crate::seed_from_js(&seed)?;
        let out = world(card_db(), self.inner.state(), &policy, seed, r as usize, budget)
        .map_err(JsValue::from)?;
        serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = h0JobFinish)]
    pub fn h0_job_finish(&self, outcomes_json: String) -> Result<String, JsValue> {
        let worlds: Vec<WorldOutcome> =
            serde_json::from_str(&outcomes_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let db = card_db();
        let step = {
            let mut guard = self.inner.h0_job_mut();
            let job = guard
                .as_mut()
                .ok_or_else(|| JsValue::from_str("no h0 job in progress; call h0JobStart first"))?;
            job.finish(db, &worlds).map_err(JsValue::from)?
        };
        self.inner.clear_h0_job();
        step_to_json(&self.inner, step).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = botActionValue)]
    pub fn bot_action_value(&self, policy: String, seed: JsValue) -> Result<String, JsValue> {
        let seed = crate::seed_from_js(&seed)?;
        let db = card_db();
        let legal = legal_actions(db, self.inner.state());
        if legal.is_empty() {
            return Err(JsValue::from_str("no legal actions"));
        }
        let mut boxed = by_name(&policy, seed)
            .ok_or_else(|| JsValue::from_str(&format!("unknown policy {policy}")))?;
        let mut rng = policy_rng(seed);
        let idx = boxed.choose(db, self.inner.state(), &legal, &mut rng);
        if idx >= legal.len() {
            return Err(JsValue::from_str(&format!(
                "policy {policy} chose {idx} past legal_len={}",
                legal.len()
            )));
        }
        let action = serde_json::to_value(to_neutral(self.inner.state(), &legal[idx]))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let value = boxed
            .last_value()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null);
        serde_json::to_string(&serde_json::json!({ "action": action, "value": value }))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Run the full job path on separate `Game` clones (for tests).
pub fn job_choose_separate_games(
    games: &[GameInner],
    policy: &str,
    seed: u64,
) -> Result<(String, Option<f32>), String> {
    if games.is_empty() {
        return Err("need at least one game".into());
    }
    let db = card_db();
    let (idx, value, _stats) = run_job_choose(
        db,
        games[0].state(),
        policy,
        seed,
        false,
        false,
        false,
    )?;
    let legal = legal_actions(db, games[0].state());
    let action = serde_json::to_string(&to_neutral(games[0].state(), &legal[idx]))
        .map_err(|e| e.to_string())?;
    Ok((action, value))
}
