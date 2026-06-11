#!/usr/bin/env bash
# Spell Diff レポート生成 (Phase 3.3, spec v0.3 §9.1)
#
# 使い方: scripts/spell-diff-report.sh <BASE_REV> [OUT_DIR]
#   BASE_REV: 比較基準のリビジョン (例: origin/main)
#   OUT_DIR : 出力先 (既定: spell-diff-out)。comment.md と svg/ が生成される
#
# CI とローカルで同一の動作をする (デバッグ可能性の要件)。
# magia バイナリは環境変数 MAGIA で差し替えられる (CI ではビルド済みパスを渡す)。
set -euo pipefail

BASE_REV="${1:?BASE_REV を指定してください (例: origin/main)}"
OUT_DIR="${2:-spell-diff-out}"
MAGIA="${MAGIA:-cargo run -q -p magia-cli --}"

mkdir -p "$OUT_DIR/svg"

$MAGIA changed --git "$BASE_REV" --json > "$OUT_DIR/changed.json"

{
  echo "<!-- spell-diff -->"
  echo "## 🔮 Spell Diff"
  echo ""
} > "$OUT_DIR/comment.md"

count="$(jq length "$OUT_DIR/changed.json")"
if [ "$count" -eq 0 ]; then
  echo "変更された関数の構造変化はありません。" >> "$OUT_DIR/comment.md"
else
  echo "変更された関数: **${count} 件**" >> "$OUT_DIR/comment.md"
  echo "" >> "$OUT_DIR/comment.md"

  # 変更された関数ごとにテキストレポートと強調 SVG を生成する。
  jq -r '.[] | [.file, .function, .status, (.new_unsafe|tostring)] | @tsv' "$OUT_DIR/changed.json" |
  while IFS=$'\t' read -r file function status new_unsafe; do
    unsafe_mark=""
    if [ "$new_unsafe" = "true" ]; then
      unsafe_mark=" ⚠ **unsafe 追加**"
    fi
    case "$status" in
      変更)
        {
          echo "<details><summary><code>${file}</code> の <code>${function}</code> — ${status}${unsafe_mark}</summary>"
          echo ""
          echo '```'
          $MAGIA diff "$file" --git "$BASE_REV" --fn "$function"
          echo '```'
          echo "</details>"
          echo ""
        } >> "$OUT_DIR/comment.md"
        # SVG はパス区切りを __ に潰したファイル名で artifact に積む。
        svg_name="$(printf '%s' "$file" | tr '/' '_')__${function}.svg"
        $MAGIA diff "$file" --git "$BASE_REV" --fn "$function" --svg -o "$OUT_DIR/svg/$svg_name"
        ;;
      追加|削除)
        echo "- \`${file}\` の \`${function}\` — ${status}${unsafe_mark}" >> "$OUT_DIR/comment.md"
        ;;
    esac
  done

  {
    echo ""
    echo "_強調 SVG (金=追加 / シアン=変更 / 灰破線=削除) はワークフローの artifact \`spell-diff-svg\` から取得できます。_"
  } >> "$OUT_DIR/comment.md"
fi

echo "生成完了: $OUT_DIR/comment.md ($(jq length "$OUT_DIR/changed.json") 件)" >&2
