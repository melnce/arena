# tkill lethal audit (h0:tkill=2000, v2 leaf pinned) vs checkpoint 2 (h0), same 512 deals per file

- tkill: `results/lethal/tkill-f{a,b}.json`, policy `h0:tkill=2000,net=engine/models/h0-linear-v2.json` both seats, on
  the build of `d5f4310` (started 2026-10-02 03:25, kept its module through the 16:12 and 22:05 rebuilds via the rename
  trick), solver budget 50 000, 512 games per file (`--first a` / `--first b`, `--seed 1`, `--by-deficit`); both halves
  exit 0 (≈ 19.5 h each, beside sweeps 32, 31b, 33, 34, holdback1, net8 and the owner's games). Not published (~39 MB each).
- Checkpoint 2: `results/lethal/checkpoint2-f{a,b}.json`, plain `h0` on `8e66bcf` (v2 default leaf), same flags.
  Checkpoint 1: `checkpoint-f{a,b}.json` (kill-turn table only).
- Tools (`../checkpoint2/tools/`): `lethal_report.py tkill checkpoint2 tkill cp2`; `killturns.py cp1=checkpoint
  cp2=checkpoint2 tkill=tkill`; `missed_split.py tkill checkpoint2 tkill cp2`; `handed_eq.py tkill checkpoint2 tkill cp2`.
  Intervals: cluster bootstrap on seed (paired: the same resamples for every audit).

## The decision quantities (runbook 32)

- thrown games (table 5): **23/2048 = 1.12 % -> 8/2048 = 0.39 %, paired change -0.73 points [-1.12, -0.39]** (below 0)
- deterministic `missed_lethal` hits: **149 -> 21, -86 % [-92 %, -79 %]** (down >= 75 %)
- -> both conditions met. Sweep 32: not better, pooled 0.5081 >= 0.49 -> the served specs get `,tkill=10000`.

## Full output

```
gate: tkill and checkpoint2 share every game_id -> deck pair in both files (512 seeds): OK
end-turns with a verdict: tkill 15367, cp2 15586;  kill turns: 1098 / 1211;  auditee-games 2048 / 2048

=== 1. balance: auditee_first share of end-turns per deck ===
  deck                            tkill              cp2
  portal-af              0.507 (n= 964)   0.507 (n= 975)
  portal-evo             0.515 (n= 940)   0.515 (n= 946)
  abyss-midrange         0.499 (n= 881)   0.498 (n= 904)
  sword-rally            0.502 (n= 844)   0.505 (n= 856)
  haven-evo              0.521 (n=1027)   0.519 (n=1039)
  dragon-aggro           0.516 (n= 818)   0.510 (n= 831)
  abyss-aggro            0.499 (n= 834)   0.501 (n= 843)
  sword-loot             0.524 (n= 925)   0.526 (n= 928)
  forest-combo           0.526 (n= 934)   0.525 (n= 954)
  dragon-ramp            0.513 (n= 990)   0.514 (n= 997)
  rune-spell             0.503 (n= 952)   0.502 (n= 963)
  portal-cutthroat       0.522 (n=1015)   0.522 (n=1028)
  rune-test-subject      0.505 (n= 933)   0.503 (n= 946)
  haven-amulet           0.536 (n=1130)   0.538 (n=1157)
  rune-crystal           0.505 (n= 954)   0.506 (n= 977)
  haven-kukishiro        0.531 (n=1226)   0.533 (n=1242)

=== 2. win rate per deck (auditee games, both seats; 128 per deck), sorted by tkill ===
  deck                  tkill    cp2  change    95% CI (paired)  sweep9
  portal-af             0.680  0.695  -0.016   [-0.040, +0.000]  0.725
  portal-evo            0.680  0.672  +0.008   [+0.000, +0.027]  0.703
  haven-evo             0.680  0.672  +0.008   [+0.000, +0.027]  0.603
  sword-rally           0.648  0.633  +0.016   [+0.000, +0.040]  0.665
  abyss-midrange        0.633  0.617  +0.016   [-0.014, +0.047]  0.692
  portal-cutthroat      0.547  0.555  -0.008   [-0.026, +0.000]  0.394
  rune-test-subject     0.531  0.539  -0.008   [-0.027, +0.000]  0.353
  forest-combo          0.523  0.531  -0.008   [-0.035, +0.018]  0.523
  dragon-aggro          0.492  0.500  -0.008   [-0.036, +0.017]  0.603
  abyss-aggro           0.445  0.453  -0.008   [-0.025, +0.000]  0.598
  dragon-ramp           0.445  0.445  +0.000   [+0.000, +0.000]  0.501
  sword-loot            0.438  0.422  +0.016   [+0.000, +0.039]  0.582
  rune-spell            0.406  0.414  -0.008   [-0.025, +0.000]  0.395
  rune-crystal          0.352  0.352  +0.000   [+0.000, +0.000]  0.219
  haven-amulet          0.336  0.320  +0.016   [-0.015, +0.047]  0.306
  haven-kukishiro       0.164  0.180  -0.016   [-0.040, +0.000]  0.138
  spread max-min: tkill 0.516 (0.164-0.680), cp2 0.516 (0.180-0.695); change +0.000 [-0.021, +0.051]
  SD across decks: tkill 0.142, cp2 0.139;  corr(tkill, cp2) +0.997;  corr(tkill, sweep9) +0.836

=== 3. handed_lethal, standardised on deficit bucket x first player ===
  -- win axis: sweep 9 per-deck win rate (as on 2026-09-22)
    tkill  rate 0.0677  corr(expected,win) -0.507 [-0.589, -0.375]  corr(residual,win) -0.101 [-0.256, +0.051]  split expected 80% / residual 20%
           + early/late stratum:  corr(expected,win) -0.547 [-0.617, -0.436]  corr(residual,win) -0.030 [-0.180, +0.111]  split expected 94% / residual 6%
    cp2    rate 0.0738  corr(expected,win) -0.529 [-0.608, -0.398]  corr(residual,win) -0.075 [-0.234, +0.083]  split expected 85% / residual 15%
           + early/late stratum:  corr(expected,win) -0.567 [-0.636, -0.454]  corr(residual,win) -0.008 [-0.161, +0.135]  split expected 98% / residual 2%
  -- win axis: each audit own per-deck win rate (step 2; shares games with handed_lethal)
    tkill  rate 0.0677  corr(expected,win) -0.531 [-0.658, -0.359]  corr(residual,win) -0.281 [-0.497, -0.107]  split expected 60% / residual 40%
           + early/late stratum:  corr(expected,win) -0.529 [-0.651, -0.368]  corr(residual,win) -0.234 [-0.451, -0.064]  split expected 65% / residual 35%
    cp2    rate 0.0738  corr(expected,win) -0.554 [-0.676, -0.379]  corr(residual,win) -0.226 [-0.451, -0.049]  split expected 66% / residual 34%
           + early/late stratum:  corr(expected,win) -0.548 [-0.670, -0.389]  corr(residual,win) -0.185 [-0.410, -0.018]  split expected 70% / residual 30%
  pooled handed rate by deficit bucket (tkill | cp2):
    <= -5   0.045 (n=2312)  |  0.045 (n=2400)
    -4..-1  0.037 (n=3788)  |  0.039 (n=3802)
    0       0.009 (n=4790)  |  0.010 (n=4796)
    1..4    0.077 (n=2934)  |  0.084 (n=2964)
    >= 5    0.340 (n=1543)  |  0.367 (n=1624)

=== 4. missed_lethal per kill turn (top/bottom six fixed by sweep 9, as on 2026-09-22) ===
  tkill: own tercile cuts 4 / 205
  cp2: own tercile cuts 4 / 169
  common tercile cuts (both audits pooled): 4 / 193
                                            tkill                                cp2
  all                     0.0856 [0.0680, 0.1043]            0.1767 [0.1565, 0.1974]   change -0.0911 [-0.1075, -0.0752]
  top six                 0.0641 [0.0415, 0.0882]            0.1518 [0.1260, 0.1805]   change -0.0877 [-0.1095, -0.0663]
  bottom six              0.1458 [0.1079, 0.1851]            0.2526 [0.2114, 0.2915]   change -0.1068 [-0.1367, -0.0773]
  bottom-top           +0.0817 [+0.0372, +0.1268]         +0.1008 [+0.0502, +0.1492]   change -0.0191 [-0.0545, +0.0188]
  tercile 1      2/375 = 0.0053 [0.0000, 0.0134]    3/431 = 0.0070 [0.0000, 0.0160]
  tercile 2     19/347 = 0.0548 [0.0304, 0.0825]   59/386 = 0.1528 [0.1161, 0.1904]
  tercile 3     73/376 = 0.1941 [0.1513, 0.2390]  152/394 = 0.3858 [0.3392, 0.4332]
  counts: top six tkill 33/515 | cp2 85/560;  bottom six tkill 50/343 | cp2 99/392

=== 5. thrown games: auditee-games with a deterministic missed kill turn that the auditee lost ===
  tkill  pooled 8/2048 = 0.39%  [0.15, 0.68] %
  cp2    pooled 23/2048 = 1.12%  [0.73, 1.56] %
  change (paired by seed) -0.73%  [-1.12, -0.39] points
  deck                  tkill    cp2   (of 128 auditee-games each)
  rune-crystal              4      4
  abyss-midrange            1      3
  abyss-aggro               1      2
  dragon-aggro              1      3
  portal-cutthroat          1      1
  portal-evo                0      1
  sword-rally               0      2
  portal-af                 0      0
  sword-loot                0      2
  haven-evo                 0      1
  dragon-ramp               0      0
  forest-combo              0      2
  rune-spell                0      1
  rune-test-subject         0      0
  haven-amulet              0      1
  haven-kukishiro           0      0

kill turns (an auditee turn with at least one lethal verdict), same 512 deals x 2 files each:
                                                           cp1                       cp2                     tkill
  kill turns                                              1159                      1211                      1098
    deterministic (a luck-free kill)                      1072                      1062                       972
    chance-dependent only                                   87                       149                       126
    chance-dependent share                0.075 [0.059, 0.091]      0.123 [0.101, 0.145]      0.115 [0.091, 0.140]
  missed: all kill turns                  0.148 [0.128, 0.167]      0.177 [0.156, 0.197]      0.086 [0.068, 0.104]
  missed: deterministic turns             0.094 [0.078, 0.110]      0.096 [0.080, 0.113]      0.019 [0.010, 0.027]
  missed: chance-dependent turns          0.805 [0.716, 0.887]      0.752 [0.677, 0.817]      0.603 [0.519, 0.681]
  counts missed: cp1 det 101/1072, chance 70/87; cp2 det 102/1062, chance 112/149; tkill det 18/972, chance 76/126
decision-level missed_lethal hits (the action taken gave up a kill): cp1 deterministic 155, rng-dependent 137 (of 5184 lethal-verdict decisions); cp2 deterministic 149, rng-dependent 253 (of 5512 lethal-verdict decisions); tkill deterministic 21, rng-dependent 146 (of 4677 lethal-verdict decisions)
paired changes (later minus earlier, same resamples):
  cp2 - cp1: chance-dependent kill turns        +62 [+30, +94]
  cp2 - cp1: deterministic kill turns           -10 [-43, +23]
  cp2 - cp1: miss rate, deterministic turns     +0.002 [-0.019, +0.024]
  cp2 - cp1: miss rate, chance-dependent turns  -0.053 [-0.154, +0.046]
  cp2 - cp1: miss rate, all kill turns          +0.029 [+0.003, +0.057]
  cp2 - cp1: deterministic hits                 -6  relative -4% [-28, +31] %
  cp2 - cp1: rng-dependent hits                 +116  relative +85% [+31, +166] %
  tkill - cp2: chance-dependent kill turns        -23 [-38, -9]
  tkill - cp2: deterministic kill turns           -90 [-110, -69]
  tkill - cp2: miss rate, deterministic turns     -0.078 [-0.093, -0.062]
  tkill - cp2: miss rate, chance-dependent turns  -0.149 [-0.213, -0.088]
  tkill - cp2: miss rate, all kill turns          -0.091 [-0.108, -0.075]
  tkill - cp2: deterministic hits                 -128  relative -86% [-92, -79] %
  tkill - cp2: rng-dependent hits                 -107  relative -42% [-54, -30] %

missed_lethal hits (decision level; the auditee had a kill and the action gave it up), tkill vs cp2, same deals:
  deterministic (rng_dependent false)    tkill    21   cp2   149   change  -128 [-160, -99]   relative -86% [-92%, -79%]
  rng-dependent (true)                   tkill   146   cp2   253   change  -107 [-155, -64]   relative -42% [-54%, -30%]
  decisions with a 'lethal' verdict: tkill 4677, cp2 5512
kill turns with a deterministic kill, not converted: tkill 18/972 = 0.019   cp2 102/1062 = 0.096   relative -82% [-90%, -75%]

handed_lethal, cells: deficit bucket x first (as table 3)
  end-turns with a verdict: tkill 15367, cp2 15586; hits tkill 1041, cp2 1151
  raw rate        tkill 0.0677 [0.0659, 0.0697]   cp2 0.0738 [0.0716, 0.0762]
  tkill at cp2's deficit mix: 0.0690 [0.0671, 0.0711]
  paired change at equal deficit: -0.0048 [-0.0058, -0.0039]   relative -6.5% [-7.7, -5.3] %
  paired change, raw:             -0.0061 [-0.0073, -0.0050]
handed_lethal, cells: exact deficit [-10, +10] x first
  end-turns with a verdict: tkill 15367, cp2 15586; hits tkill 1041, cp2 1151
  raw rate        tkill 0.0677 [0.0659, 0.0697]   cp2 0.0738 [0.0716, 0.0762]
  tkill at cp2's deficit mix: 0.0694 [0.0674, 0.0715]
  paired change at equal deficit: -0.0044 [-0.0053, -0.0035]   relative -6.0% [-7.1, -4.8] %
  paired change, raw:             -0.0061 [-0.0073, -0.0050]
```
