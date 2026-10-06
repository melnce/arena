# Sweep `sweep42`

- tag: `sweep42`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000`
- seed: 1
- engine: `9f187a3faacb63c556268be47b0e87e4fd7bd4db`
- wall: 10460.2s
- stages: screen 1252.0s, final 9208.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1` | 0.550 [0.398, 0.693] | 40 | 0.03 / 0.04 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1` | 0.547 [0.499, 0.596] | 0.605 [0.536, 0.670] | 0.567 [0.527, 0.606] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1 unclear (main) — main 0.547 [0.499, 0.596], reverse 0.605 [0.536, 0.670], pooled 0.567 [0.527, 0.606] (not gated)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1 (unclear (main))
