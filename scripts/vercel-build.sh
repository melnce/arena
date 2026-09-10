#!/usr/bin/env bash
set -euo pipefail

echo "== vercel-build =="
echo "date: $(date -u +%FT%TZ)"
echo "uname: $(uname -srm)"
echo "node: $(command -v node || echo missing) $(node -v 2>/dev/null || true)"
echo "npm: $(command -v npm || echo missing) $(npm -v 2>/dev/null || true)"
echo "cargo: $(command -v cargo || echo missing)"
echo "rustc: $(command -v rustc || echo missing)"
echo "wasm-pack: $(command -v wasm-pack || echo missing)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "installing rustup (minimal, stable)..."
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
fi

# Vercel / a home install: ~/.cargo. Some images (this one) use CARGO_HOME=/usr/local/cargo.
if [ -f "${CARGO_HOME:-}/env" ]; then
  # shellcheck disable=SC1091
  . "$CARGO_HOME/env"
elif [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
elif [ -f /usr/local/cargo/env ]; then
  # shellcheck disable=SC1091
  . /usr/local/cargo/env
fi
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$HOME/.cargo/bin:/usr/local/cargo/bin:${PATH}"

rustup target add wasm32-unknown-unknown

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "installing wasm-pack (prebuilt installer, not cargo install)..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$HOME/.cargo/bin:/usr/local/cargo/bin:${PATH}"
fi

echo "cargo: $(cargo --version)"
echo "rustc: $(rustc --version)"
echo "wasm-pack: $(wasm-pack --version)"

wasm-pack build wasm --target web --release --out-dir ../ui/pkg
cd ui && npm run build
