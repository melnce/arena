# WK lethal audit (h0:okill=7,omacro=1) vs checkpoint 2 (h0), same 512 deals per file

- WK: `results/lethal/wk-f{a,b}.json`, policy `h0:okill=7,omacro=1` both seats, on `a69b248` (v2 default leaf), solver
  budget 50 000, 512 games per file (`--first a` / `--first b`, `--seed 1`, `--by-deficit`); both halves exit 0
  (17 h each, beside sweeps 30, 31, 32 and the tkill audit). Not published (~40 MB each).
- Checkpoint 2: `results/lethal/checkpoint2-f{a,b}.json`, plain `h0` on `8e66bcf` (v2 default leaf), same flags.
- Tools: `../checkpoint2/tools/` — `lethal_report.py wk checkpoint2 WK cp2` (tables 1-5), `table6.py wk checkpoint2 WK cp2`
  (table 6, definition inferred), `handed_eq.py wk checkpoint2 WK cp2` (the decision quantity), `killturns.py cp1=checkpoint
  cp2=checkpoint2 WK=wk`. Intervals: cluster bootstrap on seed (paired: the same resamples for both audits).

## The decision quantity: handed_lethal at equal deficit

```
handed_lethal, cells: deficit bucket x first (as table 3)
  end-turns with a verdict: WK 15448, cp2 15586; hits WK 1158, cp2 1151
  raw rate        WK 0.0750 [0.0726, 0.0774]   cp2 0.0738 [0.0716, 0.0762]
  WK at cp2's deficit mix: 0.0758 [0.0728, 0.0791]
  paired change at equal deficit: +0.0020 [-0.0015, +0.0056]   relative +2.7% [-2.0, +7.7] %
  paired change, raw:             +0.0011 [-0.0016, +0.0039]
handed_lethal, cells: exact deficit [-10, +10] x first
  end-turns with a verdict: WK 15448, cp2 15586; hits WK 1158, cp2 1151
  raw rate        WK 0.0750 [0.0726, 0.0774]   cp2 0.0738 [0.0716, 0.0762]
  WK at cp2's deficit mix: 0.0743 [0.0710, 0.0777]
  paired change at equal deficit: +0.0004 [-0.0032, +0.0042]   relative +0.6% [-4.3, +5.8] %
  paired change, raw:             +0.0011 [-0.0016, +0.0039]
```

## Tables 1-6 and kill turns

```
gate: wk and checkpoint2 share every game_id -> deck pair in both files (512 seeds): OK
end-turns with a verdict: WK 15448, cp2 15586;  kill turns: 1209 / 1211;  auditee-games 2048 / 2048

=== 1. balance: auditee_first share of end-turns per deck ===
  deck                               WK              cp2
  portal-af              0.524 (n= 901)   0.507 (n= 975)
  portal-evo             0.510 (n= 945)   0.515 (n= 946)
  abyss-midrange         0.513 (n= 917)   0.498 (n= 904)
  sword-rally            0.513 (n= 824)   0.505 (n= 856)
  haven-evo              0.506 (n=1009)   0.519 (n=1039)
  dragon-aggro           0.500 (n= 846)   0.510 (n= 831)
  abyss-aggro            0.513 (n= 844)   0.501 (n= 843)
  sword-loot             0.519 (n= 955)   0.526 (n= 928)
  forest-combo           0.530 (n= 970)   0.525 (n= 954)
  dragon-ramp            0.500 (n=1023)   0.514 (n= 997)
  rune-spell             0.494 (n= 960)   0.502 (n= 963)
  portal-cutthroat       0.517 (n=1008)   0.522 (n=1028)
  rune-test-subject      0.510 (n= 945)   0.503 (n= 946)
  haven-amulet           0.534 (n=1159)   0.538 (n=1157)
  rune-crystal           0.525 (n= 965)   0.506 (n= 977)
  haven-kukishiro        0.523 (n=1177)   0.533 (n=1242)

=== 2. win rate per deck (auditee games, both seats; 128 per deck), sorted by WK ===
  deck                     WK    cp2  change    95% CI (paired)  sweep9
  portal-af             0.742  0.695  +0.047   [-0.037, +0.136]  0.725
  sword-rally           0.664  0.633  +0.031   [-0.051, +0.115]  0.665
  haven-evo             0.641  0.672  -0.031   [-0.103, +0.041]  0.603
  portal-evo            0.625  0.672  -0.047   [-0.132, +0.045]  0.703
  abyss-midrange        0.602  0.617  -0.016   [-0.115, +0.083]  0.692
  portal-cutthroat      0.570  0.555  +0.016   [-0.082, +0.114]  0.394
  rune-test-subject     0.539  0.539  +0.000   [-0.093, +0.092]  0.353
  forest-combo          0.523  0.531  -0.008   [-0.113, +0.100]  0.523
  abyss-aggro           0.477  0.453  +0.023   [-0.068, +0.123]  0.598
  dragon-aggro          0.469  0.500  -0.031   [-0.120, +0.056]  0.603
  sword-loot            0.461  0.422  +0.039   [-0.051, +0.134]  0.582
  haven-amulet          0.445  0.320  +0.125   [+0.030, +0.225]  0.306
  dragon-ramp           0.406  0.445  -0.039   [-0.141, +0.059]  0.501
  rune-spell            0.375  0.414  -0.039   [-0.117, +0.044]  0.395
  rune-crystal          0.344  0.352  -0.008   [-0.096, +0.074]  0.219
  haven-kukishiro       0.117  0.180  -0.062   [-0.140, +0.009]  0.138
  spread max-min: WK 0.625 (0.117-0.742), cp2 0.516 (0.180-0.695); change +0.109 [-0.016, +0.200]
  SD across decks: WK 0.146, cp2 0.139;  corr(WK, cp2) +0.951;  corr(WK, sweep9) +0.802

=== 3. handed_lethal, standardised on deficit bucket x first player ===
  -- win axis: sweep 9 per-deck win rate (as on 2026-09-22)
    WK     rate 0.0750  corr(expected,win) -0.510 [-0.595, -0.363]  corr(residual,win) -0.205 [-0.390, -0.006]  split expected 67% / residual 33%
           + early/late stratum:  corr(expected,win) -0.562 [-0.629, -0.442]  corr(residual,win) -0.105 [-0.289, +0.083]  split expected 82% / residual 18%
    cp2    rate 0.0738  corr(expected,win) -0.529 [-0.608, -0.398]  corr(residual,win) -0.075 [-0.234, +0.083]  split expected 85% / residual 15%
           + early/late stratum:  corr(expected,win) -0.567 [-0.636, -0.454]  corr(residual,win) -0.008 [-0.161, +0.135]  split expected 98% / residual 2%
  -- win axis: each audit own per-deck win rate (step 2; shares games with handed_lethal)
    WK     rate 0.0750  corr(expected,win) -0.508 [-0.643, -0.314]  corr(residual,win) -0.443 [-0.642, -0.254]  split expected 48% / residual 52%
           + early/late stratum:  corr(expected,win) -0.517 [-0.642, -0.347]  corr(residual,win) -0.367 [-0.577, -0.182]  split expected 55% / residual 45%
    cp2    rate 0.0738  corr(expected,win) -0.554 [-0.676, -0.379]  corr(residual,win) -0.226 [-0.451, -0.049]  split expected 66% / residual 34%
           + early/late stratum:  corr(expected,win) -0.548 [-0.670, -0.389]  corr(residual,win) -0.185 [-0.410, -0.018]  split expected 70% / residual 30%
  pooled handed rate by deficit bucket (WK | cp2):
    <= -5   0.051 (n=2378)  |  0.045 (n=2400)
    -4..-1  0.041 (n=3795)  |  0.039 (n=3802)
    0       0.012 (n=4888)  |  0.010 (n=4796)
    1..4    0.076 (n=2793)  |  0.084 (n=2964)
    >= 5    0.384 (n=1594)  |  0.367 (n=1624)

=== 4. missed_lethal per kill turn (top/bottom six fixed by sweep 9, as on 2026-09-22) ===
  WK: own tercile cuts 4 / 111
  cp2: own tercile cuts 4 / 169
  common tercile cuts (both audits pooled): 4 / 130
                                               WK                                cp2
  all                     0.1778 [0.1556, 0.2002]            0.1767 [0.1565, 0.1974]   change +0.0011 [-0.0263, +0.0273]
  top six                 0.1429 [0.1147, 0.1725]            0.1518 [0.1260, 0.1805]   change -0.0089 [-0.0431, +0.0282]
  bottom six              0.2506 [0.2071, 0.2939]            0.2526 [0.2114, 0.2915]   change -0.0019 [-0.0569, +0.0520]
  bottom-top           +0.1078 [+0.0546, +0.1623]         +0.1008 [+0.0502, +0.1492]   change +0.0070 [-0.0608, +0.0730]
  tercile 1      3/442 = 0.0068 [0.0000, 0.0153]    3/431 = 0.0070 [0.0000, 0.0160]
  tercile 2     58/379 = 0.1530 [0.1170, 0.1889]   53/362 = 0.1464 [0.1099, 0.1847]
  tercile 3    154/388 = 0.3969 [0.3444, 0.4485]  158/418 = 0.3780 [0.3341, 0.4246]
  counts: top six WK 78/546 | cp2 85/560;  bottom six WK 99/395 | cp2 99/392

=== 5. thrown games: auditee-games with a deterministic missed kill turn that the auditee lost ===
  WK     pooled 28/2048 = 1.37%  [0.88, 1.86] %
  cp2    pooled 23/2048 = 1.12%  [0.73, 1.56] %
  change (paired by seed) +0.24%  [-0.39, +0.88] points
  deck                     WK    cp2   (of 128 auditee-games each)
  rune-test-subject         7      0
  abyss-midrange            5      3
  rune-crystal              5      4
  abyss-aggro               3      2
  rune-spell                2      1
  forest-combo              2      2
  portal-cutthroat          2      1
  dragon-aggro              1      3
  haven-amulet              1      1
  sword-loot                0      2
  portal-evo                0      1
  sword-rally               0      2
  portal-af                 0      0
  haven-evo                 0      1
  dragon-ramp               0      0
  haven-kukishiro           0      0

=== 6. handed kills by the shape of the opponent's winning line ===
handed kills: WK 1158, cp2 1151
  shape (share of handed kills)                          WK                      cp2   change (paired by seed)
  with a follower attack           1121 = 0.968 [0.957, 0.978] 1102 = 0.957 [0.944, 0.969]   +0.011 [-0.003, +0.025]
  attacks, no play                  344 = 0.297 [0.270, 0.323]  320 = 0.278 [0.252, 0.303]   +0.019 [-0.014, +0.053]
  play + attack                     777 = 0.671 [0.644, 0.698]  782 = 0.679 [0.651, 0.706]   -0.008 [-0.043, +0.026]
  no attack                          37 = 0.032 [0.022, 0.043]   49 = 0.043 [0.031, 0.056]   -0.011 [-0.025, +0.003]
  with an evolve                    436 = 0.377 [0.348, 0.405]  452 = 0.393 [0.365, 0.422]   -0.016 [-0.054, +0.022]
  line length WK: mean 4.71, median 4
  line length cp2: mean 5.12, median 5

kill turns (an auditee turn with at least one lethal verdict), same 512 deals x 2 files each:
                                                           cp1                       cp2                        WK
  kill turns                                              1159                      1211                      1209
    deterministic (a luck-free kill)                      1072                      1062                      1085
    chance-dependent only                                   87                       149                       124
    chance-dependent share                0.075 [0.059, 0.091]      0.123 [0.101, 0.145]      0.103 [0.082, 0.124]
  missed: all kill turns                  0.148 [0.128, 0.167]      0.177 [0.156, 0.197]      0.178 [0.156, 0.200]
  missed: deterministic turns             0.094 [0.078, 0.110]      0.096 [0.080, 0.113]      0.112 [0.094, 0.131]
  missed: chance-dependent turns          0.805 [0.716, 0.887]      0.752 [0.677, 0.817]      0.750 [0.673, 0.825]
  counts missed: cp1 det 101/1072, chance 70/87; cp2 det 102/1062, chance 112/149; WK det 122/1085, chance 93/124
decision-level missed_lethal hits (the action taken gave up a kill): cp1 deterministic 155, rng-dependent 137 (of 5184 lethal-verdict decisions); cp2 deterministic 149, rng-dependent 253 (of 5512 lethal-verdict decisions); WK deterministic 204, rng-dependent 184 (of 5496 lethal-verdict decisions)
paired changes (later minus earlier, same resamples):
  cp2 - cp1: chance-dependent kill turns        +62 [+30, +94]
  cp2 - cp1: deterministic kill turns           -10 [-43, +23]
  cp2 - cp1: miss rate, deterministic turns     +0.002 [-0.019, +0.024]
  cp2 - cp1: miss rate, chance-dependent turns  -0.053 [-0.154, +0.046]
  cp2 - cp1: miss rate, all kill turns          +0.029 [+0.003, +0.057]
  cp2 - cp1: deterministic hits                 -6  relative -4% [-28, +31] %
  cp2 - cp1: rng-dependent hits                 +116  relative +85% [+31, +166] %
  WK - cp2: chance-dependent kill turns        -25 [-59, +8]
  WK - cp2: deterministic kill turns           +23 [-12, +56]
  WK - cp2: miss rate, deterministic turns     +0.016 [-0.008, +0.039]
  WK - cp2: miss rate, chance-dependent turns  -0.002 [-0.103, +0.106]
  WK - cp2: miss rate, all kill turns          +0.001 [-0.026, +0.027]
  WK - cp2: deterministic hits                 +55  relative +37% [-1, +85] %
  WK - cp2: rng-dependent hits                 -69  relative -27% [-50, +4] %
```
