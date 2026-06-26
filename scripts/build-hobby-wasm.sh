#!/usr/bin/env bash
# magia-hobby (Phase 4.12) の WASM を生成し、web の hobby デモから読める場所へ置く。
#
# 生成物 (web/src/hobby/wasm/):
#   magia_hobby.js        — wasm-bindgen が生成する ESM グルー (list / spell をエクスポート)
#   magia_hobby_bg.wasm   — wasm 本体
#   magia_hobby.d.ts      — 型定義
#
# 前提: rustup target add wasm32-unknown-unknown / cargo install wasm-bindgen-cli。
# 無ければ手順つきで停止する。M2 (hobby Vite ターゲット) のビルド配線から呼ぶ。
set -euo pipefail

cd "$(dirname "$0")/.."

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
  echo "wasm32 ターゲットがありません: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen がありません: cargo install wasm-bindgen-cli" >&2
  exit 1
fi

# wasm-bindgen CLI は Cargo.lock のライブラリ版と完全一致が要る (ずれると
# schema version mismatch で生成が失敗する)。lock から要求版を読んで照合する。
REQUIRED=$(awk '/^name = "wasm-bindgen"$/{f=1} f&&/^version = /{gsub(/[",]/,"",$3); print $3; exit}' Cargo.lock)
ACTUAL=$(wasm-bindgen --version | awk '{print $2}')
if [ -n "${REQUIRED}" ] && [ "${ACTUAL}" != "${REQUIRED}" ]; then
  echo "wasm-bindgen CLI バージョン不一致: want=${REQUIRED} got=${ACTUAL}" >&2
  echo "cargo install wasm-bindgen-cli --version ${REQUIRED} --force" >&2
  exit 1
fi

OUT_DIR="web/src/hobby/wasm"

echo "[1/2] cargo build --target wasm32-unknown-unknown --release"
cargo build -p magia-hobby --target wasm32-unknown-unknown --release

echo "[2/2] wasm-bindgen → ${OUT_DIR}"
mkdir -p "${OUT_DIR}"
wasm-bindgen target/wasm32-unknown-unknown/release/magia_hobby.wasm \
  --out-dir "${OUT_DIR}" \
  --target web

# wasm-opt があればサイズ最適化 (任意。無くても成立する)。
if command -v wasm-opt >/dev/null 2>&1; then
  echo "[opt] wasm-opt -Oz"
  wasm-opt -Oz "${OUT_DIR}/magia_hobby_bg.wasm" -o "${OUT_DIR}/magia_hobby_bg.wasm"
fi

echo "done: ${OUT_DIR}"
