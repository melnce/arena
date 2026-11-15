#!/usr/bin/env bash
# Interleaved A/B: edde4b5 vs current branch on the 52-case harness (sweep + served).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WT="$ROOT/../arena-edde4b5-h0-speed"
HEAD_SHA="$(git -C "$ROOT" rev-parse --short HEAD)"
BASE_SHA="edde4b5"

if [[ ! -e "$WT/.git" ]]; then
  git -C "$ROOT" worktree add -f "$WT" "$BASE_SHA"
fi

cp "$ROOT/engine/tests/h0_speed_profile.rs" "$WT/engine/tests/h0_speed_profile.rs"
cp -r "$ROOT/engine/tests/common" "$WT/engine/tests/common" 2>/dev/null || true

export H0_SPEED_BASELINE_DIR="$WT"
export H0_SPEED_HEAD_DIR="$ROOT"
cd "$ROOT"
cargo test --release -p arena-engine --test h0_speed_profile write_interleaved_ab_timing -- --ignored --nocapture