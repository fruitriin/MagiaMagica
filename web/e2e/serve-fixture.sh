#!/usr/bin/env bash
# Playwright E2E 用の magia serve 起動。専用ディレクトリに fixture を書き出して監視させる
# (他テストのファイル変更イベントが notify 監視に混入しないよう隔離 — serve_integration と同じ方針)。
set -euo pipefail
cd "$(dirname "$0")/../.."

E2E_DIR=/tmp/magia-web-e2e
rm -rf "$E2E_DIR"
mkdir -p "$E2E_DIR"
cat > "$E2E_DIR/sample.rs" << 'EOF'
fn greet(name: &str) -> String {
    format!("Hello, {name}")
}

fn compute(a: i32, b: i32) -> i32 {
    let sum = a + b;
    if sum > 10 {
        return sum * 2;
    }
    sum
}
EOF

cargo build -p magia-cli
exec target/debug/magia serve "$E2E_DIR/sample.rs" --port 4810
