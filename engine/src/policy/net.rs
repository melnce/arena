//! Hand-rolled value-net inference. No ONNX, no extra crate: `serde_json`
//! parses `net.json` and [`ValueNet::value`] runs the forward pass.
//!
//! `load` uses `std::fs::read_to_string`, which compiles for
//! `wasm32-unknown-unknown` and is never called from the WASM client.

use std::path::Path;
use std::sync::Arc;

use crate::encode::{Observation, HIST_WIDTH, LEN};

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

/// Learned leaf value. `value` returns `scale × tanh(pre-activation)`.
#[derive(Clone)]
pub struct ValueNet {
    pub arch: NetArch,
    pub feature_len: usize,
    feat_mean: Vec<f32>,
    feat_std: Vec<f32>,
    vocab: Vec<u32>,
    zones: Vec<ZoneSpec>,
    scale: f32,
    linear: Option<Linear>,
    mlp: Option<Mlp>,
    pub path: String,
}

impl std::fmt::Debug for ValueNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueNet")
            .field("arch", &self.arch)
            .field("feature_len", &self.feature_len)
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

    fn parse(src: &str, text: &str) -> Result<Arc<ValueNet>, String> {
        let v: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("{src}: invalid JSON ({e})"))?;
        let obj = v
            .as_object()
            .ok_or_else(|| format!("{src}: expected object"))?;

        let arch = match req_str(src, obj, "arch")? {
            "linear" => NetArch::Linear,
            "mlp" => NetArch::Mlp,
            other => return Err(format!("{src}: bad field 'arch' ({other})")),
        };
        let feature_len = req_usize(src, obj, "feature_len")?;
        if feature_len != LEN {
            return Err(format!(
                "{src}: bad field 'feature_len' (want {LEN}, got {feature_len})"
            ));
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
        let zones = req_zones(src, obj)?;
        let scale = req_f32(src, obj, "scale")?;

        let (linear, mlp) = match arch {
            NetArch::Linear => {
                let lin = obj
                    .get("linear")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| format!("{src}: missing field 'linear'"))?;
                let w = req_f32_vec(src, lin, "w", feature_len)?;
                let zone_w = req_f32_mat(src, lin, "zone_w", 5, vlen)?;
                let b = req_f32(src, lin, "b")?;
                (Some(Linear { w, zone_w, b }), None)
            }
            NetArch::Mlp => {
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
                )
            }
        };

        Ok(Arc::new(ValueNet {
            arch,
            feature_len,
            feat_mean,
            feat_std,
            vocab,
            zones,
            scale,
            linear,
            mlp,
            path: src.to_string(),
        }))
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
                let id = obs.ids.get(zone.id_offset + slot).copied().unwrap_or(0);
                let idx = self.id_index(id);
                let count = self.slot_count(obs, zone, slot);
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
        let pre = match self.arch {
            NetArch::Linear => {
                let lin = self.linear.as_ref().expect("linear weights");
                let s = lin.b
                    + lin
                        .w
                        .iter()
                        .zip(x.iter())
                        .map(|(wi, xi)| wi * xi)
                        .sum::<f32>();
                s + self.zone_linear(obs, &lin.zone_w)
            }
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
        };
        self.scale * pre.tanh()
    }
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

fn req_zones(
    src: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<ZoneSpec>, String> {
    let arr = obj
        .get("zones")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{src}: missing field 'zones'"))?;
    if arr.len() != 5 {
        return Err(format!("{src}: bad field 'zones' (want 5)"));
    }
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
        if id_offset.saturating_add(count) > Observation::IDS_LEN {
            return Err(format!("{src}: bad field 'zones' (id range)"));
        }
        if let Some(h) = hist_offset {
            if h.saturating_add(count) > LEN || count > HIST_WIDTH {
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
