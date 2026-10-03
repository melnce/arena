//! Replay every committed `oracle/traces/**/*.jsonl.gz` against this engine.
//!
//! Green, or the first divergence matches `oracle/known-divergences.json`
//! (same trace, same `i`, same path). `ARENA_ORACLE_STRICT=1` ignores the
//! allowlist. Stale allowlist rows fail the test.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use arena_engine::oracle::{
    replay_trace, replay_trace_stats, Divergence, DivergenceClass, KnownDivergence, ReplayOutcome,
};
use arena_engine::ReplayError;
use flate2::read::GzDecoder;

mod common;
use common::*;

struct Red {
    trace: String,
    i: u32,
    path: String,
    arena: String,
    trace_val: String,
    line: String,
    side: &'static str,
}

#[test]
fn oracle_traces() {
    let root = repo_root();
    let traces_dir = root.join("oracle/traces");
    let allow_path = root.join("oracle/known-divergences.json");
    let strict = std::env::var("ARENA_ORACLE_STRICT").ok().as_deref() == Some("1");

    let allow: Vec<KnownDivergence> = if allow_path.exists() {
        KnownDivergence::parse_list(&fs::read_to_string(&allow_path).expect("allowlist"))
            .expect("allowlist json")
    } else {
        Vec::new()
    };

    let db = load_recorded_db();
    let files = collect_gz(&traces_dir);
    assert!(
        !files.is_empty(),
        "no oracle/traces/**/*.jsonl.gz — generate the set (docs/oracle.md)"
    );

    let mut reds: Vec<Red> = Vec::new();
    let mut matched = vec![false; allow.len()];
    let mut green = 0u32;

    for path in &files {
        let key = trace_key(&traces_dir, path);
        let text = gunzip(path);
        match replay_trace(&db, &text) {
            Ok(ReplayOutcome::Green) => green += 1,
            Ok(ReplayOutcome::Divergence(d)) => {
                if !strict {
                    if let Some(idx) = allow.iter().position(|a| a.matches(&key, d.i, &d.path)) {
                        matched[idx] = true;
                        continue;
                    }
                }
                reds.push(red_from_div(key, d));
            }
            Err(e) => {
                let red = red_from_err(key, e);
                if !strict {
                    if let Some(idx) = allow
                        .iter()
                        .position(|a| a.matches(&red.trace, red.i, &red.path))
                    {
                        matched[idx] = true;
                        continue;
                    }
                }
                reds.push(red);
            }
        }
    }

    let mut stale: Vec<String> = Vec::new();
    if !strict {
        for (entry, hit) in allow.iter().zip(matched.iter()) {
            if !hit {
                stale.push(format!(
                    "stale allowlist: {} i={} {}",
                    entry.trace, entry.i, entry.path
                ));
            }
        }
    }

    for r in &reds {
        eprintln!("{}", r.line);
    }
    for s in &stale {
        eprintln!("{s}");
    }
    eprint_summary(&reds);

    let allowlisted = matched.iter().filter(|m| **m).count();
    eprintln!(
        "oracle: {} traces, {green} green, {allowlisted} allowlisted, {} red, {} stale{}",
        files.len(),
        reds.len(),
        stale.len(),
        if strict { " (STRICT)" } else { "" }
    );

    assert!(
        reds.is_empty() && stale.is_empty(),
        "{} red, {} stale allowlist entries",
        reds.len(),
        stale.len()
    );
}

/// Sum of `replay_trace_stats.compared_lines` over the oracle corpus on `origin/main`
/// (same allowlist, no kept-charge normalisation). Normalisation must not reduce it.
const MAIN_COMPARED_LINES: u32 = 17_995;

#[test]
fn bonus_pp_kept_norm_compares_full_trace_depth() {
    let traces_dir = repo_root().join("oracle/traces");
    let db = load_recorded_db();
    let mut compared_lines = 0u32;
    for path in collect_gz(&traces_dir) {
        let text = gunzip(&path);
        if let Ok((_, stats)) = replay_trace_stats(&db, &text) {
            compared_lines += stats.compared_lines;
        }
    }
    // Without kept-charge normalisation the corpus stopped at the first `bonus_pp`
    // legal diff on 139 traces (~9.6k fewer compared lines). With normalisation
    // every trace is compared through to its first real divergence or terminal.
    assert_eq!(
        compared_lines, MAIN_COMPARED_LINES,
        "oracle corpus compared lines"
    );
    eprintln!("oracle compared lines: {compared_lines}");
}

#[test]
fn bonus_pp_kept_norm_does_not_hide_unrelated_divergence() {
    let db = load_recorded_db();
    let traces_dir = repo_root().join("oracle/traces");
    let path = traces_dir.join("abyss-p8rfn-mirror/trace-20260910-1.jsonl.gz");
    let text = gunzip(&path);
    assert!(
        matches!(replay_trace(&db, &text), Ok(ReplayOutcome::Green)),
        "baseline trace should replay green with kept-charge normalisation"
    );
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect();
    // Line 22 (i=21) is inside a live kept-charge window (EOT with unspent orb at line 21).
    // Edit `shadows` — it sorts after `pp_bonus` in the snapshot diff order.
    let rec: serde_json::Value = serde_json::from_str(&lines[22]).expect("line 22 json");
    let i = rec["i"].as_u64().expect("i") as u32;
    let mut edited = rec.clone();
    edited["state"]["players"]["b"]["shadows"] = serde_json::json!(99);
    lines[22] = edited.to_string();
    let edited_text = lines.join("\n") + "\n";
    let out = replay_trace(&db, &edited_text).expect("replay edited");
    match out {
        ReplayOutcome::Divergence(d) => {
            assert_eq!(d.i, i);
            assert!(
                d.path.contains("shadows"),
                "unrelated edit must still diverge, got {}",
                d.path
            );
        }
        ReplayOutcome::Green => panic!("edited shadows must not be hidden by bonus_pp norm"),
    }
}

/// Without the recording-time overlay, patched cards in royal-nattui traces diverge on data.
#[test]
fn sabotage_royal_nattui_without_overlay_diverges() {
    let traces_dir = repo_root().join("oracle/traces");
    let path = traces_dir.join("royal-nattui-mirror/trace-20260910-0.jsonl.gz");
    let text = gunzip(&path);
    let db = load_db();
    match replay_trace(&db, &text) {
        Ok(ReplayOutcome::Divergence(_)) | Err(_) => {}
        Ok(ReplayOutcome::Green) => panic!("patched stats without overlay must not replay green"),
    }
}

#[test]
fn known_divergence_classes_include_old_rule() {
    // Allowlist contract: docs/oracle.md. `old-rule` = the old engine
    // disagrees with an owner ruling or the rulebook; arena is right.
    // Never used for an emitter representation defect (`old-emitter`).
    let text = fs::read_to_string(repo_root().join("oracle/known-divergences.json"))
        .expect("known-divergences.json");
    let rows = KnownDivergence::parse_list(&text).expect("allowlist json");
    assert!(
        rows.iter().any(|r| r.class == DivergenceClass::OldRule),
        "old-rule must appear in the allowlist"
    );
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

fn gunzip(path: &Path) -> String {
    let f = fs::File::open(path).unwrap_or_else(|e| panic!("open {}: {e}", path.display()));
    let mut dec = GzDecoder::new(f);
    let mut text = String::new();
    dec.read_to_string(&mut text)
        .unwrap_or_else(|e| panic!("gunzip {}: {e}", path.display()));
    text
}

fn red_from_div(trace: String, d: Divergence) -> Red {
    let line = d.report_line(&trace);
    Red {
        trace,
        i: d.i,
        path: d.path.clone(),
        arena: d.arena.clone(),
        trace_val: d.trace.clone(),
        side: d.side(),
        line,
    }
}

fn red_from_err(trace: String, e: ReplayError) -> Red {
    match e {
        ReplayError::Diverge {
            i,
            path,
            arena,
            trace: tv,
        } => {
            let line = format!("{trace} i={i} {path} arena={arena} trace={tv} action= cards=");
            let side = if path.starts_with("players.a.") {
                "a"
            } else if path.starts_with("players.b.") {
                "b"
            } else {
                "-"
            };
            Red {
                trace,
                i,
                path,
                arena,
                trace_val: tv,
                line,
                side,
            }
        }
        ReplayError::Illegal {
            i,
            action,
            legal,
            source,
        } => {
            let line = format!(
                "{trace} i={i} illegal arena={legal} trace={source} action={action} cards="
            );
            Red {
                trace,
                i,
                path: "illegal".into(),
                arena: legal,
                trace_val: source.to_string(),
                line,
                side: "-",
            }
        }
        ReplayError::OracleAt { i, err } => {
            let line = format!("{trace} i={i} error arena= trace={err} action= cards=");
            Red {
                trace,
                i,
                path: "error".into(),
                arena: String::new(),
                trace_val: err.to_string(),
                line,
                side: "-",
            }
        }
        ReplayError::Oracle(source) => {
            let line = format!("{trace} i=0 error arena= trace={source} action= cards=");
            Red {
                trace,
                i: 0,
                path: "error".into(),
                arena: String::new(),
                trace_val: source.to_string(),
                line,
                side: "-",
            }
        }
        other => {
            let line = format!("{trace} i=0 error arena= trace={other} action= cards=");
            Red {
                trace,
                i: 0,
                path: "error".into(),
                arena: String::new(),
                trace_val: other.to_string(),
                line,
                side: "-",
            }
        }
    }
}

fn eprint_summary(reds: &[Red]) {
    if reds.is_empty() {
        return;
    }
    let mut groups: Vec<(String, usize, &'static str, usize)> = Vec::new();
    for (idx, r) in reds.iter().enumerate() {
        if let Some(g) = groups.iter_mut().find(|g| g.0 == r.path) {
            g.1 += 1;
        } else {
            groups.push((r.path.clone(), 1, r.side, idx));
        }
    }
    groups.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    eprintln!("by path (count, side, example):");
    for (path, count, side, idx) in groups {
        let ex = &reds[idx];
        eprintln!(
            "  {count}  {side}  {path}  e.g. {} arena={} trace={}",
            ex.trace, ex.arena, ex.trace_val
        );
    }
}
