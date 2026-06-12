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
        return helper(sum);
    }
    sum
}

fn helper(value: i32) -> i32 {
    value * 2
}
EOF

# Spell Diff (Phase 4.3.7) の e2e 用に git リポジトリ化して初期内容を commit する。
# beforeEach が毎回 INITIAL に戻すため「HEAD = 初期内容」が常に成立し、
# テスト内のファイル変更がそのまま ?diff=HEAD の差分になる (live diff)。
git -C "$E2E_DIR" init -q
git -C "$E2E_DIR" -c user.name=e2e -c user.email=e2e@test add .
git -C "$E2E_DIR" -c user.name=e2e -c user.email=e2e@test commit -qm initial

cargo build -p magia-cli
exec target/debug/magia serve "$E2E_DIR/sample.rs" --port 4810
