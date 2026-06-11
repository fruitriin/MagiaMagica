import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vite-plus/test";

import { LAYERS, usePaletteStore } from "./palette.ts";

beforeEach(() => {
  setActivePinia(createPinia());
});

describe("usePaletteStore", () => {
  it("初期状態は全レイヤー可視・不透明・ミッドチルダ", () => {
    const palette = usePaletteStore();
    expect(palette.style).toBe("midchilda");
    for (const layer of LAYERS) {
      expect(palette.layers[layer]).toEqual({ visible: true, opacity: 1 });
    }
  });

  it("setVisibleSet は可視性だけ差し替えて透明度を保持する", () => {
    const palette = usePaletteStore();
    palette.setOpacity("effects", 0.3);
    palette.setVisibleSet(new Set(["effects"]));
    expect(palette.layers.control_flow.visible).toBe(false);
    expect(palette.layers.effects).toEqual({ visible: true, opacity: 0.3 });
  });

  it("showAll / hideAll は全レイヤーを一括切替する", () => {
    const palette = usePaletteStore();
    palette.hideAll();
    expect(LAYERS.every((l) => !palette.layers[l].visible)).toBe(true);
    palette.showAll();
    expect(LAYERS.every((l) => palette.layers[l].visible)).toBe(true);
  });
});
