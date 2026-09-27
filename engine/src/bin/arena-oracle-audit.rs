//! Replay oracle traces, classify first divergences, and rebuild the allowlist.
//!
//! ```text
//! arena-oracle-audit [--write-allowlist PATH] [--compare-main PATH] [--audit PATH]
//! ```
//!
//! Default: audit JSON on stdout. `--write-allowlist` emits `oracle/known-divergences.json`.
//! `--compare-main` reads a main-branch allowlist and writes `oracle/removed-divergences-notes.md`.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use arena_engine::oracle::{
    replay_trace_with_config, DivergenceClass, KnownDivergence, ReplayConfig, ReplayReport,
};
use arena_engine::{CardDb, ReplayError};
use flate2::read::GzDecoder;
use serde::Serialize;
use serde_json::Value;

const SINCE: &str = "2026-09-27";
const SINCE_OLD: &str = "2026-09-10";

#[derive(Debug, Clone, Serialize)]
struct AuditRow {
    trace: String,
    i: u32,
    path: String,
    class: DivergenceClass,
    reason: String,
    reconverged: bool,
    action_summary: String,
    played_card: Option<PlayedCard>,
    arena: String,
    trace_val: String,
    cards: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PlayedCard {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
struct TraceLine {
    i: u32,
    action: Value,
    state: Value,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("arena-oracle-audit: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let mut write_allowlist: Option<PathBuf> = None;
    let mut compare_main: Option<PathBuf> = None;
    let mut audit_out: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--write-allowlist" => {
                i += 1;
                write_allowlist = Some(PathBuf::from(args.get(i).ok_or("--write-allowlist PATH")?));
            }
            "--compare-main" => {
                i += 1;
                compare_main = Some(PathBuf::from(args.get(i).ok_or("--compare-main PATH")?));
            }
            "--audit" => {
                i += 1;
                audit_out = Some(PathBuf::from(args.get(i).ok_or("--audit PATH")?));
            }
            "--help" | "-h" => {
                eprintln!(
                    "usage: arena-oracle-audit [--write-allowlist PATH] [--compare-main PATH] [--audit PATH]"
                );
                return Ok(());
            }
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }

    let root = repo_root();
    let traces_dir = root.join("oracle/traces");
    let mut db = CardDb::load(&root).map_err(|e| e.to_string())?;
    db.load_extra_dir(root.join("engine/tests/fixtures/cards"))
        .map_err(|e| e.to_string())?;

    let mut rows = Vec::new();
    let mut unclassified = Vec::new();
    for path in collect_gz(&traces_dir) {
        let key = trace_key(&traces_dir, &path);
        let text = gunzip(&path)?;
        let lines = parse_trace_lines(&text)?;
        match audit_one(&db, &key, &text, &lines) {
            Ok(Some(row)) => rows.push(row),
            Ok(None) => {}
            Err(e) => unclassified.push(format!("{key}: {e}")),
        }
    }

    rows.sort_by(|a, b| (&a.trace, a.i, &a.path).cmp(&(&b.trace, b.i, &b.path)));

    let audit_json = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    if let Some(path) = audit_out {
        fs::write(&path, &audit_json).map_err(|e| e.to_string())?;
    } else {
        println!("{audit_json}");
    }

    if !unclassified.is_empty() {
        eprintln!("unclassified ({}):", unclassified.len());
        for u in &unclassified {
            eprintln!("  {u}");
        }
    }

    let allowlist: Vec<KnownDivergence> = rows
        .iter()
        .map(|r| KnownDivergence {
            trace: r.trace.clone(),
            i: r.i,
            path: r.path.clone(),
            class: r.class.clone(),
            reason: r.reason.clone(),
            since: if r.reason.contains("2026-09-10") || r.since_tag() == SINCE_OLD {
                SINCE_OLD.into()
            } else {
                SINCE.into()
            },
        })
        .collect();

    if let Some(path) = write_allowlist {
        let json = serde_json::to_string_pretty(&allowlist).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        eprintln!(
            "wrote {} allowlist entries to {}",
            allowlist.len(),
            path.display()
        );
        print_mechanism_summary(&allowlist);
    }

    if let Some(main_path) = compare_main {
        let main_text = fs::read_to_string(&main_path).map_err(|e| e.to_string())?;
        let main_rows = KnownDivergence::parse_list(&main_text)?;
        let notes = removed_notes(&db, &traces_dir, &main_rows, &allowlist);
        let notes_path = root.join("oracle/removed-divergences-notes.md");
        fs::write(&notes_path, notes).map_err(|e| e.to_string())?;
        eprintln!("wrote {}", notes_path.display());
    }

    Ok(())
}

impl AuditRow {
    fn since_tag(&self) -> &str {
        if self.reason.contains("Adahime")
            || self.reason.contains("Earth Sigil")
            || self.reason.contains("Exact Copy")
            || self.reason.contains("owner ruling 2026-09-10")
            || self.reason.contains("official Q&A")
            || self.reason.contains("official glossary")
        {
            SINCE_OLD
        } else {
            SINCE
        }
    }
}

fn audit_one(
    db: &CardDb,
    key: &str,
    text: &str,
    lines: &[TraceLine],
) -> Result<Option<AuditRow>, String> {
    let FindingParts {
        i,
        path,
        arena,
        trace_val,
        action,
        cards,
    } = match first_finding(db, text) {
        Ok(None) => return Ok(None),
        Ok(Some(f)) => f,
        Err(e) => return Err(e.to_string()),
    };
    let reconverged = replay_trace_with_config(
        db,
        text,
        ReplayConfig {
            continue_on_divergence: true,
        },
    )
    .map(|r| r.reconverged_after_first)
    .unwrap_or(false);

    let ctx = TraceCtx::new(db, lines, i, &path, &action);
    let (class, reason) = classify(db, key, i, &path, &arena, &trace_val, &action, &cards, &ctx)?;
    let played = ctx.played_card().map(|(id, name)| PlayedCard { id, name });
    Ok(Some(AuditRow {
        trace: key.to_string(),
        i,
        path,
        class,
        reason,
        reconverged,
        action_summary: action_summary(&action),
        played_card: played,
        arena,
        trace_val,
        cards,
    }))
}

struct FindingParts {
    i: u32,
    path: String,
    arena: String,
    trace_val: String,
    action: Value,
    cards: Vec<String>,
}

fn first_finding(db: &CardDb, text: &str) -> Result<Option<FindingParts>, ReplayError> {
    match replay_trace_with_config(db, text, ReplayConfig::default()) {
        Ok(ReplayReport { divergences, .. }) if divergences.is_empty() => Ok(None),
        Ok(ReplayReport { divergences, .. }) => {
            let d = &divergences[0];
            Ok(Some(FindingParts {
                i: d.i,
                path: d.path.clone(),
                arena: d.arena.clone(),
                trace_val: d.trace.clone(),
                action: d.action.clone(),
                cards: d.cards.clone(),
            }))
        }
        Err(e) => Ok(Some(err_parts(e)?)),
    }
}

fn err_parts(e: ReplayError) -> Result<FindingParts, ReplayError> {
    match e {
        ReplayError::Diverge {
            i,
            path,
            arena,
            trace,
        } => Ok(FindingParts {
            i,
            path,
            arena,
            trace_val: trace,
            action: Value::Null,
            cards: Vec::new(),
        }),
        ReplayError::Illegal {
            i,
            action,
            legal,
            source,
        } => Ok(FindingParts {
            i,
            path: "illegal".into(),
            arena: legal,
            trace_val: source.to_string(),
            action: serde_json::from_str(&action).unwrap_or(Value::String(action)),
            cards: Vec::new(),
        }),
        ReplayError::OracleAt { i, err } => Ok(FindingParts {
            i,
            path: "error".into(),
            arena: String::new(),
            trace_val: err.to_string(),
            action: Value::Null,
            cards: Vec::new(),
        }),
        ReplayError::Oracle(err) => Ok(FindingParts {
            i: 0,
            path: "error".into(),
            arena: String::new(),
            trace_val: err.to_string(),
            action: Value::Null,
            cards: Vec::new(),
        }),
        other => Err(other),
    }
}

struct TraceCtx<'a> {
    db: &'a CardDb,
    lines: &'a [TraceLine],
    i: u32,
    path: &'a str,
    action: &'a Value,
}

impl<'a> TraceCtx<'a> {
    fn new(
        db: &'a CardDb,
        lines: &'a [TraceLine],
        i: u32,
        path: &'a str,
        action: &'a Value,
    ) -> Self {
        Self {
            db,
            lines,
            i,
            path,
            action,
        }
    }

    fn line_at(&self, i: u32) -> Option<&TraceLine> {
        self.lines.iter().find(|l| l.i == i)
    }

    fn state_at(&self, i: u32) -> Option<&Value> {
        self.line_at(i).map(|l| &l.state)
    }

    fn recent_play(&self, max_back: u32) -> Option<(String, String)> {
        for back in 0..=max_back {
            let idx = self.i.saturating_sub(back);
            if idx == 0 {
                break;
            }
            if let Some(l) = self.line_at(idx) {
                if let Some(id) = action_play_card(&l.action) {
                    return Some((id.clone(), card_name(self.db, &id)));
                }
            }
        }
        None
    }

    fn played_card(&self) -> Option<(String, String)> {
        action_play_card(self.action)
            .map(|id| (id.clone(), card_name(self.db, &id)))
            .or_else(|| self.recent_play(6))
    }

    fn field_card_at_path(&self) -> Option<String> {
        let rest = self.path.strip_prefix("players.")?;
        let (side, rest) = rest.split_once('.')?;
        let after = rest.strip_prefix("field[")?;
        let (idx, _) = after.split_once(']')?;
        let idx: usize = idx.parse().ok()?;
        let st = self.state_at(self.i)?;
        st.get("players")?
            .get(side)?
            .get("field")?
            .get(idx)?
            .get("card")?
            .as_str()
            .map(str::to_string)
    }

    fn cemetery_card_id(&self) -> Option<String> {
        self.path.split("cemetery.").nth(1).map(str::to_string)
    }

    fn countdown_at_path(&self, i: u32) -> Option<String> {
        wog_countdown_at(self, i, self.path)
    }

    fn wog_field(&self) -> Option<(String, usize)> {
        let rest = self.path.strip_prefix("players.")?;
        let (side, rest) = rest.split_once('.')?;
        let after = rest.strip_prefix("field[")?;
        let (idx, _) = after.split_once(']')?;
        let idx: usize = idx.parse().ok()?;
        Some((side.to_string(), idx))
    }

    fn recent_evolve(&self) -> Option<String> {
        for back in 0..=8 {
            let idx = self.i.saturating_sub(back);
            if idx == 0 {
                break;
            }
            if let Some(l) = self.line_at(idx) {
                if let Some(slot) = l.action.get("evolve") {
                    let side = slot.get("player")?.as_str()?;
                    let slot = slot.get("slot")?.as_u64()? as usize;
                    let st = &l.state;
                    let id = st
                        .get("players")?
                        .get(side)?
                        .get("field")?
                        .get(slot)?
                        .get("card")?
                        .as_str()?;
                    return Some(id.to_string());
                }
            }
        }
        None
    }
}

fn classify(
    db: &CardDb,
    trace: &str,
    i: u32,
    path: &str,
    arena: &str,
    trace_val: &str,
    action: &Value,
    cards: &[String],
    ctx: &TraceCtx<'_>,
) -> Result<(DivergenceClass, String), String> {
    if path == "error" {
        return classify_error(db, trace, i, trace_val);
    }
    if path == "illegal" {
        return Err(format!("illegal at {trace} i={i}: {trace_val}"));
    }
    if path.ends_with(".max_defense") {
        return Ok(classify_max_defense(db, trace, cards, ctx));
    }
    if path.contains(".banished.10031210") || path.contains(".banished.90031210") {
        return Ok(classify_earth_sigil(db, ctx));
    }
    if path.ends_with(".faith") {
        return Ok(classify_faith(db, ctx));
    }
    if path.ends_with(".countdown") {
        return Ok(classify_countdown(db, trace, arena, trace_val, ctx));
    }
    if path.contains(".cemetery.") {
        return Ok(classify_cemetery(db, trace, arena, trace_val, ctx));
    }
    if path.contains(".field[") && trace_val.contains("90074140") {
        return Ok(classify_imari_buddies(db, ctx));
    }
    if path.ends_with(".attack") && trace.starts_with("afnm-minatodao-mirror/") {
        return Ok(classify_exact_copy_attack(db, cards, ctx));
    }
    if path.ends_with(".earth") {
        return Ok(classify_earth_stat(db, ctx));
    }
    if is_whole_field_slot(path) {
        if let Some((play_id, _play_name)) = ctx.recent_play(8) {
            if play_id == "10642310" {
                return Ok((
                    DivergenceClass::OldRule,
                    "owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai, Eld Blades 10644120 discarded by Spilling Red 10642310 summons after the destroy, not before the second selection".into(),
                ));
            }
            if play_id == "10844120" {
                return Ok((
                    DivergenceClass::OldRule,
                    "owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai, Eld Blades 10644120 discarded by Lumiore & Argente 10844120 ('Select 2 cards in your hand and discard them') summons after both discards and the damage, not after the first selection".into(),
                ));
            }
        }
    }
    Err(format!("unclassified {trace} i={i} {path} action={action}"))
}

fn classify_error(
    db: &CardDb,
    trace: &str,
    i: u32,
    trace_val: &str,
) -> Result<(DivergenceClass, String), String> {
    if trace_val.contains("multiset_pick") {
        let repeated = trace_val
            .split("candidates: [")
            .nth(1)
            .and_then(|s| s.strip_suffix(']'))
            .and_then(|s| s.split(',').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "duplicate".into());
        let rep_name = card_name(db, &repeated);
        if trace_val.contains("both chose") {
            return Ok((
                DivergenceClass::OldData,
                format!(
                    "Adahime, Anathema of Death 10754110 prints 'differently named'; the old engine's two multiset_picks both chose {rep_name} ({repeated})"
                ),
            ));
        }
        return Ok((
            DivergenceClass::OldData,
            format!(
                "Adahime, Anathema of Death 10754110 prints 'differently named'; the old engine's second multiset_pick repeated {rep_name} ({repeated})"
            ),
        ));
    }
    if trace.contains("elf-neanisu2-mirror/") {
        return Ok((
            DivergenceClass::OldRule,
            "official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts, so the fifth advance destroys World of Games and Last Words draw; the old engine did not count the enemy card and recorded no draw pick".into(),
        ));
    }
    Err(format!("unclassified error at {trace} i={i}: {trace_val}"))
}

fn classify_max_defense(
    _db: &CardDb,
    trace: &str,
    cards: &[String],
    ctx: &TraceCtx<'_>,
) -> (DivergenceClass, String) {
    let target = cards.first().cloned().unwrap_or_else(|| "follower".into());
    if trace == "ramp-37772-mirror/trace-20260910-2.jsonl" {
        return (
            DivergenceClass::OldRule,
            "owner ruling 2026-09-10: a −0/−4 lowers max_defense by 4; the old engine set it to the current defense".into(),
        );
    }
    if trace.starts_with("royal-nattui-mirror/") {
        return (
            DivergenceClass::OldRule,
            format!(
                "owner ruling 2026-09-10 (like Baal): Gilded Necklace 90021340 'give it +0/+1 and Ward' raises max_defense; the old engine left {target} max at the pre-buff value"
            ),
        );
    }
    if trace.starts_with("abyss-p8rfn-mirror/") {
        return (
            DivergenceClass::OldRule,
            format!(
                "owner ruling 2026-09-10: Baal, Elemental Resonance 10452130 '+1/+1' raises max_defense; the old engine left {target} max at the printed value"
            ),
        );
    }
    if let Some(ev) = ctx.recent_evolve() {
        if ev == "10913110" {
            return (
                DivergenceClass::OldRule,
                format!(
                    "owner ruling 2026-09-10 (like Baal): Virid Lieutenant 10913110 Evolve 'Select another allied follower on the field and give it +1/+1 and Rush' raises max_defense; the old engine left {target} max at the pre-buff value"
                ),
            );
        }
    }
    (
        DivergenceClass::OldRule,
        format!(
            "owner ruling 2026-09-10 (like Baal): Setus & Maisha, Bladerights 10814110 'Give all other allied followers on the field +1/+1' raises max_defense; the old engine left {target} max at the pre-buff value"
        ),
    )
}

fn classify_earth_sigil(db: &CardDb, ctx: &TraceCtx<'_>) -> (DivergenceClass, String) {
    let (play_id, play_name) = ctx
        .recent_play(4)
        .unwrap_or_else(|| ("10031210".into(), card_name(db, "10031210")));
    let holder = if ctx.path.contains("90031210") {
        "Magic Sediment 90031210"
    } else {
        "another Earth Sigil amulet"
    };
    (
        DivergenceClass::OldRule,
        format!(
            "official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing {play_name} {play_id} onto {holder} banishes the older holder (no shadow); the old engine cemeteries it"
        ),
    )
}

fn classify_faith(db: &CardDb, ctx: &TraceCtx<'_>) -> (DivergenceClass, String) {
    let (play_id, play_name) = ctx
        .played_card()
        .unwrap_or_else(|| ("90024320".into(), card_name(db, "90024320")));
    (
        DivergenceClass::OldRule,
        format!(
            "play-time selection: playing {play_name} {play_id} opens its Enhanced pick before Yidmetra Faith ticks; the old engine incremented faith before the pick"
        ),
    )
}

fn wog_countdown_at(ctx: &TraceCtx<'_>, i: u32, path: &str) -> Option<String> {
    let rest = path.strip_prefix("players.")?;
    let (side, rest) = rest.split_once('.')?;
    let after = rest.strip_prefix("field[")?;
    let (idx, _) = after.split_once(']')?;
    let idx: usize = idx.parse().ok()?;
    let st = ctx.state_at(i)?;
    st.get("players")?
        .get(side)?
        .get("field")?
        .get(idx)?
        .get("countdown")
        .map(|v| v.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WogTraceAdvance {
    /// Trace countdown dropped on the play line, before any choose.
    BeforePick,
    /// Trace countdown unchanged through play+choose, drops on a later line.
    AfterPick,
    /// Trace never advanced WoG in the play/pick window (Fanfare removed the cost-3/5 card first).
    Never,
    /// Play with no choose (Combo etc.); trace may or may not advance on the play line.
    NoPick,
}

struct WogPlayWindow {
    play_i: u32,
    play_id: String,
    play_name: String,
    has_pick: bool,
    pick_i: Option<u32>,
    cd_before_play: Option<i32>,
    cd_after_play: Option<i32>,
    cd_after_pick: Option<i32>,
    advance: WogTraceAdvance,
}

fn parse_cd(s: &Option<String>) -> Option<i32> {
    s.as_ref().and_then(|v| v.parse().ok())
}

fn wog_play_window(db: &CardDb, ctx: &TraceCtx<'_>) -> Option<WogPlayWindow> {
    let path = ctx.path;
    if !path.ends_with(".countdown") {
        return None;
    }
    let (play_i, play_id) = (0..=6).find_map(|back| {
        let idx = ctx.i.saturating_sub(back);
        if idx == 0 && back > 0 {
            return None;
        }
        ctx.line_at(idx)
            .and_then(|l| action_play_card(&l.action).map(|id| (idx, id)))
    })?;
    let play_name = card_name(db, &play_id);
    let pick_i = (play_i + 1..=play_i + 2).find(|&pi| {
        ctx.line_at(pi)
            .is_some_and(|l| l.action.get("choose").is_some())
    });
    let has_pick = pick_i.is_some()
        || ctx
            .line_at(ctx.i)
            .is_some_and(|l| l.action.get("choose").is_some());
    let cd_before_play = parse_cd(&wog_countdown_at(ctx, play_i.saturating_sub(1), path));
    let cd_after_play = parse_cd(&wog_countdown_at(ctx, play_i, path));
    let pick_line = pick_i.or_else(|| {
        if ctx
            .line_at(ctx.i)
            .is_some_and(|l| l.action.get("choose").is_some())
        {
            Some(ctx.i)
        } else {
            None
        }
    });
    let cd_before_pick =
        pick_line.and_then(|pi| parse_cd(&wog_countdown_at(ctx, pi.saturating_sub(1), path)));
    let cd_after_pick = pick_line.and_then(|pi| parse_cd(&wog_countdown_at(ctx, pi, path)));
    let cd_after_pick_next =
        pick_line.and_then(|pi| parse_cd(&wog_countdown_at(ctx, pi + 1, path)));
    let advance = if !has_pick {
        WogTraceAdvance::NoPick
    } else if cd_before_play.is_some() && cd_after_play.is_some() && cd_after_play < cd_before_play
    {
        // Trace countdown dropped on the play line, before the choose at pick_i.
        WogTraceAdvance::BeforePick
    } else if cd_before_pick.is_some() && cd_after_pick.is_some() && cd_after_pick < cd_before_pick
    {
        // Trace countdown dropped on the pick line (i−1 → i).
        WogTraceAdvance::AfterPick
    } else if cd_after_pick.is_some()
        && cd_after_pick_next.is_some()
        && cd_after_pick_next < cd_after_pick
    {
        // Trace countdown dropped immediately after the pick (i → i+1).
        WogTraceAdvance::AfterPick
    } else if cd_before_play.is_some()
        && cd_after_pick
            .or(cd_after_play)
            .zip(cd_before_play)
            .is_some_and(|(after, before)| after == before)
    {
        // Unchanged through the play/pick window — Fanfare removed the cost match first.
        WogTraceAdvance::Never
    } else {
        WogTraceAdvance::AfterPick
    };
    Some(WogPlayWindow {
        play_i,
        play_id,
        play_name,
        has_pick,
        pick_i: pick_line,
        cd_before_play,
        cd_after_play,
        cd_after_pick,
        advance,
    })
}

fn wog_fanfare_detail(play_id: &str) -> &'static str {
    match play_id {
        "10514120" => "Miroku mode 2, 3-damage split",
        "10914110" => "Magachiyo Fanfare, 4 damage to the picked follower",
        _ => "Fanfare",
    }
}

fn classify_wog_countdown(
    db: &CardDb,
    arena: &str,
    trace_val: &str,
    ctx: &TraceCtx<'_>,
) -> (DivergenceClass, String) {
    let w = wog_play_window(db, ctx).unwrap_or_else(|| WogPlayWindow {
        play_i: ctx.i,
        play_id: ctx.played_card().map(|(id, _)| id).unwrap_or_default(),
        play_name: ctx.played_card().map(|(_, n)| n).unwrap_or_default(),
        has_pick: false,
        pick_i: None,
        cd_before_play: None,
        cd_after_play: None,
        cd_after_pick: None,
        advance: WogTraceAdvance::NoPick,
    });
    let (play_id, play_name) = (w.play_id, w.play_name);

    if play_id == "10811130" {
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: Moelle, Gloomy Maiden 10811130 Fanfare return pick locks before World of Games 10503210 Last Words draw; the old engine drew first (countdown arena={arena} trace={trace_val})"
            ),
        );
    }

    if play_id == "10913310" {
        let cd = w
            .cd_before_play
            .zip(w.cd_after_play)
            .map(|(a, b)| format!("{a}→{b}"))
            .unwrap_or_else(|| format!("trace={trace_val}"));
        return (
            DivergenceClass::OldRule,
            format!(
                "E39 / play-time selection: playing Crimson Incense {play_id} spell target pick opens before World of Games 10503210 play reaction; the old engine advanced WoG before the pick (trace countdown {cd} at i={})",
                w.play_i
            ),
        );
    }

    if play_id == "10914110" && !w.has_pick {
        return (
            DivergenceClass::OldRule,
            format!(
                "E39: playing {play_name} {play_id} with Combo — no pick; arena judges World of Games 10503210 at play (counts same-cost cards per official Q&A, countdown arena={arena} trace={trace_val}); the old engine resolved Fanfare first (destroyed the only other cost-3 follower) and never advanced WoG"
            ),
        );
    }

    if w.has_pick && w.advance == WogTraceAdvance::Never {
        let detail = wog_fanfare_detail(&play_id);
        let cd = w
            .cd_after_pick
            .or(w.cd_after_play)
            .map(|c| c.to_string())
            .unwrap_or_else(|| trace_val.to_string());
        return (
            DivergenceClass::OldRule,
            format!(
                "E39: branch advances World of Games 10503210 after {play_name} {play_id}'s pick before Fanfare body; the old engine resolved Fanfare first ({detail}) destroying the only other same-cost card and never advanced WoG (trace countdown {cd} through the pick at i={})",
                w.pick_i.unwrap_or(ctx.i)
            ),
        );
    }

    if w.has_pick && w.advance == WogTraceAdvance::BeforePick {
        let cd = w
            .cd_before_play
            .zip(w.cd_after_play)
            .map(|(a, b)| format!("{a}→{b}"))
            .unwrap_or_else(|| trace_val.to_string());
        return (
            DivergenceClass::OldRule,
            format!(
                "E39 / play-time selection: playing {play_name} {play_id} pick opens before World of Games 10503210 play reaction; the old engine advanced WoG before the pick (trace countdown {cd} at i={})",
                w.play_i
            ),
        );
    }

    (
        DivergenceClass::OldRule,
        format!(
            "play-time selection: playing {play_name} {play_id} opens its pick before World of Games 10503210 spell-play advance; the old engine advanced WoG (countdown arena={arena} trace={trace_val}) before the pick"
        ),
    )
}

fn classify_countdown(
    db: &CardDb,
    _trace: &str,
    arena: &str,
    trace_val: &str,
    ctx: &TraceCtx<'_>,
) -> (DivergenceClass, String) {
    let slot_card = ctx.field_card_at_path().unwrap_or_default();
    let slot_name = card_name(db, &slot_card);
    let (play_id, play_name) = ctx.played_card().unwrap_or_default();

    if slot_card == "90021210" {
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: playing {play_name} {play_id} opens its pick before Dread Pirate's Flag 90021210 spell-advance; the old engine advanced the flag (countdown arena={arena} trace={trace_val}) before the pick"
            ),
        );
    }

    if slot_card == "10503210" {
        return classify_wog_countdown(db, arena, trace_val, ctx);
    }

    (
        DivergenceClass::OldRule,
        format!(
            "play-time selection: {slot_name} {slot_card} countdown at {} diverges after playing {play_name} {play_id} (arena={arena} trace={trace_val})",
            ctx.path
        ),
    )
}

fn classify_cemetery(
    db: &CardDb,
    trace: &str,
    arena: &str,
    trace_val: &str,
    ctx: &TraceCtx<'_>,
) -> (DivergenceClass, String) {
    let cem_id = ctx.cemetery_card_id().unwrap_or_default();
    let cem_name = card_name(db, &cem_id);
    let (play_id, play_name) = ctx.recent_play(8).unwrap_or_default();

    if play_id == "10642310" {
        if cem_id == "90044330" {
            return (
                DivergenceClass::OldRule,
                format!(
                    "owner ruling 2026-09-10: Depths of the Eld Blades 90044330 discarded by Spilling Red 10642310 ('When this card is discarded, deal 1 damage to the enemy leader and restore 1 defense to your leader') resolves after the destroy, not before the second selection"
                ),
            );
        }
        if cem_id == "10644120" {
            return (
                DivergenceClass::OldRule,
                format!(
                    "owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai, Eld Blades 10644120 discarded by Spilling Red 10642310 summons after the destroy, not before the second selection"
                ),
            );
        }
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: Spilling Red 10642310 destroy/discard picks lock at play; {cem_name} {cem_id} cemetery count shifts because the old engine deferred picks (arena={arena} trace={trace_val})"
            ),
        );
    }

    if play_id == "10844120" {
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: Lumiore & Argente 10844120 hand-discard picks lock at play; {cem_name} {cem_id} cemetery timing differs because the old engine discarded before opening the second pick (arena={arena} trace={trace_val})"
            ),
        );
    }

    if play_id == "10854110" {
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: Itsurugi & Taketsumi 10854110 Fanfare mode pick locks at play before play reactions; {cem_name} {cem_id} cemetery count differs (arena={arena} trace={trace_val})"
            ),
        );
    }

    if play_id == "10733110" || cem_id == "10503210" {
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: playing {play_name} {play_id} shifts {cem_name} {cem_id} cemetery timing relative to World of Games 10503210 reactions (arena={arena} trace={trace_val})"
            ),
        );
    }

    if trace.contains("royal") && cem_id == "90021210" {
        return (
            DivergenceClass::OldRule,
            format!(
                "play-time selection: playing {play_name} {play_id} advances Dread Pirate's Flag 90021210 after its pick; the old engine cemeteried the flag earlier (arena={arena} trace={trace_val})"
            ),
        );
    }

    (
        DivergenceClass::OldRule,
        format!(
            "play-time selection: playing {play_name} {play_id} shifts {cem_name} {cem_id} cemetery timing (arena={arena} trace={trace_val})"
        ),
    )
}

fn classify_imari_buddies(_db: &CardDb, ctx: &TraceCtx<'_>) -> (DivergenceClass, String) {
    let (play_id, play_name) = ctx.played_card().unwrap_or_default();
    (
        DivergenceClass::OldRule,
        format!(
            "play-time selection: {play_name} {play_id} pick excludes Imari's Little Buddies 90074140 summoned by Imari, Dewdrop 10574120 play reaction; the old engine offered the buddy as a candidate"
        ),
    )
}

fn classify_exact_copy_attack(
    _db: &CardDb,
    cards: &[String],
    ctx: &TraceCtx<'_>,
) -> (DivergenceClass, String) {
    let target = cards.first().cloned().unwrap_or_else(|| "follower".into());
    let (play_id, _) = ctx.played_card().unwrap_or_default();
    (
        DivergenceClass::OldRule,
        format!(
            "rules/official-glossary.md Exact Copy: 'An exact copy of a card retains any damage and effects on the original' — a plain copy does not, and a card in hand is never evolved; Depths of the Eld Axe 90074320 'add a copy of it to your hand'; the old engine played the copy as an evolved statline ({target} after playing {play_id})"
        ),
    )
}

fn classify_earth_stat(_db: &CardDb, ctx: &TraceCtx<'_>) -> (DivergenceClass, String) {
    let (play_id, play_name) = ctx.played_card().unwrap_or_default();
    (
        DivergenceClass::OldRule,
        format!(
            "play-time selection: Sweet Abomination 10733110 Earth Rite pay gates the play-time walk; playing {play_name} {play_id} shifts earth count because picks lock before pay/reactions (arena earth differs)"
        ),
    )
}

fn is_whole_field_slot(path: &str) -> bool {
    let Some(idx) = path.rfind(".field[") else {
        return false;
    };
    path[idx..].ends_with(']') && !path[idx..].contains('.')
}

fn card_name(db: &CardDb, id: &str) -> String {
    if let Some(cid) = arena_engine::card::CardId::parse(id) {
        if let Ok(card) = db.card(cid) {
            return card.name().to_string();
        }
    }
    if let Some(rec) = db.catalog.get(id) {
        return rec.name.clone();
    }
    id.to_string()
}

fn action_play_card(action: &Value) -> Option<String> {
    action
        .get("play")
        .and_then(|p| p.get("card"))
        .and_then(|c| c.as_str())
        .map(str::to_string)
}

fn action_summary(action: &Value) -> String {
    if let Some(p) = action.get("play") {
        return format!(
            "play {} pos {}",
            p.get("card").and_then(|c| c.as_str()).unwrap_or("?"),
            p.get("hand_pos").and_then(|c| c.as_u64()).unwrap_or(0)
        );
    }
    if let Some(c) = action.get("choose") {
        return format!("choose {:?}", c.get("option"));
    }
    if let Some(e) = action.get("evolve") {
        return format!(
            "evolve slot {} super {}",
            e.get("slot").and_then(|c| c.as_u64()).unwrap_or(0),
            e.get("super").and_then(|c| c.as_bool()).unwrap_or(false)
        );
    }
    action.to_string()
}

fn parse_trace_lines(text: &str) -> Result<Vec<TraceLine>, String> {
    let mut out = Vec::new();
    for (ln, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        if ln == 0 {
            continue;
        }
        let v: Value =
            serde_json::from_str(line).map_err(|e| format!("parse line {}: {e}", ln + 1))?;
        out.push(TraceLine {
            i: v.get("i").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
            action: v.get("action").cloned().unwrap_or(Value::Null),
            state: v.get("state").cloned().unwrap_or(Value::Null),
        });
    }
    Ok(out)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_gz(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_gz(dir, &mut out);
    out.sort();
    out
}

fn walk_gz(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    let mut ents: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    ents.sort();
    for p in ents {
        if p.is_dir() {
            walk_gz(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("gz")
            && p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with(".jsonl.gz"))
        {
            out.push(p);
        }
    }
}

fn trace_key(traces_dir: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(traces_dir).unwrap_or(path);
    let s = rel.to_string_lossy().replace('\\', "/");
    s.strip_suffix(".gz").unwrap_or(&s).to_string()
}

fn gunzip(path: &Path) -> Result<String, String> {
    let f = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut dec = GzDecoder::new(f);
    let mut text = String::new();
    dec.read_to_string(&mut text)
        .map_err(|e| format!("gunzip {}: {e}", path.display()))?;
    Ok(text)
}

fn print_mechanism_summary(rows: &[KnownDivergence]) {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for r in rows {
        let key = mechanism_bucket(&r.reason);
        *counts.entry(key).or_default() += 1;
    }
    eprintln!("allowlist by mechanism:");
    for (k, v) in counts {
        eprintln!("  {v:3}  {k}");
    }
}

fn mechanism_bucket(reason: &str) -> String {
    if reason.contains("Adahime") {
        "Adahime multiset_pick (old-data)".into()
    } else if reason.contains("Baal")
        || reason.contains("Virid")
        || reason.contains("Setus")
        || reason.contains("Gilded Necklace")
        || reason.contains("−0/−4")
    {
        "max_defense owner ruling".into()
    } else if reason.contains("Earth Sigil") {
        "Earth Sigil banish".into()
    } else if reason.contains("Exact Copy") {
        "Exact Copy (afnm)".into()
    } else if reason.contains("90074140") {
        "Imari buddies pick pool".into()
    } else if reason.contains("Spilling Red") {
        "Spilling Red play-time picks".into()
    } else if reason.contains("Lumiore & Argente") {
        "Lumiore discard picks".into()
    } else if reason.contains("Itsurugi") {
        "Itsurugi mode at play".into()
    } else if reason.contains("90024320") || reason.contains("Yidmetra Faith") {
        "Enhanced Depths / Yidmetra faith".into()
    } else if reason.contains("Dread Pirate") {
        "Dread Pirate flag countdown".into()
    } else if reason.contains("World of Games") || reason.contains("WoG") {
        "World of Games countdown / E39".into()
    } else if reason.contains("Sweet Abomination") || reason.contains("earth count") {
        "Sweet Abomination earth".into()
    } else if reason.contains("Divine Thunder") {
        "Divine Thunder Q&A".into()
    } else {
        "other play-time selection".into()
    }
}

fn removed_notes(
    db: &CardDb,
    traces_dir: &Path,
    main: &[KnownDivergence],
    new: &[KnownDivergence],
) -> String {
    let new_keys: HashSet<(String, u32, String)> = new
        .iter()
        .map(|r| (r.trace.clone(), r.i, r.path.clone()))
        .collect();
    let mut removed: Vec<&KnownDivergence> = main
        .iter()
        .filter(|r| !new_keys.contains(&(r.trace.clone(), r.i, r.path.clone())))
        .collect();
    removed.sort_by(|a, b| (&a.trace, a.i, &a.path).cmp(&(&b.trace, b.i, &b.path)));
    let removed_count = removed.len();

    let mut categories: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in &removed {
        let cat = removed_category(r);
        let reason = removed_reason(db, traces_dir, r, new);
        categories.entry(cat).or_default().push(format!(
            "- `{}` i={} `{}` — {}",
            r.trace, r.i, r.path, reason
        ));
    }

    let mut out = String::from("# Removed allowlist rows (main → play-time-selection branch)\n\n");
    out.push_str(&format!(
        "Main had **{}** entries; new allowlist has **{}** entries. **{}** main rows are absent (different `(trace, i, path)` or fixed).\n\n",
        main.len(),
        new.len(),
        removed_count
    ));
    for (cat, items) in categories {
        out.push_str(&format!("## {cat} ({})\n\n", items.len()));
        for item in items {
            out.push_str(&item);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

fn removed_reason(
    db: &CardDb,
    traces_dir: &Path,
    r: &KnownDivergence,
    new: &[KnownDivergence],
) -> String {
    if r.path.ends_with(".countdown")
        && (r.reason.contains("World of Games") || r.reason.contains("Divine Thunder"))
    {
        let trace_green = !new.iter().any(|n| n.trace == r.trace);
        let gz = traces_dir.join(r.trace.replace(".jsonl", ".jsonl.gz"));
        if let Ok(text) = gunzip(&gz) {
            if let Ok(lines) = parse_trace_lines(&text) {
                let ctx = TraceCtx::new(db, &lines, r.i, &r.path, &serde_json::Value::Null);
                let (_, detail) = classify_wog_countdown(db, "?", "?", &ctx);
                if trace_green {
                    return format!(
                        "{detail}; play-time selection restores pick-before-advance order — trace now green"
                    );
                }
                let moved = new
                    .iter()
                    .find(|n| n.trace == r.trace)
                    .map(|n| format!("new first divergence i={} `{}`", n.i, n.path))
                    .unwrap_or_else(|| "first divergence moved".into());
                return format!("{detail}; {moved}");
            }
        }
    }
    r.reason.clone()
}

fn removed_category(r: &KnownDivergence) -> String {
    if r.path == "phase" && r.reason.contains("Sweet Abomination") {
        "Sweet Abomination phase divergences — fixed by Effect::Pay gating".into()
    } else if r.reason.contains("Spilling Red") && r.path.contains("field") {
        "Spilling Red — divergence moved from field snapshot to cemetery/play-time path".into()
    } else if r.reason.contains("World of Games") && r.path.contains("countdown") {
        "WoG countdown — superseded by play-time selection rows at new `(i, path)`".into()
    } else if r.reason.contains("Exact Copy") {
        "afnm Exact Copy — same ruling, different trace step after play-time pin".into()
    } else if r.path == "error" {
        "Adahime / Divine Thunder error rows — different step index".into()
    } else if r.path.contains("hand") {
        "E39 hand-card ordering — superseded by play-time selection timing".into()
    } else {
        "Other — ruling unchanged but first divergence key moved or trace now green".into()
    }
}
