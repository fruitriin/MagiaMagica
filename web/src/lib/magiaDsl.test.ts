import { describe, expect, it } from "vite-plus/test";

import { exportDsl, parseDsl } from "./magiaDsl.ts";
import type { LayerName } from "../stores/palette.ts";

describe("exportDsl", () => {
  it("可視レイヤーを show 行に書き出す", () => {
    expect(exportDsl(new Set<LayerName>(["control_flow", "effects"]))).toBe(
      "show: control_flow + effects",
    );
  });

  it("全レイヤー非表示はコメント付き hide 行になる", () => {
    expect(exportDsl(new Set())).toBe(
      "# 全レイヤー非表示\nhide: control_flow + effects + type_info",
    );
  });
});

describe("parseDsl", () => {
  it("show 行は列挙したレイヤーだけを可視にする", () => {
    const result = parseDsl("show: control_flow + type_info");
    expect(result).toEqual({
      ok: true,
      visible: new Set(["control_flow", "type_info"]),
      note: "",
    });
  });

  it("hide 行は既定 (全表示) から差し引く", () => {
    const result = parseDsl("hide: effects");
    expect(result).toEqual({
      ok: true,
      visible: new Set(["control_flow", "type_info"]),
      note: "",
    });
  });

  it("空行とコメント行は無視する", () => {
    const result = parseDsl("# コメント\n\nshow: effects\n");
    expect(result).toEqual({ ok: true, visible: new Set(["effects"]), note: "" });
  });

  it("show のカテゴリ指定は CLI 案内の注記つきで通る", () => {
    const result = parseDsl("show: effects[io]");
    expect(result).toEqual({
      ok: true,
      visible: new Set(["effects"]),
      note: "effects[カテゴリ] の絞り込みは magia render --filter で適用されます",
    });
  });

  it("hide のカテゴリ指定は行番号つきエラー", () => {
    expect(parseDsl("hide: effects[io]")).toEqual({
      ok: false,
      error: "1行目: hide にカテゴリ指定 [...] はできません",
    });
  });

  it("未知のレイヤー名は行番号つきエラー", () => {
    expect(parseDsl("show: effects\nshow: nonsense")).toEqual({
      ok: false,
      error: "2行目: 未知のレイヤー名 `nonsense`",
    });
  });

  it("show:/hide: 以外のディレクティブはエラー", () => {
    expect(parseDsl("layers: all")).toEqual({
      ok: false,
      error: "1行目: show: / hide: のみ使用できます",
    });
  });

  it("show と hide の併用は show 集合から hide を差し引く", () => {
    const result = parseDsl("show: control_flow + effects\nhide: effects");
    expect(result).toEqual({ ok: true, visible: new Set(["control_flow"]), note: "" });
  });
});
