//! Walk `../cards/**/*.json` (skip `cards/official/`) and emit one bundle
//! plus a git SHA (or `"dev"`) into `OUT_DIR`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cards_root = manifest.join("../cards");
    println!("cargo:rerun-if-changed={}", cards_root.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest.join("../.git/HEAD").display()
    );

    let mut map = serde_json::Map::new();
    if cards_root.exists() {
        walk_json(&cards_root, &cards_root, &mut map);
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bundle_path = out.join("bundle.json");
    let text = serde_json::to_string(&serde_json::Value::Object(map)).expect("bundle json");
    fs::write(&bundle_path, text).expect("write bundle");

    let sha = git_sha(&manifest);
    fs::write(out.join("version.txt"), sha).expect("write version");
}

fn git_sha(manifest: &Path) -> String {
    if let Ok(sha) = env::var("GITHUB_SHA") {
        let t = sha.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(manifest)
        .output()
    {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let t = s.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
    }
    "dev".into()
}

fn walk_json(dir: &Path, cards_root: &Path, map: &mut serde_json::Map<String, serde_json::Value>) {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let Ok(rd) = fs::read_dir(&cur) else {
            continue;
        };
        let mut ents: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        ents.sort();
        for p in ents {
            if p.is_dir() {
                if p.file_name().and_then(|s| s.to_str()) == Some("official") {
                    continue;
                }
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
                let Ok(text) = fs::read_to_string(&p) else {
                    continue;
                };
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                    continue;
                };
                let rel = p
                    .strip_prefix(cards_root)
                    .map(|r| format!("cards/{}", r.display()))
                    .unwrap_or_else(|_| p.display().to_string());
                map.insert(rel.replace('\\', "/"), value);
            }
        }
    }
}
