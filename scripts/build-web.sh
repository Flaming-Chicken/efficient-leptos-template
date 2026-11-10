#!/usr/bin/env bash
# Builds the web release: a pre-rendered static site with a hydration bundle.
#
#   scripts/build-web.sh            # output in target/site
#
# Every step fails the build on error. Tool versions are pinned:
#   - cargo-leptos (below), which itself pins Sass and wasm-opt (Binaryen)
#   - wasm-bindgen-cli, always the version in Cargo.lock
set -euo pipefail

CARGO_LEPTOS_VERSION="0.3.4"
SITE_DIR="target/site"

cd "$(dirname "$0")/.."

installed_version() { "$@" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true; }

if [ "$(installed_version cargo leptos --version)" != "$CARGO_LEPTOS_VERSION" ]; then
    echo "==> Installing cargo-leptos $CARGO_LEPTOS_VERSION"
    cargo install cargo-leptos --version "$CARGO_LEPTOS_VERSION" --locked
fi

WASM_BINDGEN_VERSION="$(cargo pkgid --quiet wasm-bindgen | sed 's/.*[@#]//')"
if [ "$(installed_version wasm-bindgen --version)" != "$WASM_BINDGEN_VERSION" ]; then
    echo "==> Installing wasm-bindgen-cli $WASM_BINDGEN_VERSION (from Cargo.lock)"
    cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --locked
fi

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

echo "==> Building hydration bundle, stylesheet and the ssg binary"
cargo leptos build --release

echo "==> Pre-rendering every route to HTML"
./target/release/ssg "$SITE_DIR"

# Fail if any page is missing or empty: a broken site must not deploy.
for page in index.html about.html 404.html; do
    if [ ! -s "$SITE_DIR/$page" ]; then
        echo "ERROR: $SITE_DIR/$page was not generated" >&2
        exit 1
    fi
done

echo "==> Web release ready in $SITE_DIR"
