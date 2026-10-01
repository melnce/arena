# checkpoint 2 lethal audit (h0 on 8e66bcf) vs checkpoint 1 (h0 on 1f8b068+), same 512 deals per file

Files: results/lethal/checkpoint2-f{a,b}.json vs checkpoint-f{a,b}.json (not published: ~40 MB each; on the owner's box).
Tables 1-5: tools/lethal_report.py checkpoint2 checkpoint (the method of the checkpoint runbook, steps 1-5; cp2 = checkpoint 2, cp1 = checkpoint 1).
Table 6 (definition inferred - the checkpoint 2 runbook's own was not available): tools/table6.py checkpoint2 checkpoint cp2 cp1.

```
gate: checkpoint2 and checkpoint share every game_id -> deck pair in both files (512 seeds): OK
end-turns with a verdict: cp2 15586, cp1 15050;  kill turns: 1211 / 1159;  auditee-games 2048 / 2048

=== 1. balance: auditee_first share of end-turns per deck ===
  deck                              cp2              cp1
  portal-af              0.507 (n= 975)   0.534 (n= 919)
  portal-evo             0.515 (n= 946)   0.499 (n= 920)
  abyss-midrange         0.498 (n= 904)   0.505 (n= 876)
  sword-rally            0.505 (n= 856)   0.514 (n= 846)
  haven-evo              0.519 (n=1039)   0.516 (n=1009)
  dragon-aggro           0.510 (n= 831)   0.496 (n= 820)
  abyss-aggro            0.501 (n= 843)   0.506 (n= 777)
  sword-loot             0.526 (n= 928)   0.523 (n= 896)
  forest-combo           0.525 (n= 954)   0.515 (n= 967)
  dragon-ramp            0.514 (n= 997)   0.529 (n= 982)
  rune-spell             0.502 (n= 963)   0.492 (n= 909)
  portal-cutthroat       0.522 (n=1028)   0.502 (n= 969)
  rune-test-subject      0.503 (n= 946)   0.500 (n= 932)
  haven-amulet           0.538 (n=1157)   0.546 (n=1127)
  rune-crystal           0.506 (n= 977)   0.523 (n= 910)
  haven-kukishiro        0.533 (n=1242)   0.537 (n=1191)

=== 2. win rate per deck (auditee games, both seats; 128 per deck), sorted by cp2 ===
  deck                    cp2    cp1  change    95% CI (paired)  sweep9
  portal-af             0.695  0.672  +0.023   [-0.082, +0.127]  0.725
  portal-evo            0.672  0.680  -0.008   [-0.107, +0.095]  0.703
  haven-evo             0.672  0.555  +0.117   [+0.016, +0.224]  0.603
  sword-rally           0.633  0.703  -0.070   [-0.157, +0.009]  0.665
  abyss-midrange        0.617  0.688  -0.070   [-0.176, +0.041]  0.692
  portal-cutthroat      0.555  0.477  +0.078   [-0.024, +0.182]  0.394
  rune-test-subject     0.539  0.508  +0.031   [-0.075, +0.139]  0.353
  forest-combo          0.531  0.414  +0.117   [+0.009, +0.222]  0.523
  dragon-aggro          0.500  0.562  -0.062   [-0.172, +0.045]  0.603
  abyss-aggro           0.453  0.594  -0.141   [-0.238, -0.046]  0.598
  dragon-ramp           0.445  0.500  -0.055   [-0.156, +0.045]  0.501
  sword-loot            0.422  0.539  -0.117   [-0.231, -0.007]  0.582
  rune-spell            0.414  0.391  +0.023   [-0.066, +0.114]  0.395
  rune-crystal          0.352  0.375  -0.023   [-0.121, +0.077]  0.219
  haven-amulet          0.320  0.234  +0.086   [+0.006, +0.167]  0.306
  haven-kukishiro       0.180  0.109  +0.070   [+0.000, +0.145]  0.138
  spread max-min: cp2 0.516 (0.180-0.695), cp1 0.594 (0.109-0.703); change -0.078 [-0.192, +0.022]
  SD across decks: cp2 0.139, cp1 0.161;  corr(cp2, cp1) +0.873;  corr(cp2, sweep9) +0.827

=== 3. handed_lethal, standardised on deficit bucket x first player ===
  -- win axis: sweep 9 per-deck win rate (as on 2026-09-22)
    cp2    rate 0.0738  corr(expected,win) -0.529 [-0.608, -0.398]  corr(residual,win) -0.075 [-0.234, +0.083]  split expected 85% / residual 15%
           + early/late stratum:  corr(expected,win) -0.567 [-0.636, -0.454]  corr(residual,win) -0.008 [-0.161, +0.135]  split expected 98% / residual 2%
    cp1    rate 0.0749  corr(expected,win) -0.696 [-0.763, -0.566]  corr(residual,win) -0.237 [-0.421, -0.001]  split expected 77% / residual 23%
           + early/late stratum:  corr(expected,win) -0.699 [-0.761, -0.587]  corr(residual,win) -0.149 [-0.338, +0.066]  split expected 84% / residual 16%
  -- win axis: each audit own per-deck win rate (step 2; shares games with handed_lethal)
    cp2    rate 0.0738  corr(expected,win) -0.554 [-0.676, -0.379]  corr(residual,win) -0.226 [-0.451, -0.049]  split expected 66% / residual 34%
           + early/late stratum:  corr(expected,win) -0.548 [-0.670, -0.389]  corr(residual,win) -0.185 [-0.410, -0.018]  split expected 70% / residual 30%
    cp1    rate 0.0749  corr(expected,win) -0.808 [-0.868, -0.669]  corr(residual,win) -0.112 [-0.387, +0.094]  split expected 89% / residual 11%
           + early/late stratum:  corr(expected,win) -0.801 [-0.865, -0.674]  corr(residual,win) -0.032 [-0.312, +0.161]  split expected 97% / residual 3%
  pooled handed rate by deficit bucket (cp2 | cp1):
    <= -5   0.045 (n=2400)  |  0.036 (n=2376)
    -4..-1  0.039 (n=3802)  |  0.037 (n=3768)
    0       0.010 (n=4796)  |  0.011 (n=4486)
    1..4    0.084 (n=2964)  |  0.074 (n=2783)
    >= 5    0.367 (n=1624)  |  0.395 (n=1637)

=== 4. missed_lethal per kill turn (top/bottom six fixed by sweep 9, as on 2026-09-22) ===
  cp2: own tercile cuts 4 / 169
  cp1: own tercile cuts 4 / 113
  common tercile cuts (both audits pooled): 4 / 132
                                              cp2                                cp1
  all                     0.1767 [0.1565, 0.1974]            0.1475 [0.1284, 0.1670]   change +0.0292 [+0.0026, +0.0569]
  top six                 0.1518 [0.1260, 0.1805]            0.1319 [0.1055, 0.1584]   change +0.0199 [-0.0139, +0.0565]
  bottom six              0.2526 [0.2114, 0.2915]            0.1795 [0.1399, 0.2193]   change +0.0731 [+0.0178, +0.1271]
  bottom-top           +0.1008 [+0.0502, +0.1492]         +0.0476 [-0.0004, +0.0942]   change +0.0532 [-0.0115, +0.1174]
  tercile 1      3/431 = 0.0070 [0.0000, 0.0160]    2/419 = 0.0048 [0.0000, 0.0123]
  tercile 2     53/363 = 0.1460 [0.1095, 0.1844]   33/367 = 0.0899 [0.0623, 0.1184]
  tercile 3    158/417 = 0.3789 [0.3349, 0.4259]  136/373 = 0.3646 [0.3167, 0.4136]
  counts: top six cp2 85/560 | cp1 74/561;  bottom six cp2 99/392 | cp1 56/312

=== 5. thrown games: auditee-games with a deterministic missed kill turn that the auditee lost ===
  cp2    pooled 23/2048 = 1.12%  [0.73, 1.56] %
  cp1    pooled 13/2048 = 0.63%  [0.34, 0.98] %
  change (paired by seed) +0.49%  [-0.05, +1.07] points
  deck                    cp2    cp1   (of 128 auditee-games each)
  rune-crystal              4      0
  abyss-midrange            3      0
  dragon-aggro              3      1
  abyss-aggro               2      1
  sword-loot                2      0
  sword-rally               2      2
  forest-combo              2      3
  portal-evo                1      0
  haven-amulet              1      0
  haven-evo                 1      2
  portal-cutthroat          1      2
  rune-spell                1      0
  portal-af                 0      0
  dragon-ramp               0      0
  rune-test-subject         0      2
  haven-kukishiro           0      0

=== 6. handed kills by the shape of the opponent's winning line ===
handed kills: cp2 1151, cp1 1127
  shape (share of handed kills)                         cp2                      cp1   change (paired by seed)
  with a follower attack           1102 = 0.957 [0.944, 0.969] 1076 = 0.955 [0.942, 0.966]   +0.003 [-0.014, +0.020]
  attacks, no play                  320 = 0.278 [0.252, 0.303]  313 = 0.278 [0.252, 0.302]   +0.000 [-0.035, +0.035]
  play + attack                     782 = 0.679 [0.651, 0.706]  763 = 0.677 [0.651, 0.704]   +0.002 [-0.035, +0.039]
  no attack                          49 = 0.043 [0.031, 0.056]   51 = 0.045 [0.034, 0.058]   -0.003 [-0.020, +0.014]
  with an evolve                    452 = 0.393 [0.365, 0.422]  472 = 0.419 [0.389, 0.448]   -0.026 [-0.063, +0.012]
  line length cp2: mean 5.12, median 5
  line length cp1: mean 4.59, median 4
```
