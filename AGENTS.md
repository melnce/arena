# Arena — Cloud Agent Notes

## Python virtualenv

The snapshot provides a pre-built venv at **`$HOME/.venvs/arena`** (outside the repo).
It has `maturin`, `pytest`, and the `arena` extension built at the snapshot commit.

Activate it in your shell:

```bash
. "$HOME/.venvs/arena/bin/activate"
```

## After changing Rust code

Re-build the Python extension before running tests:

```bash
maturin develop --release -m py/Cargo.toml
```

## Running tests

```bash
pytest py/tests -q
```

Absolute fallback (no activation needed):

```bash
"$HOME/.venvs/arena/bin/python" -m pytest py/tests -q
```

## Skipped tests are not a pass

If `arena` is missing, several test files skip via `pytest.importorskip("arena")`.
Always confirm the extension loads first:

```bash
python -c "import arena"
```

A pytest run where `arena` tests are skipped is **not** a successful test run.
