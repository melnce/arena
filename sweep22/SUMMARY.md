# Sweep `sweep22`

- tag: `sweep22`
- baseline: `h0`
- seed: 2
- engine: `2d12e25f27ce4276ef6274be8b52e30d1e510f6b`
- wall: 4990.8s
- stages: screen 698.6s, final 4292.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep20/leaf-soup-s0-4.json` | 0.482 [0.439, 0.526] | 512 | 1.44 / 1.39 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep20/leaf-soup-s0-4.json` | 0.478 [0.462, 0.493] | 0.509 [0.487, 0.530] | 0.488 [0.475, 0.500] |

verdict: h0:net=results/sweep20/leaf-soup-s0-4.json worse — main 0.478 [0.462, 0.493], reverse 0.509 [0.487, 0.530], pooled 0.488 [0.475, 0.500] (not gated)

best: h0:net=results/sweep20/leaf-soup-s0-4.json (worse)
