#!/usr/bin/env bash

set -euo pipefail

mode="${1:-native}"

if [[ "$mode" != "native" && "$mode" != "wasm" ]]; then
    echo "usage: ./build.sh [native|wasm]" >&2
    exit 2
fi

cargo test

cargo fmt --all -- --check

cargo clippy --all -- -D warnings

if [[ "$mode" == "native" ]]; then
    cargo build
    exit 0
fi

if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
    echo "missing Rust WASM target; install it with:" >&2
    echo "  rustup target add wasm32-unknown-unknown" >&2
    exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
    echo "missing wasm-bindgen CLI; install the Cargo.lock version with:" >&2
    echo "  cargo install wasm-bindgen-cli --version 0.2.126 --locked" >&2
    exit 1
fi

wasm_bindgen_version="$(wasm-bindgen --version | awk '{print $2}')"
if [[ "$wasm_bindgen_version" != "0.2.126" ]]; then
    echo "wasm-bindgen CLI $wasm_bindgen_version does not match Cargo.lock (0.2.126)." >&2
    echo "Install the matching version with:" >&2
    echo "  cargo install wasm-bindgen-cli --version 0.2.126 --locked --force" >&2
    exit 1
fi

cargo build --release --target wasm32-unknown-unknown
mkdir -p web/pkg
wasm-bindgen \
    --out-name keep_trucking \
    --out-dir web/pkg \
    --target web \
    target/wasm32-unknown-unknown/release/keep_trucking.wasm

echo "WASM build written to web/. Serve it over HTTP, for example:"
echo "  python -m http.server 8080 --directory web"
