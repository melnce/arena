//! Player policies: random-legal, first-legal, and H0 (determinized search).
//!
//! This module (and `encode` / `search_key` / `determinize`) must compile for
//! `wasm32-unknown-unknown`: no `std::time::{Instant, SystemTime}`, threads,
//! rayon, or `getrandom` without the js feature. The built-in value model is
//! `include_str!`-embedded and parsed once through `OnceLock`.
//! `ValueNet::load` is still the only `std::fs` user and is never called
//! from wasm (it compiles on wasm32). The node cap is the only search
//! budget. `Policy` is object-safe so WASM can hold `Box<dyn Policy>`.

mod h0;
mod needs;
mod net;
mod record;

pub use h0::{builtin_net, Alloc, Info, SearchStats, ValueVersion, Weights, BUILTIN_NET_NAME, H0};
pub use needs::{CardNeeds, NeedsTable, SkippedAmount};
pub use net::{NetArch, ValueNet};
pub use record::{Recorder, Sample};

use crate::action::Action;
use crate::db::CardDb;
use crate::rng::Xoshiro256ss;
use crate::state::State;

/// Object-safe player. WASM holds `Box<dyn Policy>` from [`by_name`].
pub trait Policy {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize;

    /// Value the policy computed for the position at its most recent
    /// [`choose`], from the acting player's perspective, on the leaf's
    /// scale (`[-wv, wv]`). `None` when it did not search.
    fn last_value(&self) -> Option<f32> {
        None
    }
}

const NAMES: &[&str] = &["random", "first-legal", "h0"];

/// Names [`by_name`] accepts, in announcement order.
pub fn names() -> &'static [&'static str] {
    NAMES
}

/// Construct a policy by the stable WASM / bench name.
///
/// `seed` is accepted so a host can pass the game seed at construction;
/// `Random` / `FirstLegal` / `H0` do not store it — `choose` uses the
/// caller-supplied rng (typically `policy_rng(seed)`). Unknown names and
/// malformed specs return `None` (the WASM client still lists only
/// [`names`]).
pub fn by_name(name: &str, seed: u64) -> Option<Box<dyn Policy>> {
    let _ = seed;
    Some(Box::new(AnyPolicy::parse_spec(name).ok()?))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Random;

impl Policy for Random {
    fn choose(
        &mut self,
        _db: &CardDb,
        _state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        if legal.is_empty() {
            return 0;
        }
        rng.gen_range(legal.len() as u32) as usize
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FirstLegal;

impl Policy for FirstLegal {
    fn choose(
        &mut self,
        _db: &CardDb,
        _state: &State,
        legal: &[Action],
        _rng: &mut Xoshiro256ss,
    ) -> usize {
        let _ = legal;
        0
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AnyPolicy {
    Random(Random),
    FirstLegal(FirstLegal),
    H0(H0),
}

impl AnyPolicy {
    pub fn parse(s: &str) -> Self {
        Self::parse_spec(s).unwrap_or_else(|e| panic!("{e}"))
    }

    /// `"random"` | `"first-legal"` | `"h0"` ([`H0::default`]) | `"h0-fast"`
    /// ([`H0::fast`]) | `"h0:depth=6,beam=4,k=4,nodes=2000"` — any subset of
    /// keys, the rest default; `k` = `determinizations`, `nodes` = `node_cap`.
    /// H0 also accepts `value=v0|v1|net` (default `net` = the built-in
    /// `h0-linear-v1`), `net=<path>` (overrides the built-in; only
    /// meaningful with `value=net`; `h0:net=<path>` alone means
    /// `h0:value=net,net=<path>`; the path may not contain commas; a
    /// missing file is a parse error naming the path),
    /// `odepth=` / `obeam=`
    /// (opponent model; `odepth=0` is the greedy line), `olethal=0|1` (cheap
    /// opponent-lethal sweep on the greedy path; default `1`; `olethal=0`
    /// restores the pre-flip greedy path; ignored when
    /// `odepth≥1`), `oevo=0|1` (extend that sweep with one evolve;
    /// default `1` is the sweep-8b flip; `oevo=0` restores the pre-flip
    /// glance path; only meaningful with `olethal=1` and `odepth=0`;
    /// no hard error for other combinations), `osteps=<u32>` (greedy
    /// forced-`EndTurn` step; default `6`;
    /// hard stop is `osteps+3`), `wv=<f32>` (saturation bound on every
    /// accumulated value; default `80`), `pess=<f32>` (pessimism weight
    /// on the root mean, in `[0, 1]`; default `0` is today's mean;
    /// `1` is the worst determinization), `tt=0|1` (per-decision
    /// transposition table; default `1`), `alloc=root|fair` (how the
    /// node cap is spent across `(root, candidate)` pairs; default
    /// `fair` = per-pair share; `alloc=root` restores the pre-#46
    /// root-major spend), `info=fair|draws|all` (what the search is
    /// allowed to know; default `fair` = own deck resampled (hand
    /// untouched), opponent resampled — a human with open decklists;
    /// `draws` restores the pre-flip path (own draw order exact);
    /// `all` is the true state and builds one root regardless of `k`),
    /// and `w_shadows=`,
    /// `w_earth=`, `w_faith=`, `w_rally=`, `w_boost=`, `w_need=`,
    /// `w_lw=` (f32; only meaningful with `value=v1`).
    ///
    /// `Err` names the offending token: unknown policy, unknown key, or bad
    /// number.
    pub fn parse_spec(s: &str) -> Result<AnyPolicy, String> {
        let s = s.trim();
        match s {
            "random" => Ok(AnyPolicy::Random(Random)),
            "first-legal" => Ok(AnyPolicy::FirstLegal(FirstLegal)),
            "h0" => Ok(AnyPolicy::H0(H0::default())),
            "h0-fast" => Ok(AnyPolicy::H0(H0::fast())),
            other if other.starts_with("h0:") => parse_h0_params(&other[3..]).map(AnyPolicy::H0),
            other => Err(format!("unknown policy '{other}'")),
        }
    }

    /// Canonical form: `"random"`, `"first-legal"`, `"h0"`, `"h0-fast"`, or
    /// `"h0:depth=…,beam=…,k=…,nodes=…"` plus `value=v0` / `value=v1` or
    /// `value=net,net=<path>` (the built-in net is the default and is not
    /// printed), non-default
    /// `odepth` / `obeam`, `olethal=0` / non-default `osteps` when set,
    /// `oevo=0` when the evolve branch is off, non-default `wv`,
    /// non-default `pess`, `tt=0` when the table is off, `alloc=root`
    /// when the allocator is the pre-#46 root-major spend, `info=draws`
    /// / `info=all` when the information regime is not the default
    /// `fair`, and any non-default weight.
    /// `"h0"` still round-trips to `"h0"`.
    pub fn spec(&self) -> String {
        match self {
            AnyPolicy::Random(_) => "random".to_string(),
            AnyPolicy::FirstLegal(_) => "first-legal".to_string(),
            AnyPolicy::H0(h) => h0_spec(h),
        }
    }
}

fn h0_spec(h: &H0) -> String {
    if h0_fields_eq(h, &H0::default()) {
        return "h0".to_string();
    }
    if h0_fields_eq(h, &H0::fast()) {
        return "h0-fast".to_string();
    }
    let def = H0::default();
    let mut parts: Vec<String> = Vec::new();
    if h.depth != def.depth
        || h.beam != def.beam
        || h.determinizations != def.determinizations
        || h.node_cap != def.node_cap
    {
        parts.push(format!("depth={}", h.depth));
        parts.push(format!("beam={}", h.beam));
        parts.push(format!("k={}", h.determinizations));
        parts.push(format!("nodes={}", h.node_cap));
    }
    match h.value {
        ValueVersion::V0 => parts.push("value=v0".to_string()),
        ValueVersion::V1 => parts.push("value=v1".to_string()),
        ValueVersion::Net => {
            if let Some(path) = &h.net_path {
                parts.push("value=net".to_string());
                parts.push(format!("net={path}"));
            }
        }
    }
    if h.odepth != def.odepth {
        parts.push(format!("odepth={}", h.odepth));
    }
    if h.obeam != def.obeam {
        parts.push(format!("obeam={}", h.obeam));
    }
    if !h.olethal {
        parts.push("olethal=0".to_string());
    }
    if !h.oevo {
        parts.push("oevo=0".to_string());
    }
    if h.osteps != def.osteps {
        parts.push(format!("osteps={}", h.osteps));
    }
    if h.wv != def.wv {
        parts.push(format!("wv={}", h.wv));
    }
    if h.pess != def.pess {
        parts.push(format!("pess={}", h.pess));
    }
    if !h.tt {
        parts.push("tt=0".to_string());
    }
    if h.alloc != Alloc::Fair {
        parts.push("alloc=root".to_string());
    }
    if h.info != Info::Fair {
        parts.push(match h.info {
            Info::Draws => "info=draws".to_string(),
            Info::All => "info=all".to_string(),
            Info::Fair => unreachable!(),
        });
    }
    let w = &h.weights;
    let dw = Weights::default();
    if w.shadows != dw.shadows {
        parts.push(format!("w_shadows={}", w.shadows));
    }
    if w.earth != dw.earth {
        parts.push(format!("w_earth={}", w.earth));
    }
    if w.faith != dw.faith {
        parts.push(format!("w_faith={}", w.faith));
    }
    if w.rally != dw.rally {
        parts.push(format!("w_rally={}", w.rally));
    }
    if w.boost != dw.boost {
        parts.push(format!("w_boost={}", w.boost));
    }
    if w.need != dw.need {
        parts.push(format!("w_need={}", w.need));
    }
    if w.last_words != dw.last_words {
        parts.push(format!("w_lw={}", w.last_words));
    }
    format!("h0:{}", parts.join(","))
}

fn h0_fields_eq(a: &H0, b: &H0) -> bool {
    a.depth == b.depth
        && a.beam == b.beam
        && a.determinizations == b.determinizations
        && a.node_cap == b.node_cap
        && a.value == b.value
        && a.odepth == b.odepth
        && a.obeam == b.obeam
        && a.olethal == b.olethal
        && a.oevo == b.oevo
        && a.osteps == b.osteps
        && a.wv == b.wv
        && a.pess == b.pess
        && a.tt == b.tt
        && a.alloc == b.alloc
        && a.info == b.info
        && a.net_path == b.net_path
        && weights_eq(&a.weights, &b.weights)
}

fn weights_eq(a: &Weights, b: &Weights) -> bool {
    a.shadows == b.shadows
        && a.earth == b.earth
        && a.faith == b.faith
        && a.rally == b.rally
        && a.boost == b.boost
        && a.need == b.need
        && a.last_words == b.last_words
}

fn parse_h0_params(body: &str) -> Result<H0, String> {
    let mut h = H0::default();
    let mut net_path: Option<String> = None;
    if body.is_empty() {
        return Ok(h);
    }
    for token in body.split(',') {
        let token = token.trim();
        if token.is_empty() {
            return Err("unknown key ''".to_string());
        }
        let Some((key, val)) = token.split_once('=') else {
            return Err(format!("unknown key '{token}'"));
        };
        let key = key.trim();
        let val = val.trim();
        match key {
            "depth" => h.depth = parse_num(val)?,
            "beam" => h.beam = parse_num(val)?,
            "k" => h.determinizations = parse_num(val)?,
            "nodes" => h.node_cap = parse_num(val)?,
            "value" => {
                h.value = match val {
                    "v0" => ValueVersion::V0,
                    "v1" => ValueVersion::V1,
                    "net" => ValueVersion::Net,
                    other => return Err(format!("unknown value '{other}'")),
                }
            }
            "net" => {
                if val.is_empty() {
                    return Err("missing key 'net'".to_string());
                }
                net_path = Some(val.to_string());
            }
            "w_shadows" => h.weights.shadows = parse_num(val)?,
            "w_earth" => h.weights.earth = parse_num(val)?,
            "w_faith" => h.weights.faith = parse_num(val)?,
            "w_rally" => h.weights.rally = parse_num(val)?,
            "w_boost" => h.weights.boost = parse_num(val)?,
            "w_need" => h.weights.need = parse_num(val)?,
            "w_lw" => h.weights.last_words = parse_num(val)?,
            "odepth" => h.odepth = parse_num(val)?,
            "obeam" => h.obeam = parse_num(val)?,
            "osteps" => h.osteps = parse_num(val)?,
            "olethal" => {
                h.olethal = match val {
                    "0" => false,
                    "1" => true,
                    other => return Err(format!("unknown value '{other}'")),
                }
            }
            "oevo" => {
                h.oevo = match val {
                    "0" => false,
                    "1" => true,
                    other => return Err(format!("unknown oevo '{other}'")),
                }
            }
            "wv" => h.wv = parse_num(val)?,
            "pess" => {
                let v: f32 = val.parse().map_err(|_| format!("bad pess '{val}'"))?;
                if !(0.0..=1.0).contains(&v) {
                    return Err(format!("pess out of range '{val}'"));
                }
                h.pess = v;
            }
            "tt" => {
                h.tt = match val {
                    "0" => false,
                    "1" => true,
                    other => return Err(format!("unknown value '{other}'")),
                }
            }
            "alloc" => {
                h.alloc = match val {
                    "root" => Alloc::Root,
                    "fair" => Alloc::Fair,
                    other => return Err(format!("unknown alloc '{other}'")),
                }
            }
            "info" => {
                h.info = match val {
                    "fair" => Info::Fair,
                    "draws" => Info::Draws,
                    "all" => Info::All,
                    other => return Err(format!("unknown info '{other}' (fair|draws|all)")),
                }
            }
            other => return Err(format!("unknown key '{other}'")),
        }
    }
    match (h.value, net_path) {
        (ValueVersion::Net, Some(path)) => {
            let net = crate::policy::net::ValueNet::load(&path)?;
            h.net = Some(net);
            h.net_path = Some(path);
        }
        (ValueVersion::Net, None) => {}
        (ValueVersion::V0 | ValueVersion::V1, None) => {
            h.net = None;
        }
        (ValueVersion::V0 | ValueVersion::V1, Some(_)) => {
            return Err("net= requires value=net".to_string());
        }
    }
    Ok(h)
}

fn parse_num<T: std::str::FromStr>(val: &str) -> Result<T, String> {
    val.parse().map_err(|_| format!("bad number '{val}'"))
}

impl PartialEq for AnyPolicy {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Random(_), Self::Random(_)) | (Self::FirstLegal(_), Self::FirstLegal(_)) => true,
            (Self::H0(a), Self::H0(b)) => h0_fields_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for AnyPolicy {}

impl Policy for AnyPolicy {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        match self {
            AnyPolicy::Random(p) => p.choose(db, state, legal, rng),
            AnyPolicy::FirstLegal(p) => p.choose(db, state, legal, rng),
            AnyPolicy::H0(p) => p.choose(db, state, legal, rng),
        }
    }

    fn last_value(&self) -> Option<f32> {
        match self {
            AnyPolicy::Random(p) => p.last_value(),
            AnyPolicy::FirstLegal(p) => p.last_value(),
            AnyPolicy::H0(p) => p.last_value(),
        }
    }
}
