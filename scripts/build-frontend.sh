#!/usr/bin/env bash
# Vercel ビルド: Leptos フロントを wasm32 + Trunk で dist/ に静的ビルドする。
# (api/execute.rs は vercel-rust ランタイムが別途ビルドする)
set -euo pipefail

TRUNK_VERSION="v0.21.14"

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi
# rustup 直後は PATH に乗っていないことがある
export PATH="$HOME/.cargo/bin:$PATH"

rustup target add wasm32-unknown-unknown

# cargo install trunk は遅すぎるので prebuilt バイナリを使う
if ! command -v trunk >/dev/null 2>&1; then
  curl -sL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz -C "$HOME/.cargo/bin"
fi

trunk build --release
