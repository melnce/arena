#!/usr/bin/env bash
# Reduced-scale screens (1024 main + 1024 reverse per candidate). Not a box verdict.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PY="${PY:-$HOME/.venvs/arena/bin/python}"
SP="$("$PY" -c "import site; print(site.getsitepackages()[0])")"
export PYTHONPATH="${SP}:$ROOT/py"
BASE="h0:nodes=8000,horizon=3,k=4,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=/tmp/leaf87q.json,mull=leaf,mulls=256"
OUT="$ROOT/results/levers-reduced-screen"
mkdir -p "$OUT"
run_one() {
  local tag="$1" cand="$2"
  echo "== $tag =="
  "$PY" "$ROOT/py/matchup.py" --policy-a "$cand" --policy-b "$BASE" --games 4 --pool meta --seed 42 \
    --out "$OUT/${tag}-main.json" >/dev/null
  "$PY" "$ROOT/py/matchup.py" --policy-a "$BASE" --policy-b "$cand" --games 4 --pool meta --seed 42 \
    --out "$OUT/${tag}-reverse.json" >/dev/null
  "$PY" -c "
import json, math, sys
sys.path.insert(0, '$ROOT/py')
from runlib import pooled_candidate, rate_ci
main=json.load(open('$OUT/${tag}-main.json'))
rev=json.load(open('$OUT/${tag}-reverse.json'))
p,ci=pooled_candidate(main,rev,tag='$tag')
elo=400*math.log10(p/(1-p)) if 0<p<1 else 0
wall=main['summary']['seconds']+rev['summary']['seconds']
print('$tag pooled', rate_ci(p,ci), f'Elo {elo:+.1f}', f'wall {wall:.0f}s')
"
}
run_one spreadpess0.25 "$BASE,spreadpess=0.25"
run_one wargmax1 "$BASE,wargmax=1"
run_one pess0.25 "$BASE,pess=0.25"
