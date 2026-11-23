#!/usr/bin/env bash
# Cloudflare Pages build command. Installs Rust if the build image lacks it, then
# builds the pre-rendered web release into target/site (see scripts/build-web.sh).
#
# Cloudflare Pages settings:  Build command: bash deploy.sh
#                             Build output directory: target/site
set -euo pipefail

if [ -d "/opt/buildhome" ]; then
    export CARGO_HOME="/opt/buildhome/.cargo"
    export RUSTUP_HOME="/opt/buildhome/.rustup"
fi
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export PATH="$CARGO_HOME/bin:$PATH"

SPODEIAN_GIT_TOKEN="${SPODEIAN_CORE_TOKEN:-${PRIVATE_READ_ACCESS:-${GH_PAT:-}}}"
if [ -n "$SPODEIAN_GIT_TOKEN" ]; then
    export CARGO_NET_GIT_FETCH_WITH_CLI=true
    export GIT_CONFIG_COUNT=1
    export GIT_CONFIG_KEY_0="url.https://x-access-token:${SPODEIAN_GIT_TOKEN}@github.com/.insteadOf"
    export GIT_CONFIG_VALUE_0="https://github.com/"
fi

if ! command -v rustup >/dev/null 2>&1; then
    echo "==> Installing Rust (toolchain from rust-toolchain.toml)"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
fi

# rust-toolchain.toml selects the toolchain and installs the wasm32 target on first use.
rustc --version

bash scripts/build-web.sh

if [ "${CLOUDFLARE_WORKER_DEPLOY:-false}" = "true" ]; then
    echo "==> Deploying with Wrangler"
    npx --yes wrangler@4 deploy
fi
