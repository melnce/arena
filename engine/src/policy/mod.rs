//! Player policies: random-legal, first-legal, and H0 (determinized search).
//!
//! This module (and `encode` / `search_key` / `determinize`) must compile for
//! `wasm32-unknown-unknown`: no `std::time::{Instant, SystemTime}`, threads,
//! rayon, or `getrandom` without the js feature. The built-in value model is
//! `include_str!`-embedded and parsed once through `OnceLock`.
//! `ValueNet::load` is still the only `std::fs` user and is never called
//! from wasm (it compiles on wasm32). The node cap is the only search
//! budget. `Policy` is object-safe so WASM can hold `Box<dyn Policy>`.

mod explain;
mod fuse_guard;
mod h0;
mod mulligan;
mod needs;
mod net;
mod record;

use crate::determinize::{HREAD_DELTA, HREAD_EPS_FA, HREAD_EPS_S};

pub use explain::{
    CandidateRecord, ChoosePath, ExplainRecord, LostRerankRecord, PvEnd, PvLeaf, WorldRecord,
};
pub use fuse_guard::fuse_is_noop;
pub use h0::{
    builtin_mulligan, builtin_net, fuse_completion_partner_sets, Alloc, Deal, Info, MullMode,
    SearchStats, ValueVersion, Weights, Wseed, BUILTIN_MULLIGAN_NAME, BUILTIN_NET_NAME, H0,
};
pub use mulligan::{
    deck_fingerprint_counts, deck_fingerprint_player, mulligan_seat, DeckMulligan, MulliganTable,
};
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
    /// `h0-linear-v4`), `net=<path>` (overrides the built-in; only
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
    /// no hard error for other combinations), `okill=<u32>` (bitmask of
    /// extra kill shapes in the opponent-lethal sweep: bit 1 = Ward break,
    /// 2 = slot-freeing, 4 = play-through; default `0` = off), `omacro=0|1`
    /// (greedy opponent reply credits a `Play` with an evolve on the
    /// just-played slot; default `0` = off), `olsolve=<u32>` (after a
    /// sweep miss, run [`forced_lethal`] with this node budget charged to
    /// the pair cap; default `0` = off), `tkill=<u32>` (on each own-turn
    /// decision — Main, Combat, or Choice — run [`forced_lethal_det`] on the
    /// first determinization before search; default `0` = off),
    /// `tkroll=<u32>` (after a deterministic miss, run [`forced_lethal`] on the
    /// first determinization and confirm under rerolled dice on every root;
    /// only when `tkill>0`; default `0` = off), `fuseguard=0|1` (drop no-op
    /// fuses from the bot's own root candidates and own-turn search; default
    /// `0` = off), `hbcheck=<u32>` (after search
    /// chooses `EndTurn` with a kill attack available, re-score `EndTurn` per
    /// root with a bounded opponent removal search; applies are outside
    /// `node_cap`; default `0` = off), `osteps=<u32>`
    /// (greedy
    /// forced-`EndTurn` step; default `6`;
    /// hard stop is `osteps+3`), `wv=<f32>` (saturation bound on every
    /// accumulated value; default `80`), `pess=<f32>` (pessimism weight
    /// on the root mean, in `[0, 1]`; default `0` is today's mean;
    /// `1` is the worst determinization), `tt=0|1` (per-decision
    /// transposition table; default `1`), `alloc=root|fair` (how the
    /// node cap is spent across `(root, candidate)` pairs; default
    /// `fair` = per-pair share; `alloc=root` restores the pre-#46
    /// root-major spend), `info=open|fair|draws|all` (what the search is
    /// allowed to know; default `open` = deal the opponent only what the
    /// bot cannot rule out; `fair` = own deck resampled (hand untouched),
    /// opponent resampled — a human with open decklists; `draws` restores
    /// the pre-flip path (own side untouched); `all` is the true hidden
    /// state (opponent hand and deck contents exact; RNG reseeded per root)),
    /// and `w_shadows=`,
    /// `w_earth=`, `w_faith=`, `w_rally=`, `w_boost=`, `w_need=`,
    /// `w_lw=` (f32; only meaningful with `value=v1`),
    /// `lcap=<f>` (consensus-lethal node budget as a fraction of `node_cap`,
    /// in `(0, 1]`; default `0.5` is the sweep-10 flip), and `clip=<c>`
    /// (`c ≥ 0`; standardised-input clamp for the learned leaf; default `5`
    /// is the sweep-10 flip; ignored with `value=v0` / `value=v1`),
    /// `mull=builtin|rule|random|<path>` (opening keep policy; default
    /// `builtin` is the embedded `mulligan-v1` table; `rule` is cost ≥ 4
    /// send back; `random` draws one `next_u64()` from the rng passed to
    /// `choose` and sends back slot `i` iff bit `i` is set, `i < hand
    /// length`, at most 4 — deterministic for a seed; `<path>` loads a
    /// keep table at parse time like `net=`).
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
    /// `oevo=0` when the evolve branch is off, non-default `okill`,
    /// `omacro=1` when the greedy reply fuses play→evolve, non-default `olsolve`,
    /// non-default `tkill`, non-default `tkroll`, non-default `hbcheck`,
    /// `fuseguard=1` when on,
    /// non-default `wv`,
    /// non-default `pess`, `tt=0` when the table is off, `alloc=root`
    /// when the allocator is the pre-#46 root-major spend, `info=fair`
    /// / `info=draws` / `info=all` when the information regime is not the
    /// default `open`, `mull=rule` / `mull=random` / `mull=<path>` when the
    /// mulligan mode is not the built-in table, any non-default weight,
    /// non-default `lcap`, non-default `clip`, non-default `horizon`,
    /// non-default `hres`, `deal=block`, `hread=on` or custom weights, and
    /// non-default `hreadm`.
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
    if h.okill != def.okill {
        parts.push(format!("okill={}", h.okill));
    }
    if h.omacro {
        parts.push("omacro=1".to_string());
    }
    if h.olsolve != def.olsolve {
        parts.push(format!("olsolve={}", h.olsolve));
    }
    if h.tkill != def.tkill {
        parts.push(format!("tkill={}", h.tkill));
    }
    if h.tkroll != def.tkroll {
        parts.push(format!("tkroll={}", h.tkroll));
    }
    if h.hbcheck != def.hbcheck {
        parts.push(format!("hbcheck={}", h.hbcheck));
    }
    if h.fuseguard {
        parts.push("fuseguard=1".to_string());
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
    if h.info != Info::Open {
        parts.push(match h.info {
            Info::Fair => "info=fair".to_string(),
            Info::Draws => "info=draws".to_string(),
            Info::All => "info=all".to_string(),
            Info::Open => unreachable!(),
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
    if h.lcap != def.lcap {
        parts.push(format!("lcap={}", h.lcap));
    }
    if h.clip != def.clip {
        parts.push(format!("clip={}", h.clip));
    }
    if !h.fusemacro {
        parts.push("fusemacro=0".to_string());
    }
    if h.bpp1 != def.bpp1 {
        parts.push(format!("bpp1={}", h.bpp1));
    }
    if h.bpp2 != def.bpp2 {
        parts.push(format!("bpp2={}", h.bpp2));
    }
    if h.bppv != def.bppv {
        parts.push(format!("bppv={}", h.bppv));
    }
    if h.horizon != def.horizon {
        parts.push(format!("horizon={}", h.horizon));
    }
    if h.hres != def.hres {
        parts.push(format!("hres={}", h.hres));
    }
    if h.wseed != def.wseed {
        parts.push(match h.wseed {
            Wseed::Off => unreachable!(),
            Wseed::Turn => "wseed=turn".to_string(),
        });
    }
    if h.wbase != def.wbase {
        parts.push(format!("wbase={}", h.wbase.unwrap_or(0)));
    }
    if h.lostrank != def.lostrank {
        parts.push(format!("lostrank={}", h.lostrank));
    }
    if h.deal != def.deal {
        parts.push(match h.deal {
            Deal::Indep => unreachable!(),
            Deal::Block => "deal=block".to_string(),
        });
    }
    if let Some((eps_fa, eps_s, delta)) = h.hread {
        if (eps_fa, eps_s, delta) == (HREAD_EPS_FA, HREAD_EPS_S, HREAD_DELTA) {
            parts.push("hread=on".to_string());
        } else {
            parts.push(format!("hread={eps_fa}/{eps_s}/{delta}"));
        }
    }
    if h.hreadm != def.hreadm {
        parts.push(format!("hreadm={}", h.hreadm));
    }
    match h.mull {
        MullMode::Rule => parts.push("mull=rule".to_string()),
        MullMode::Random => parts.push("mull=random".to_string()),
        MullMode::Table => {
            if let Some(path) = &h.mull_path {
                parts.push(format!("mull={path}"));
            }
        }
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
        && a.okill == b.okill
        && a.omacro == b.omacro
        && a.olsolve == b.olsolve
        && a.tkill == b.tkill
        && a.tkroll == b.tkroll
        && a.hbcheck == b.hbcheck
        && a.fuseguard == b.fuseguard
        && a.osteps == b.osteps
        && a.wv == b.wv
        && a.pess == b.pess
        && a.tt == b.tt
        && a.alloc == b.alloc
        && a.info == b.info
        && a.fusemacro == b.fusemacro
        && a.net_path == b.net_path
        && a.lcap == b.lcap
        && a.clip == b.clip
        && a.mull == b.mull
        && a.mull_path == b.mull_path
        && a.bpp1 == b.bpp1
        && a.bpp2 == b.bpp2
        && a.bppv == b.bppv
        && a.horizon == b.horizon
        && a.hres == b.hres
        && a.wseed == b.wseed
        && a.wbase == b.wbase
        && a.lostrank == b.lostrank
        && a.deal == b.deal
        && a.hread == b.hread
        && a.hreadm == b.hreadm
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
    let mut mull_path: Option<String> = None;
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
            "okill" => {
                let v: u32 = parse_num(val)?;
                h.okill = v;
            }
            "omacro" => {
                h.omacro = match val {
                    "0" => false,
                    "1" => true,
                    other => return Err(format!("unknown omacro '{other}'")),
                }
            }
            "olsolve" => {
                let v: u32 = parse_num(val)?;
                h.olsolve = v;
            }
            "tkill" => {
                let v: u32 = parse_num(val)?;
                h.tkill = v;
            }
            "tkroll" => {
                let v: u32 = parse_num(val)?;
                h.tkroll = v;
            }
            "hbcheck" => {
                let v: u32 = parse_num(val)?;
                h.hbcheck = v;
            }
            "fuseguard" => {
                h.fuseguard = match val {
                    "0" => false,
                    "1" => true,
                    other => return Err(format!("unknown fuseguard '{other}'")),
                };
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
                    "open" => Info::Open,
                    "fair" => Info::Fair,
                    "draws" => Info::Draws,
                    "all" => Info::All,
                    other => return Err(format!("unknown info '{other}' (open|fair|draws|all)")),
                }
            }
            "lcap" => {
                let v: f32 = val.parse().map_err(|_| format!("bad lcap '{val}'"))?;
                if !(v.is_finite() && v > 0.0 && v <= 1.0) {
                    return Err(format!("lcap out of range '{val}'"));
                }
                h.lcap = v;
            }
            "clip" => {
                let v: f32 = val.parse().map_err(|_| format!("bad clip '{val}'"))?;
                if !v.is_finite() || v < 0.0 {
                    return Err(format!("clip out of range '{val}'"));
                }
                h.clip = v;
            }
            "fusemacro" => {
                h.fusemacro = match val {
                    "0" => false,
                    "1" => true,
                    other => return Err(format!("unknown fusemacro '{other}'")),
                }
            }
            "bpp1" => {
                let v: u32 = parse_num(val)?;
                if !(1..=6).contains(&v) {
                    return Err(format!("bpp1 out of range '{val}'"));
                }
                h.bpp1 = v;
            }
            "bpp2" => {
                let v: u32 = parse_num(val)?;
                if v < 6 {
                    return Err(format!("bpp2 out of range '{val}'"));
                }
                h.bpp2 = v;
            }
            "bppv" => {
                let v: f32 = val.parse().map_err(|_| format!("bad bppv '{val}'"))?;
                if !v.is_finite() || v < 0.0 {
                    return Err(format!("bppv out of range '{val}'"));
                }
                h.bppv = v;
            }
            "horizon" => {
                let v: u32 = parse_num(val)?;
                if v > 3 {
                    return Err(format!("horizon out of range '{val}'"));
                }
                h.horizon = v;
            }
            "hres" => {
                let v: u32 = parse_num(val)?;
                if v < 1 {
                    return Err(format!("hres out of range '{val}'"));
                }
                h.hres = v;
            }
            "wseed" => {
                h.wseed = match val {
                    "off" => Wseed::Off,
                    "turn" => Wseed::Turn,
                    other => return Err(format!("unknown wseed '{other}'")),
                };
            }
            "wbase" => {
                let v: u64 = parse_num(val)?;
                h.wbase = Some(v);
            }
            "lostrank" => {
                let v: u32 = parse_num(val)?;
                if v > 1_000_000 {
                    return Err(format!("lostrank out of range '{val}'"));
                }
                h.lostrank = v;
            }
            "deal" => {
                h.deal = match val {
                    "indep" => Deal::Indep,
                    "block" => Deal::Block,
                    other => return Err(format!("unknown deal '{other}'")),
                };
            }
            "hread" => {
                if val == "off" {
                    h.hread = None;
                } else if val == "on" {
                    h.hread = Some((HREAD_EPS_FA, HREAD_EPS_S, HREAD_DELTA));
                } else {
                    let parts: Vec<&str> = val.split('/').collect();
                    if parts.len() != 3 {
                        return Err(format!("bad hread '{val}'"));
                    }
                    let eps_fa: f32 = parse_num(parts[0])?;
                    let eps_s: f32 = parse_num(parts[1])?;
                    let delta: f32 = parse_num(parts[2])?;
                    if !(eps_fa > 0.0 && eps_fa <= 1.0)
                        || !(eps_s > 0.0 && eps_s <= 1.0)
                        || !(delta > 0.0 && delta <= 1.0)
                    {
                        return Err(format!("hread out of range '{val}'"));
                    }
                    h.hread = Some((eps_fa, eps_s, delta));
                }
            }
            "hreadm" => {
                let v: u32 = parse_num(val)?;
                if !(1..=4096).contains(&v) {
                    return Err(format!("hreadm out of range '{val}'"));
                }
                h.hreadm = v;
            }
            "mull" => match val {
                "builtin" => {
                    h.mull = MullMode::Table;
                    h.mull_table = Some(crate::policy::h0::builtin_mulligan());
                    h.mull_path = None;
                }
                "rule" => {
                    h.mull = MullMode::Rule;
                    h.mull_table = None;
                    h.mull_path = None;
                }
                "random" => {
                    h.mull = MullMode::Random;
                    h.mull_table = None;
                    h.mull_path = None;
                }
                "" => return Err("missing key 'mull'".to_string()),
                other => mull_path = Some(other.to_string()),
            },
            other => return Err(format!("unknown key '{other}'")),
        }
    }
    if let Some(path) = mull_path {
        let table = mulligan::MulliganTable::load(&path)?;
        h.mull = MullMode::Table;
        h.mull_table = Some(table);
        h.mull_path = Some(path);
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
