// シグネチャ円弧テキストのフィット計算 (細部修正 2026-06-12)。
// textPath はパス (上半円、弧長 = π × 半径) より長いテキストを**両端で切り捨てる**
// (startOffset 50% の中央寄せのため前後ともちぎれる — オーナー報告のバグ)。
// 対策: まずフォントを縮小し、可読下限でも収まらなければ省略記号で切り詰める。

/** 平均文字幅 / fontSize の近似 (既定 sans-serif の実測感)。広字 (M, W) が
 *  連続する最悪ケースでも欠けないよう、中立値 (≈0.62) よりやや保守的に取る
 *  (レビュー指摘 — ずれても「端が少し欠ける」で旧バグの両端切断には戻らない)。 */
const CHAR_WIDTH_RATIO = 0.65;
/** これ以上縮めると読めない下限。以降は文字数側を削る。 */
const MIN_FONT_SIZE = 7;

export type FittedText = {
  text: string;
  fontSize: number;
};

/**
 * 弧長に収まるフォントサイズと表示テキストを決める。
 * `arcRadius` が無い (旧 IR) ときはフィットせずそのまま返す。
 */
export function fitToArc(
  text: string,
  fontSize: number,
  arcRadius: number | undefined,
): FittedText {
  if (arcRadius === undefined || text.length === 0) {
    return { text, fontSize };
  }
  const arcLength = Math.PI * arcRadius;
  const widthAt = (size: number) => text.length * CHAR_WIDTH_RATIO * size;
  let fitted = fontSize;
  if (widthAt(fitted) > arcLength) {
    // SSR 出力を安定させるため 0.1 刻みに丸める (浮動小数の桁ノイズ防止)。
    fitted = Math.max(
      MIN_FONT_SIZE,
      Math.floor((arcLength / (text.length * CHAR_WIDTH_RATIO)) * 10) / 10,
    );
  }
  if (widthAt(fitted) <= arcLength) {
    return { text, fontSize: fitted };
  }
  // 下限フォントでも溢れる: 省略記号込みで収まる文字数に切り詰める。
  const maxChars = Math.floor(arcLength / (CHAR_WIDTH_RATIO * fitted));
  return { text: `${text.slice(0, Math.max(1, maxChars - 1))}…`, fontSize: fitted };
}
