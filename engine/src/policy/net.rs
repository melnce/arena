//! Hand-rolled value-net inference. No ONNX, no extra crate: `serde_json`
//! parses `net.json` and [`ValueNet::value`] runs the forward pass.
//!
//! The built-in model is `include_str!`-embedded and parsed once through
//! `OnceLock` ([`crate::policy::builtin_net`]). [`ValueNet::load`] uses
//! `std::fs::read_to_string`, which compiles for `wasm32-unknown-unknown`
//! and is never called from the WASM client. [`ValueNet::from_json`]
//! names the source `<json>`; [`ValueNet::from_json_named`] names it
//! (the built-in uses `builtin:h0-linear-v4`).

use std::path::Path;
use std::sync::Arc;

use crate::encode::{
    race_features, EncodingVersion, Observation, HIST_WIDTH, RACE_LEN, RACE_NAMES,
};

/// One input zone: a run of id slots, optionally weighted by a histogram.
#[derive(Debug, Clone)]
pub struct ZoneSpec {
    #[allow(dead_code)]
    pub name: String,
    pub id_offset: usize,
    pub count: usize,
    pub hist_offset: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetArch {
    Linear,
    Mlp,
    Residual,
}

#[derive(Clone)]
struct Linear {
    w: Vec<f32>,
    zone_w: Vec<Vec<f32>>,
    b: f32,
}

#[derive(Clone)]
struct Mlp {
    emb: Vec<Vec<f32>>,
    w1: Vec<Vec<f32>>,
    b1: Vec<f32>,
    w2: Vec<f32>,
    b2: f32,
}

#[derive(Clone)]
struct ResidualMlp {
    emb: Vec<Vec<Vec<f32>>>,
    emb_dim: usize,
    hidden: usize,
    w1: Vec<Vec<f32>>,
    b1: Vec<f32>,
    w2: Vec<f32>,
    b2: f32,
    x0_clip0: Vec<f32>,
    x0_clip5: Vec<f32>,
    a0_clip0: Vec<f32>,
    a0_clip5: Vec<f32>,
    /// `w1[k][i]` stored column-major: index `i * hidden + k`.
    w1_feat_col: Vec<f32>,
    /// Zone/embedding columns of `w1`, column-major: `j * hidden + k`.
    w1_zone_col: Vec<f32>,
}

const RESIDUAL_HIDDEN_STACK: usize = 256;
const RESIDUAL_ZONE_EMB_STACK: usize = 128;

#[derive(Clone)]
struct Race {
    mean: Vec<f32>,
    std: Vec<f32>,
    w: Vec<f32>,
}

/// Learned leaf value. `value` returns `scale × tanh(pre-activation)`.
#[derive(Clone)]
pub struct ValueNet {
    pub arch: NetArch,
    pub feature_len: usize,
    pub encoding: EncodingVersion,
    feat_mean: Vec<f32>,
    feat_std: Vec<f32>,
    vocab: Vec<u32>,
    zones: Vec<ZoneSpec>,
    scale: f32,
    linear: Option<Linear>,
    mlp: Option<Mlp>,
    residual_mlp: Option<ResidualMlp>,
    race: Option<Race>,
    pub path: String,
}

impl std::fmt::Debug for ValueNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueNet")
            .field("arch", &self.arch)
            .field("feature_len", &self.feature_len)
            .field("encoding", &self.encoding)
            .field("vocab", &self.vocab.len())
            .field("scale", &self.scale)
            .field("path", &self.path)
            .finish()
    }
}

impl ValueNet {
    /// Load `net.json` from disk. Errors name `path` and the field.
    pub fn load(path: impl AsRef<Path>) -> Result<Arc<ValueNet>, String> {
        let path = path.as_ref();
        let name = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
        Self::parse(&name, &text)
    }

    /// Parse a model document (tests / in-memory).
    pub fn from_json(text: &str) -> Result<Arc<ValueNet>, String> {
        Self::parse("<json>", text)
    }

    /// Parse a model document and name it in `Debug` / errors (`src`).
    pub fn from_json_named(name: &str, text: &str) -> Result<Arc<ValueNet>, String> {
        Self::parse(&format!("builtin:{name}"), text)
    }

    fn parse(src: &str, text: &str) -> Result<Arc<ValueNet>, String> {
        let v: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("{src}: invalid JSON ({e})"))?;
        let obj = v
            .as_object()
            .ok_or_else(|| format!("{src}: expected object"))?;

        let feature_len = req_usize(src, obj, "feature_len")?;
        let encoding = match obj.get("encoding") {
            None | Some(serde_json::Value::Null) => EncodingVersion::V1,
            Some(v) => {
                let n = as_u8(v).ok_or_else(|| format!("{src}: bad field 'encoding'"))?;
                EncodingVersion::parse(n)
                    .ok_or_else(|| format!("{src}: bad field 'encoding' ({n})"))?
            }
        };
        let want_len = encoding.feature_len();
        if feature_len != want_len {
            return Err(format!(
                "{src}: bad field 'feature_len' (encoding {} wants {}, got {})",
                encoding.as_u8(),
                want_len,
                feature_len
            ));
        }
        let arch = match req_str(src, obj, "arch")? {
            "linear" => NetArch::Linear,
            "mlp" => NetArch::Mlp,
            "residual" => NetArch::Residual,
            other => return Err(format!("{src}: bad field 'arch' ({other})")),
        };
        if encoding == EncodingVersion::V3 && arch == NetArch::Mlp {
            return Err(format!(
                "{src}: encoding 3 requires arch 'linear' or 'residual'"
            ));
        }
        if arch == NetArch::Residual && encoding != EncodingVersion::V3 {
            return Err(format!("{src}: arch 'residual' requires encoding 3"));
        }
        let feat_mean = req_f32_vec(src, obj, "feat_mean", feature_len)?;
        let feat_std = req_f32_vec(src, obj, "feat_std", feature_len)?;
        let vocab = req_u32_vec(src, obj, "vocab")?;
        if vocab.is_empty() || vocab[0] != 0 {
            return Err(format!("{src}: bad field 'vocab' (index 0 must be 0)"));
        }
        if !vocab.windows(2).all(|w| w[0] <= w[1]) {
            return Err(format!("{src}: bad field 'vocab' (must be ascending)"));
        }
        let vlen = vocab.len();
        let zones = req_zones(src, obj, encoding, feature_len)?;
        let scale = req_f32(src, obj, "scale")?;
        let n_zones = zones.len();

        let race = match obj.get("race") {
            None | Some(serde_json::Value::Null) => None,
            Some(_) if arch != NetArch::Linear => {
                return Err(format!("{src}: field 'race' requires arch 'linear'"));
            }
            Some(v) => Some(parse_race(src, v)?),
        };

        let (linear, mlp, residual_mlp) = match arch {
            NetArch::Linear => {
                let lin = obj
                    .get("linear")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| format!("{src}: missing field 'linear'"))?;
                let w = req_f32_vec(src, lin, "w", feature_len)?;
                let zone_w = req_f32_mat(src, lin, "zone_w", n_zones, vlen)?;
                if encoding == EncodingVersion::V3 {
                    for (row, zone) in zone_w.iter().zip(zones.iter()).skip(5) {
                        if row.first().copied().unwrap_or(0.0) != 0.0 {
                            return Err(format!(
                                "{src}: bad field 'zone_w' (index 0 must be 0 for zone '{}')",
                                zone.name
                            ));
                        }
                    }
                }
                let b = req_f32(src, lin, "b")?;
                (Some(Linear { w, zone_w, b }), None, None)
            }
            NetArch::Residual => {
                if obj.get("race").is_some() {
                    return Err(format!("{src}: field 'race' requires arch 'linear'"));
                }
                let lin = obj
                    .get("linear")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| format!("{src}: missing field 'linear'"))?;
                let w = req_f32_vec(src, lin, "w", feature_len)?;
                let zone_w = req_f32_mat(src, lin, "zone_w", n_zones, vlen)?;
                for (row, zone) in zone_w.iter().zip(zones.iter()).skip(5) {
                    if row.first().copied().unwrap_or(0.0) != 0.0 {
                        return Err(format!(
                            "{src}: bad field 'zone_w' (index 0 must be 0 for zone '{}')",
                            zone.name
                        ));
                    }
                }
                let b = req_f32(src, lin, "b")?;
                let mlp_obj = obj
                    .get("mlp")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| format!("{src}: missing field 'mlp'"))?;
                let res_mlp = parse_residual_mlp(
                    src,
                    mlp_obj,
                    feature_len,
                    n_zones,
                    vlen,
                    &feat_mean,
                    &feat_std,
                )?;
                (Some(Linear { w, zone_w, b }), None, Some(res_mlp))
            }
            NetArch::Mlp => {
                if obj.get("race").is_some() {
                    return Err(format!("{src}: field 'race' requires arch 'linear'"));
                }
                let mlp = obj
                    .get("mlp")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| format!("{src}: missing field 'mlp'"))?;
                let emb = req_f32_mat(src, mlp, "emb", vlen, 0)?;
                let emb_dim = emb.first().map(|r| r.len()).unwrap_or(0);
                if emb_dim == 0 {
                    return Err(format!("{src}: bad field 'emb'"));
                }
                let in_dim = feature_len + 5 * emb_dim;
                let w1 = req_f32_mat(src, mlp, "w1", 0, in_dim)?;
                if w1.is_empty() {
                    return Err(format!("{src}: bad field 'w1'"));
                }
                let hidden = w1.len();
                let b1 = req_f32_vec(src, mlp, "b1", hidden)?;
                let w2 = req_f32_vec(src, mlp, "w2", hidden)?;
                let b2 = req_f32(src, mlp, "b2")?;
                (
                    None,
                    Some(Mlp {
                        emb,
                        w1,
                        b1,
                        w2,
                        b2,
                    }),
                    None,
                )
            }
        };

        Ok(Arc::new(ValueNet {
            arch,
            feature_len,
            encoding,
            feat_mean,
            feat_std,
            vocab,
            zones,
            scale,
            linear,
            mlp,
            residual_mlp,
            race,
            path: src.to_string(),
        }))
    }

    fn race_linear(&self, obs: &Observation, clip: f32) -> f32 {
        let Some(race) = self.race.as_ref() else {
            return 0.0;
        };
        let r = race_features(&obs.features);
        let mut s = 0.0f32;
        for (j, rv) in r.iter().enumerate() {
            let x = (*rv - race.mean[j]) / race.std[j];
            let x = if clip > 0.0 { x.clamp(-clip, clip) } else { x };
            s += race.w[j] * x;
        }
        s
    }

    fn id_index(&self, id: u32) -> usize {
        self.vocab.binary_search(&id).unwrap_or(0)
    }

    fn slot_count(&self, obs: &Observation, zone: &ZoneSpec, slot: usize) -> f32 {
        match zone.hist_offset {
            Some(off) => obs.features.get(off + slot).copied().unwrap_or(0.0),
            None => 1.0,
        }
    }

    fn zone_linear(&self, obs: &Observation, zone_w: &[Vec<f32>]) -> f32 {
        let mut extra = 0.0f32;
        for (z, zone) in self.zones.iter().enumerate() {
            let table = &zone_w[z];
            for slot in 0..zone.count {
                let count = self.slot_count(obs, zone, slot);
                if zone.hist_offset.is_some() && count == 0.0 {
                    continue;
                }
                let id = obs.ids.get(zone.id_offset + slot).copied().unwrap_or(0);
                let idx = self.id_index(id);
                extra += count * table.get(idx).copied().unwrap_or(0.0);
            }
        }
        extra
    }

    fn zone_embs(&self, obs: &Observation, emb: &[Vec<f32>], emb_dim: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; 5 * emb_dim];
        for (z, zone) in self.zones.iter().enumerate() {
            let dest = z * emb_dim;
            for slot in 0..zone.count {
                let id = obs.ids.get(zone.id_offset + slot).copied().unwrap_or(0);
                let idx = self.id_index(id);
                let count = self.slot_count(obs, zone, slot);
                if let Some(row) = emb.get(idx) {
                    for e in 0..emb_dim {
                        out[dest + e] += count * row.get(e).copied().unwrap_or(0.0);
                    }
                }
            }
        }
        out
    }

    fn zone_embs_per_zone(
        &self,
        obs: &Observation,
        emb: &[Vec<Vec<f32>>],
        emb_dim: usize,
    ) -> Vec<f32> {
        let n_zones = emb.len();
        let mut out = vec![0.0f32; n_zones * emb_dim];
        for (z, zone) in self.zones.iter().enumerate() {
            let table = &emb[z];
            let dest = z * emb_dim;
            for slot in 0..zone.count {
                let id = obs.ids.get(zone.id_offset + slot).copied().unwrap_or(0);
                let idx = self.id_index(id);
                let count = self.slot_count(obs, zone, slot);
                if zone.hist_offset.is_some() && count == 0.0 {
                    continue;
                }
                if let Some(row) = table.get(idx) {
                    for e in 0..emb_dim {
                        out[dest + e] += count * row.get(e).copied().unwrap_or(0.0);
                    }
                }
            }
        }
        out
    }

    fn linear_pre(&self, x: &[f32], obs: &Observation) -> f32 {
        let lin = self.linear.as_ref().expect("linear weights");
        lin.b
            + lin
                .w
                .iter()
                .zip(x.iter())
                .map(|(wi, xi)| wi * xi)
                .sum::<f32>()
            + self.zone_linear(obs, &lin.zone_w)
    }

    fn residual_mlp_dense(&self, mlp: &ResidualMlp, x: &[f32], obs: &Observation) -> f32 {
        let z = self.zone_embs_per_zone(obs, &mlp.emb, mlp.emb_dim);
        let feat_off = self.feature_len;
        let h: Vec<f32> = mlp
            .w1
            .iter()
            .enumerate()
            .map(|(hi, row)| {
                let mut acc = mlp.b1.get(hi).copied().unwrap_or(0.0);
                for (wi, xi) in row.iter().zip(x.iter()) {
                    acc += wi * xi;
                }
                let extra = row.get(feat_off..).unwrap_or(&[]);
                for (wi, zi) in extra.iter().zip(z.iter()) {
                    acc += wi * zi;
                }
                acc.max(0.0)
            })
            .collect();
        mlp.b2
            + mlp
                .w2
                .iter()
                .zip(h.iter())
                .map(|(wi, hi)| wi * hi)
                .sum::<f32>()
    }

    #[inline]
    fn add_scaled_col(h: &mut [f32], scale: f32, col: &[f32]) {
        for k in 0..h.len() {
            h[k] += scale * col[k];
        }
    }

    /// Fused linear + MLP residual pre-activation for `clip` 0 and 5 (no per-call heap).
    fn residual_pre_sparse(&self, x: &[f32], obs: &Observation, clip: f32) -> f32 {
        let lin = self.linear.as_ref().expect("linear weights");
        let mlp = self.residual_mlp.as_ref().expect("residual mlp");
        let (x0, a0) = if clip == 0.0 {
            (&mlp.x0_clip0, &mlp.a0_clip0)
        } else {
            (&mlp.x0_clip5, &mlp.a0_clip5)
        };
        let hidden = mlp.hidden;
        let feat_len = self.feature_len;
        let emb_dim = mlp.emb_dim;
        let n_zone_emb = self.zones.len() * emb_dim;
        assert!(
            n_zone_emb <= RESIDUAL_ZONE_EMB_STACK,
            "zone embedding width {n_zone_emb} exceeds stack cap"
        );

        let mut pre = lin.b
            + lin
                .w
                .iter()
                .zip(x.iter())
                .map(|(wi, xi)| *wi * *xi)
                .sum::<f32>();

        let mut h_stack = [0.0f32; RESIDUAL_HIDDEN_STACK];
        let mut h_vec: Vec<f32>;
        let h: &mut [f32] = if hidden <= RESIDUAL_HIDDEN_STACK {
            let h = &mut h_stack[..hidden];
            h.copy_from_slice(a0);
            h
        } else {
            h_vec = a0.to_vec();
            &mut h_vec
        };

        for i in 0..feat_len {
            let f = obs.features.get(i).copied().unwrap_or(0.0);
            if f == 0.0 {
                continue;
            }
            let d = x[i] - x0[i];
            if d == 0.0 {
                continue;
            }
            let col = &mlp.w1_feat_col[i * hidden..(i + 1) * hidden];
            Self::add_scaled_col(h, d, col);
        }

        let mut e_stack = [0.0f32; RESIDUAL_ZONE_EMB_STACK];
        let e = &mut e_stack[..n_zone_emb];
        for (z, zone) in self.zones.iter().enumerate() {
            let zone_w_row = &lin.zone_w[z];
            let emb_table = &mlp.emb[z];
            let dest = z * emb_dim;
            for slot in 0..zone.count {
                let count = self.slot_count(obs, zone, slot);
                if zone.hist_offset.is_some() && count == 0.0 {
                    continue;
                }
                let id = obs.ids.get(zone.id_offset + slot).copied().unwrap_or(0);
                let idx = self.id_index(id);
                pre += count * zone_w_row.get(idx).copied().unwrap_or(0.0);
                if let Some(row) = emb_table.get(idx) {
                    for e_i in 0..emb_dim {
                        e[dest + e_i] += count * row.get(e_i).copied().unwrap_or(0.0);
                    }
                }
            }
        }

        for (j, ej) in e.iter().enumerate() {
            if *ej == 0.0 {
                continue;
            }
            let col = &mlp.w1_zone_col[j * hidden..(j + 1) * hidden];
            Self::add_scaled_col(h, *ej, col);
        }

        let mlp_add = mlp.b2
            + h.iter()
                .zip(mlp.w2.iter())
                .map(|(hk, wk)| wk * hk.max(0.0))
                .sum::<f32>();
        pre + mlp_add
    }

    fn standardize(&self, obs: &Observation, clip: f32) -> Vec<f32> {
        (0..self.feature_len)
            .map(|i| {
                let f = obs.features.get(i).copied().unwrap_or(0.0);
                let mean = self.feat_mean.get(i).copied().unwrap_or(0.0);
                let std = self.feat_std.get(i).copied().unwrap_or(1.0);
                let x = (f - mean) / std;
                if clip > 0.0 {
                    x.clamp(-clip, clip)
                } else {
                    x
                }
            })
            .collect()
    }

    /// `scale × tanh(pre-activation)` on the standardized observation.
    pub fn value(&self, obs: &Observation) -> f32 {
        let x: Vec<f32> = (0..self.feature_len)
            .map(|i| {
                let f = obs.features.get(i).copied().unwrap_or(0.0);
                let mean = self.feat_mean.get(i).copied().unwrap_or(0.0);
                let std = self.feat_std.get(i).copied().unwrap_or(1.0);
                (f - mean) / std
            })
            .collect();
        self.forward(&x, obs, 0.0)
    }

    /// Like [`value`], but clamps each standardised input to `[-clip, clip]`
    /// before the forward pass when `clip > 0`.
    pub fn value_clipped(&self, obs: &Observation, clip: f32) -> f32 {
        let x = self.standardize(obs, clip);
        self.forward(&x, obs, clip)
    }

    /// Dense residual forward for tests (sparse path must match for `clip` 0 and 5).
    #[doc(hidden)]
    pub fn value_residual_dense(&self, obs: &Observation, clip: f32) -> f32 {
        assert_eq!(self.arch, NetArch::Residual);
        let x = self.standardize(obs, clip);
        let mlp = self.residual_mlp.as_ref().expect("residual mlp");
        let pre = self.linear_pre(&x, obs) + self.residual_mlp_dense(mlp, &x, obs);
        self.scale * pre.tanh()
    }

    fn forward(&self, x: &[f32], obs: &Observation, clip: f32) -> f32 {
        let pre = match self.arch {
            NetArch::Linear => self.linear_pre(x, obs),
            NetArch::Mlp => {
                let mlp = self.mlp.as_ref().expect("mlp weights");
                let emb_dim = mlp.emb.first().map(|r| r.len()).unwrap_or(0);
                let z = self.zone_embs(obs, &mlp.emb, emb_dim);
                let h: Vec<f32> = mlp
                    .w1
                    .iter()
                    .enumerate()
                    .map(|(hi, row)| {
                        let mut acc = mlp.b1.get(hi).copied().unwrap_or(0.0);
                        for (wi, xi) in row.iter().zip(x.iter()) {
                            acc += wi * xi;
                        }
                        let extra = row.get(self.feature_len..).unwrap_or(&[]);
                        for (wi, zi) in extra.iter().zip(z.iter()) {
                            acc += wi * zi;
                        }
                        acc.max(0.0)
                    })
                    .collect();
                mlp.b2
                    + mlp
                        .w2
                        .iter()
                        .zip(h.iter())
                        .map(|(wi, hi)| wi * hi)
                        .sum::<f32>()
            }
            NetArch::Residual => {
                if clip == 0.0 || clip == 5.0 {
                    self.residual_pre_sparse(x, obs, clip)
                } else {
                    let mlp = self.residual_mlp.as_ref().expect("residual mlp");
                    self.linear_pre(x, obs) + self.residual_mlp_dense(mlp, x, obs)
                }
            }
        } + self.race_linear(obs, clip);
        self.scale * pre.tanh()
    }
}

fn std_zero_feature(mean: f32, std: f32, clip: f32) -> f32 {
    let x = -mean / std;
    if clip > 0.0 {
        x.clamp(-clip, clip)
    } else {
        x
    }
}

fn parse_residual_mlp(
    src: &str,
    mlp: &serde_json::Map<String, serde_json::Value>,
    feature_len: usize,
    n_zones: usize,
    vlen: usize,
    feat_mean: &[f32],
    feat_std: &[f32],
) -> Result<ResidualMlp, String> {
    let emb = req_emb_per_zone(src, mlp, "emb", n_zones, vlen)?;
    let emb_dim = emb
        .first()
        .and_then(|t| t.first())
        .map(|r| r.len())
        .unwrap_or(0);
    if emb_dim == 0 {
        return Err(format!("{src}: bad field 'emb'"));
    }
    let in_dim = feature_len + n_zones * emb_dim;
    let w1 = req_f32_mat(src, mlp, "w1", 0, in_dim)?;
    if w1.is_empty() {
        return Err(format!("{src}: bad field 'w1'"));
    }
    let hidden = w1.len();
    if hidden == 0 {
        return Err(format!("{src}: bad field 'w1'"));
    }
    let b1 = req_f32_vec(src, mlp, "b1", hidden)?;
    let w2 = req_f32_vec(src, mlp, "w2", hidden)?;
    let b2 = req_f32(src, mlp, "b2")?;
    for (field, vals) in [("b1", &b1), ("w2", &w2)] {
        for (i, v) in vals.iter().enumerate() {
            if !v.is_finite() {
                return Err(format!("{src}: bad field '{field}' (index {i})"));
            }
        }
    }
    if !b2.is_finite() {
        return Err(format!("{src}: bad field 'b2'"));
    }
    for table in &emb {
        for row in table {
            for (i, v) in row.iter().enumerate() {
                if !v.is_finite() {
                    return Err(format!("{src}: bad field 'emb' (index {i})"));
                }
            }
        }
    }
    for row in &w1 {
        for (i, v) in row.iter().enumerate() {
            if !v.is_finite() {
                return Err(format!("{src}: bad field 'w1' (index {i})"));
            }
        }
    }

    let x0_clip0: Vec<f32> = feat_mean
        .iter()
        .zip(feat_std.iter())
        .map(|(m, s)| std_zero_feature(*m, *s, 0.0))
        .collect();
    let x0_clip5: Vec<f32> = feat_mean
        .iter()
        .zip(feat_std.iter())
        .map(|(m, s)| std_zero_feature(*m, *s, 5.0))
        .collect();
    let mut a0_clip0 = vec![0.0f32; hidden];
    let mut a0_clip5 = vec![0.0f32; hidden];
    let mut w1_feat_col = vec![0.0f32; feature_len * hidden];
    for (k, row) in w1.iter().enumerate() {
        let mut z0 = b1[k] as f64;
        let mut z5 = b1[k] as f64;
        for i in 0..feature_len {
            let wi = row[i];
            w1_feat_col[i * hidden + k] = wi;
            z0 += (wi as f64) * (x0_clip0[i] as f64);
            z5 += (wi as f64) * (x0_clip5[i] as f64);
        }
        a0_clip0[k] = z0 as f32;
        a0_clip5[k] = z5 as f32;
    }

    let zone_in = n_zones * emb_dim;
    let mut w1_zone_col = vec![0.0f32; zone_in * hidden];
    for (k, row) in w1.iter().enumerate() {
        for j in 0..zone_in {
            w1_zone_col[j * hidden + k] = row[feature_len + j];
        }
    }

    Ok(ResidualMlp {
        emb,
        emb_dim,
        hidden,
        w1,
        b1,
        w2,
        b2,
        x0_clip0,
        x0_clip5,
        a0_clip0,
        a0_clip5,
        w1_feat_col,
        w1_zone_col,
    })
}

fn req_emb_per_zone(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    n_zones: usize,
    vlen: usize,
) -> Result<Vec<Vec<Vec<f32>>>, String> {
    let arr = obj
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field '{field}'"))?;
    if arr.len() != n_zones {
        return Err(format!(
            "{src}: bad field '{field}' (rows {}, want {n_zones})",
            arr.len()
        ));
    }
    let mut out = Vec::with_capacity(n_zones);
    let mut emb_dim = 0usize;
    for table in arr {
        let mat = table
            .as_array()
            .ok_or_else(|| format!("{src}: bad field '{field}'"))?;
        if mat.len() != vlen {
            return Err(format!("{src}: bad field '{field}'"));
        }
        let mut rows = Vec::with_capacity(vlen);
        for row in mat {
            let r = row
                .as_array()
                .ok_or_else(|| format!("{src}: bad field '{field}'"))?;
            if emb_dim == 0 {
                emb_dim = r.len();
                if emb_dim == 0 {
                    return Err(format!("{src}: bad field '{field}'"));
                }
            } else if r.len() != emb_dim {
                return Err(format!("{src}: bad field '{field}'"));
            }
            let mut vals = Vec::with_capacity(r.len());
            for v in r {
                vals.push(as_f32(v).ok_or_else(|| format!("{src}: bad field '{field}'"))?);
            }
            rows.push(vals);
        }
        out.push(rows);
    }
    Ok(out)
}

fn parse_race(src: &str, v: &serde_json::Value) -> Result<Race, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| format!("{src}: bad field 'race'"))?;
    let version = req_usize(src, obj, "version")?;
    if version != 1 {
        return Err(format!("{src}: bad field 'race.version' ({version})"));
    }
    let names = obj
        .get("names")
        .and_then(|x| x.as_array())
        .ok_or_else(|| format!("{src}: missing field 'race.names'"))?;
    if names.len() != RACE_LEN {
        return Err(format!(
            "{src}: bad field 'race.names' (len {}, want {RACE_LEN})",
            names.len()
        ));
    }
    for (i, n) in names.iter().enumerate() {
        let s = n
            .as_str()
            .ok_or_else(|| format!("{src}: bad field 'race.names'"))?;
        if s != RACE_NAMES[i] {
            return Err(format!("{src}: bad field 'race.names' (index {i})"));
        }
    }
    let mean = req_f32_vec(src, obj, "mean", RACE_LEN)?;
    let std = req_f32_vec(src, obj, "std", RACE_LEN)?;
    let w = req_f32_vec(src, obj, "w", RACE_LEN)?;
    for (i, s) in std.iter().enumerate() {
        if !s.is_finite() {
            return Err(format!("{src}: bad field 'race.std' (index {i})"));
        }
        if *s <= 0.0 {
            return Err(format!("{src}: bad field 'race.std' (index {i})"));
        }
    }
    for (field, vals) in [("mean", &mean), ("w", &w)] {
        for (i, v) in vals.iter().enumerate() {
            if !v.is_finite() {
                return Err(format!("{src}: bad field 'race.{field}' (index {i})"));
            }
        }
    }
    Ok(Race { mean, std, w })
}

fn req_str<'a>(
    src: &str,
    obj: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    obj.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{src}: missing field '{field}'"))
}

fn req_f32(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<f32, String> {
    let v = obj
        .get(field)
        .ok_or_else(|| format!("{src}: missing field '{field}'"))?;
    as_f32(v).ok_or_else(|| format!("{src}: bad field '{field}'"))
}

fn req_usize(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<usize, String> {
    let v = obj
        .get(field)
        .ok_or_else(|| format!("{src}: missing field '{field}'"))?;
    as_usize(v).ok_or_else(|| format!("{src}: bad field '{field}'"))
}

fn req_f32_vec(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    want: usize,
) -> Result<Vec<f32>, String> {
    let arr = obj
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field '{field}'"))?;
    if want > 0 && arr.len() != want {
        return Err(format!(
            "{src}: bad field '{field}' (len {}, want {want})",
            arr.len()
        ));
    }
    arr.iter()
        .map(|v| as_f32(v).ok_or_else(|| format!("{src}: bad field '{field}'")))
        .collect()
}

fn req_u32_vec(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<Vec<u32>, String> {
    let arr = obj
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field '{field}'"))?;
    arr.iter()
        .map(|v| as_u32(v).ok_or_else(|| format!("{src}: bad field '{field}'")))
        .collect()
}

/// `rows == 0` accepts any non-zero row count; `cols == 0` accepts any
/// non-zero column count (taken from the first row).
fn req_f32_mat(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    rows: usize,
    cols: usize,
) -> Result<Vec<Vec<f32>>, String> {
    let arr = obj
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field '{field}'"))?;
    if rows > 0 && arr.len() != rows {
        return Err(format!(
            "{src}: bad field '{field}' (rows {}, want {rows})",
            arr.len()
        ));
    }
    let mut out = Vec::with_capacity(arr.len());
    let mut expect_cols = cols;
    for row in arr {
        let r = row
            .as_array()
            .ok_or_else(|| format!("{src}: bad field '{field}'"))?;
        if expect_cols == 0 {
            expect_cols = r.len();
            if expect_cols == 0 {
                return Err(format!("{src}: bad field '{field}'"));
            }
        } else if r.len() != expect_cols {
            return Err(format!("{src}: bad field '{field}'"));
        }
        let mut vals = Vec::with_capacity(r.len());
        for v in r {
            vals.push(as_f32(v).ok_or_else(|| format!("{src}: bad field '{field}'"))?);
        }
        out.push(vals);
    }
    Ok(out)
}

fn expected_zone_specs(
    encoding: EncodingVersion,
) -> Vec<(&'static str, usize, usize, Option<usize>)> {
    let mut out = vec![
        ("own_hand", 0, 9, None),
        ("own_deck", 9, HIST_WIDTH, Some(353)),
        ("opp_board", 105, 5, None),
        ("own_board", 110, 5, None),
        ("opp_pool", 115, HIST_WIDTH, Some(449)),
    ];
    if encoding == EncodingVersion::V3 {
        out.extend([
            ("own_crests", 220, 5, None),
            ("opp_crests", 225, 5, None),
            ("own_amulet_soon", 110, 5, Some(567)),
            ("opp_amulet_soon", 105, 5, Some(572)),
            ("own_entered", 9, HIST_WIDTH, Some(577)),
            ("opp_entered", 115, HIST_WIDTH, Some(673)),
            ("own_cemetery", 9, HIST_WIDTH, Some(769)),
            ("opp_cemetery", 115, HIST_WIDTH, Some(865)),
        ]);
    }
    out
}

fn req_zones(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    encoding: EncodingVersion,
    feature_len: usize,
) -> Result<Vec<ZoneSpec>, String> {
    if encoding == EncodingVersion::V3 {
        req_zones_v3(src, obj, feature_len)
    } else {
        req_zones_legacy(src, obj, encoding, feature_len)
    }
}

fn req_zones_legacy(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    encoding: EncodingVersion,
    feature_len: usize,
) -> Result<Vec<ZoneSpec>, String> {
    let arr = obj
        .get("zones")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field 'zones'"))?;
    if arr.len() != 5 {
        return Err(format!("{src}: bad field 'zones' (want 5)"));
    }
    let ids_cap = encoding.ids_len();
    let mut out = Vec::with_capacity(5);
    for z in arr {
        let o = z
            .as_object()
            .ok_or_else(|| format!("{src}: bad field 'zones'"))?;
        let name = req_str(src, o, "name")?.to_string();
        let id_offset = req_usize(src, o, "id_offset")?;
        let count = req_usize(src, o, "count")?;
        let hist_offset = match o.get("hist_offset") {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(as_usize(v).ok_or_else(|| format!("{src}: bad field 'hist_offset'"))?),
        };
        if id_offset.saturating_add(count) > ids_cap {
            return Err(format!("{src}: bad field 'zones' (id range)"));
        }
        if let Some(h) = hist_offset {
            if h.saturating_add(count) > feature_len || count > HIST_WIDTH {
                return Err(format!("{src}: bad field 'zones' (hist range)"));
            }
        }
        out.push(ZoneSpec {
            name,
            id_offset,
            count,
            hist_offset,
        });
    }
    Ok(out)
}

fn req_zones_v3(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    feature_len: usize,
) -> Result<Vec<ZoneSpec>, String> {
    let expected = expected_zone_specs(EncodingVersion::V3);
    let arr = obj
        .get("zones")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field 'zones'"))?;
    if arr.len() != expected.len() {
        return Err(format!(
            "{src}: bad field 'zones' (want {})",
            expected.len()
        ));
    }
    let ids_cap = EncodingVersion::V3.ids_len();
    let mut out = Vec::with_capacity(expected.len());
    for (z, (exp_name, exp_id, exp_count, exp_hist)) in arr.iter().zip(expected.iter()) {
        let o = z
            .as_object()
            .ok_or_else(|| format!("{src}: bad field 'zones'"))?;
        let name = req_str(src, o, "name")?.to_string();
        let id_offset = req_usize(src, o, "id_offset")?;
        let count = req_usize(src, o, "count")?;
        let hist_offset = match o.get("hist_offset") {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(as_usize(v).ok_or_else(|| format!("{src}: bad field 'hist_offset'"))?),
        };
        if name != *exp_name
            || id_offset != *exp_id
            || count != *exp_count
            || hist_offset != *exp_hist
        {
            return Err(format!("{src}: bad field 'zones'"));
        }
        if id_offset.saturating_add(count) > ids_cap {
            return Err(format!("{src}: bad field 'zones' (id range)"));
        }
        if let Some(h) = hist_offset {
            if h.saturating_add(count) > feature_len || count > HIST_WIDTH {
                return Err(format!("{src}: bad field 'zones' (hist range)"));
            }
        }
        out.push(ZoneSpec {
            name,
            id_offset,
            count,
            hist_offset,
        });
    }
    Ok(out)
}

fn as_f32(v: &serde_json::Value) -> Option<f32> {
    match v {
        serde_json::Value::Number(n) => n.as_f64().map(|x| x as f32),
        _ => None,
    }
}

fn as_u8(v: &serde_json::Value) -> Option<u8> {
    match v {
        serde_json::Value::Number(n) => n
            .as_u64()
            .and_then(|x| u8::try_from(x).ok())
            .or_else(|| n.as_i64().and_then(|x| u8::try_from(x).ok())),
        _ => None,
    }
}

fn as_u32(v: &serde_json::Value) -> Option<u32> {
    match v {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|x| x as u32)
            .or_else(|| n.as_i64().map(|x| x as u32)),
        _ => None,
    }
}

fn as_usize(v: &serde_json::Value) -> Option<usize> {
    match v {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|x| x as usize)
            .or_else(|| n.as_i64().map(|x| x as usize)),
        _ => None,
    }
}

#[cfg(test)]
mod residual_sparse_tests {
    use super::*;
    use crate::encode::Observation;

    fn tiny_residual() -> Arc<ValueNet> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../py/tests/fixtures/order_residual/tiny-residual.json");
        ValueNet::load(path).expect("tiny residual fixture")
    }

    fn synthetic_obs(net: &ValueNet, seed: u64) -> Observation {
        let mut st = seed;
        let mut features = vec![0.0f32; net.feature_len];
        for _ in 0..12 {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1);
            let i = (st as usize) % net.feature_len;
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1);
            features[i] = ((st >> 32) as i32 as f32) * 0.001;
        }
        let mut ids = vec![0u32; 230];
        for id in &mut ids {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1);
            let vi = 1 + (st as usize % net.vocab.len().max(2));
            *id = net.vocab[vi.min(net.vocab.len() - 1)];
        }
        Observation { features, ids }
    }

    fn value_dense_residual(net: &ValueNet, obs: &Observation, clip: f32) -> f32 {
        let x = net.standardize(obs, clip);
        let mlp = net.residual_mlp.as_ref().expect("residual mlp");
        let lin = net.linear_pre(&x, obs);
        let add = net.residual_mlp_dense(mlp, &x, obs);
        net.scale * (lin + add).tanh()
    }

    #[test]
    fn sparse_first_layer_matches_dense() {
        let net = tiny_residual();
        let mut obs: Vec<Observation> = (0..30).map(|s| synthetic_obs(&net, 10_000 + s)).collect();
        let review = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../py/tests/fixtures/review13/games/4984932781931433298-b4973563.json"),
        )
        .expect("review");
        let cap: serde_json::Value = serde_json::from_str(&review).expect("json");
        let seed = cap["seed"].as_u64().expect("seed");
        let mut features = vec![0.0f32; net.feature_len];
        if let Some(arr) = cap.get("feat_snapshot").and_then(|v| v.as_array()) {
            for (i, v) in arr.iter().enumerate() {
                if i < features.len() {
                    features[i] = v.as_f64().unwrap_or(0.0) as f32;
                }
            }
        } else {
            features[42] = 1.5;
        }
        let ids = vec![0u32; 230];
        obs.push(Observation { features, ids });
        obs.push(synthetic_obs(&net, seed));

        for clip in [0.0f32, 5.0f32, 2.5f32] {
            for ob in &obs {
                let got = if clip == 0.0 {
                    net.value(ob)
                } else {
                    net.value_clipped(ob, clip)
                };
                let dense = value_dense_residual(&net, ob, clip);
                assert!(
                    (got - dense).abs() <= 1e-4,
                    "clip={clip} got={got} dense={dense}"
                );
            }
        }
    }
}
