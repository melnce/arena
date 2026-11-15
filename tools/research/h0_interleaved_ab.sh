#!/usr/bin/env bash
# Interleaved A/B: edde4b5 vs current branch on the 52-case harness (sweep + served).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WT="$ROOT/../arena-edde4b5-h0-speed"
HEAD_SHA="$(git -C "$ROOT" rev-parse --short HEAD)"
BASE_SHA="edde4b5"

if [[ ! -d "$WT/.git" ]]; then
  git -C "$ROOT" worktree add -f "$WT" "$BASE_SHA"
fi

cp "$ROOT/engine/tests/h0_speed_profile.rs" "$WT/engine/tests/h0_speed_profile.rs"
cp -r "$ROOT/engine/tests/common" "$WT/engine/tests/common" 2>/dev/null || true

echo "Building baseline ($BASE_SHA) …"
cargo build --release -p arena-engine --test h0_speed_profile --manifest-path "$WT/Cargo.toml" -q
BASE_EXE="$(find "$WT/target/release/deps" -maxdepth 1 -name 'h0_speed_profile-*' -executable -type f | head -1)"

echo "Building head ($HEAD_SHA) …"
cargo build --release -p arena-engine --test h0_speed_profile --manifest-path "$ROOT/engine/Cargo.toml" -q
HEAD_EXE="$(find "$ROOT/target/release/deps" -maxdepth 1 -name 'h0_speed_profile-*' -executable -type f | head -1)"

export H0_SPEED_BASELINE_EXE="$BASE_EXE"
export H0_SPEED_HEAD_EXE="$HEAD_EXE"
cd "$ROOT"
cargo test --release -p arena-engine --test h0_speed_profile write_interleaved_ab_timing -- --ignored --nocapture