# Sweep `sweep6`

- tag: `sweep6`
- baseline: `h0:olethal=1,osteps=6`
- seed: 7
- engine: `c913b40a587e81456f32a57f99b9a1b20fe4a345`
- wall: 8016.7s
- stages: screen 3358.6s, final 4658.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:olethal=1,osteps=6,wv=120` | 0.522 [0.492, 0.553] | 1024 | 2.10 / 2.17 | finalist |
| 2 | `h0:olethal=1,osteps=6,wv=200` | 0.522 [0.492, 0.553] | 1024 | 2.54 / 2.17 | finalist |
| 3 | `h0:olethal=1,osteps=6,wv=300` | 0.522 [0.492, 0.553] | 1024 | 2.50 / 2.17 | not in top 2 |
| 6 | `h0:olethal=1,osteps=6,nodes=2400` | 0.515 [0.484, 0.545] | 1024 | 2.15 / 2.17 | not in top 2 |
| 5 | `h0:olethal=1,osteps=6,wv=200,k=8` | 0.501 [0.470, 0.532] | 1024 | 2.14 / 2.17 | not in top 2 |
| 4 | `h0:olethal=1,osteps=6,k=8` | 0.497 [0.467, 0.528] | 1024 | 2.10 / 2.17 | not in top 2 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 1 | `h0:olethal=1,osteps=6,wv=120` | 0.502 [0.487, 0.518] | 0.487 [0.466, 0.509] |
| 2 | `h0:olethal=1,osteps=6,wv=200` | 0.503 [0.488, 0.519] | 0.489 [0.467, 0.510] |

verdict: h0:olethal=1,osteps=6,wv=120 coin flip — main 0.502 [0.487, 0.518], reverse 0.487 [0.466, 0.509]
verdict: h0:olethal=1,osteps=6,wv=200 coin flip — main 0.503 [0.488, 0.519], reverse 0.489 [0.467, 0.510]

best: h0:olethal=1,osteps=6,wv=200 (coin flip)
