# Dev harness

Plain static page over the WASM bindings. No framework, no design.

From the repo root (`--out-dir` is relative to the `wasm/` crate):

```
wasm-pack build wasm --target web --release --out-dir ../ui/dev/pkg
cd ui/dev && python3 -m http.server
```

Open http://localhost:8000/. `ui/dev/decks/` is a symlink to `oracle/decks/`.
