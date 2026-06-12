// 弧長フィット (細部修正 2026-06-12) — textPath の両端ちぎれ防止の検算。
import { describe, expect, it } from "vite-plus/test";

import { fitToArc } from "./signatureFit.ts";

describe("fitToArc", () => {
  it("収まるテキストはそのまま", () => {
    expect(fitToArc("fn greet()", 11, 60)).toEqual({ text: "fn greet()", fontSize: 11 });
  });

  it("溢れるテキストはフォントを縮小して収める", () => {
    const long = "fn lock_or_recover(mutex: &Mutex<T>) -> MutexGuard<T>";
    const fitted = fitToArc(long, 11, 90);
    expect(fitted.text).toBe(long); // 縮小で収まる範囲なら全文維持
    expect(fitted.fontSize).toBeLessThan(11);
    expect(fitted.fontSize).toBeGreaterThanOrEqual(7);
    // フィット後は弧長 (π×90) に収まる。
    expect(long.length * 0.65 * fitted.fontSize).toBeLessThanOrEqual(Math.PI * 90);
  });

  it("下限フォントでも溢れる超長文は省略記号で切り詰める", () => {
    const huge = "fn x(".concat("a: VeryLongTypeName, ".repeat(20), ") -> Out");
    const fitted = fitToArc(huge, 11, 40);
    expect(fitted.fontSize).toBe(7);
    expect(fitted.text.endsWith("…")).toBe(true);
    // +7 = 省略記号 1 文字分の描画幅マージン (… は平均近似より広いことがある)
    expect(fitted.text.length * 0.65 * 7).toBeLessThanOrEqual(Math.PI * 40 + 7);
  });

  it("旧 IR (arcRadius なし) はフィットせずそのまま", () => {
    const long = "x".repeat(200);
    expect(fitToArc(long, 11, undefined)).toEqual({ text: long, fontSize: 11 });
  });
});
