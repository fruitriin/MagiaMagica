#!/usr/bin/env bash
# 開発用の並走起動 (Phase 4.0.5 M5): magia serve (API, 4747) + vite dev (HMR, 5173)。
# 使い方: scripts/dev-web.sh [監視対象の .rs ファイル] (省略時は自己ホスティング)
set -euo pipefail
cd "$(dirname "$0")/.."

FILE="${1:-crates/magia-core/src/render/midchilda.rs}"

cargo build -p magia-cli
target/debug/magia serve "$FILE" --port 4747 &
MAGIA_PID=$!
trap 'kill "$MAGIA_PID" 2>/dev/null' EXIT

cd web && bun run dev
