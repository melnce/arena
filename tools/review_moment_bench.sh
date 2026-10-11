#!/usr/bin/env bash
# Wall time for one H0::choose per REVIEW_MOMENT (threads=1), release build.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SPEC="${1:-h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1}"
cargo test --release -p arena-engine --test h0_job review_moment_choose_wall -- --exact --nocapture --include-ignored 2>&1 |
  rg 'review_moment_choose_wall:'
