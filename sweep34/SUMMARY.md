# Sweep `sweep34`

- tag: `sweep34`
- baseline: `h0:tkill=2000`
- seed: 1
- engine: `1bcb402bd9a81c9dcb3f1ac6160b567924f290b9`
- wall: 5287.1s
- stages: screen 1349.5s, final 3937.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:tkill=2000,tkroll=8` | 0.518 [0.474, 0.561] | 512 | 0.59 / 1.00 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:tkill=2000,tkroll=8` | 0.503 [0.482, 0.525] | 0.497 [0.467, 0.528] | 0.501 [0.484, 0.519] |

verdict: h0:tkill=2000,tkroll=8 coin flip — main 0.503 [0.482, 0.525], reverse 0.497 [0.467, 0.528], pooled 0.501 [0.484, 0.519] (not gated)

best: h0:tkill=2000,tkroll=8 (coin flip)
