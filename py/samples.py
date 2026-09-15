"""Load matchup export shards written by `arena.matchup(..., export=dir)`.

Raw little-endian arrays, no headers — `numpy.fromfile` reads them. Numpy is a
tool/test dependency, not of the `arena` extension.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def load(dir: str | Path) -> dict[str, Any]:
    """Return `{features, ids, labels, aux, meta, aux_columns}`.

    Shapes: `features (N, 545) float32`, `ids (N, 220) uint32`,
    `labels (N,)`, `aux (N, len(aux_columns))`.
    """
    import numpy as np

    d = Path(dir)
    meta = json.loads((d / "meta.json").read_text())
    n = int(meta["samples"])
    feat_len = int(meta["feature_len"])
    ids_len = int(meta["ids_len"])
    n_aux = len(meta["aux_columns"])
    features = np.fromfile(d / "features.f32le", dtype="<f4")
    ids = np.fromfile(d / "ids.u32le", dtype="<u4")
    labels = np.fromfile(d / "labels.f32le", dtype="<f4")
    aux = np.fromfile(d / "aux.f32le", dtype="<f4")
    if n == 0:
        features = features.reshape(0, feat_len)
        ids = ids.reshape(0, ids_len)
        aux = aux.reshape(0, n_aux)
    else:
        features = features.reshape(n, feat_len)
        ids = ids.reshape(n, ids_len)
        aux = aux.reshape(n, n_aux)
    return {
        "features": features,
        "ids": ids,
        "labels": labels,
        "aux": aux,
        "meta": meta,
        "aux_columns": meta["aux_columns"],
    }
