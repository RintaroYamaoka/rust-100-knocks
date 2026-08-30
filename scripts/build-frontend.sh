#!/usr/bin/env bash
# Vercel ビルド: Leptos フロントを wasm32 + Trunk で dist/ に静的ビルドする。
# (api/execute.rs は vercel-rust ランタイムが別途ビルドする)
set -euo pipefail

TRUNK_VERSION="v0.21.14"

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi
# rustup 直後は PATH に乗っていないことがある。
# Vercel の build image は cargo が別の場所にあり $HOME/.cargo/bin 自体が無いことがあるので必ず作る。
BIN_DIR="$HOME/.cargo/bin"
mkdir -p "$BIN_DIR"
export PATH="$BIN_DIR:$PATH"

rustup target add wasm32-unknown-unknown

# cargo install trunk は遅すぎるので prebuilt バイナリを使う
if ! command -v trunk >/dev/null 2>&1; then
  # musl (静的リンク) 版を使う: gnu 版は GLIBC 2.35 を要求し、Vercel build image の glibc より新しくて起動できない
  curl -sL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz -C "$BIN_DIR"
fi

# 収録済み言語のマニフェストを実ファイルから生成する (フロントはこれ 1 本で
# セレクタの中身を決める)。node は Vercel の build image に入っている
node scripts/gen-manifest.mjs

trunk build --release
