# sweep25b tools

`combine_finals.py` combines one candidate's finals from several sweeps **per arm** (main arms summed, reverse arms
summed; decisive games only) and applies the standing rule: `better` = combined main low > 0.50 **and** combined reverse
low > 0.50. Run from the repo root:

    python combine_finals.py results/sweep25:c01 results/sweep25b:c01

`../COMBINED.txt` is its output for c01 (e2, the confirm candidate fixed by sweep 25), c02 (e4) and c03 (e8).
