// URL クエリ ↔ store 群の双方向同期。クエリ形式は Phase 2.x inline 版と完全互換:
//   ?pin=<qualified> ?style=belka ?layers=a,b (全表示時は省略) ?op=layer:0.5,... (1.0 は省略)
// URL を唯一の状態源とする一方向ループ: UI は store を変え、store → URL (replace)、
// URL → store (watch)。関数切替 (?fn=) だけは FunctionToc が push して履歴に積む。

import { watch } from "vue";
import { type LocationQuery, useRoute, useRouter } from "vue-router";

import { useFocusStore } from "../stores/focus.ts";
import { LAYERS, type LayerName, usePaletteStore } from "../stores/palette.ts";

function asString(value: LocationQuery[string] | undefined): string | null {
  return typeof value === "string" ? value : null;
}

export function useQuerySync() {
  const route = useRoute();
  const router = useRouter();
  const focus = useFocusStore();
  const palette = usePaletteStore();

  function applyQueryToStores(query: LocationQuery) {
    palette.setStyle(asString(query["style"]) === "belka" ? "belka" : "midchilda");

    const layers = asString(query["layers"]);
    const visible =
      layers === null
        ? new Set<LayerName>(LAYERS)
        : new Set<LayerName>(
            layers
              .split(",")
              .filter((l): l is LayerName => (LAYERS as readonly string[]).includes(l)),
          );
    palette.setVisibleSet(visible);

    for (const layer of LAYERS) palette.setOpacity(layer, 1);
    for (const pair of (asString(query["op"]) ?? "").split(",").filter(Boolean)) {
      const sep = pair.indexOf(":");
      if (sep === -1) continue;
      const layer = pair.slice(0, sep);
      const value = Number.parseFloat(pair.slice(sep + 1));
      if ((LAYERS as readonly string[]).includes(layer) && !Number.isNaN(value)) {
        palette.setOpacity(layer as LayerName, value);
      }
    }

    // pin は currentFn と異なるときだけロードする。初期化パス (下の setInitialFn 直後)
    // ではこのガードにより selectFunction が走らず、初回ロードは SSE 接続直後
    // イベントの refresh に一本化される — このガードはその前提を担う (壊すと二重フェッチ)。
    const fn = asString(query["pin"]);
    if (fn !== null && fn !== focus.currentFn) {
      void focus.selectFunction(fn);
    }

    // diff 基準 rev (Phase 4.3.7)。pin のような変更ガードは置かず
    // setDiffRev の同値スキップに任せる (初期化パスでは null === null で素通り)。
    void focus.setDiffRev(asString(query["diff"]));

    // 監視ファイルの希望 (Phase 4.4.5)。切替の発火は下の watch (file 確定後)。
    focus.setRequestedFile(asString(query["file"]));

    // 周辺チップの引数表示 (細部修正 2026-06-12)。`name` / `type` のカンマ区切り。
    const args = (asString(query["args"]) ?? "").split(",");
    palette.argNames = args.includes("name");
    palette.argTypes = args.includes("type");

    // 視野 (Phase 4.5)。workspace 以外の値は focus に倒す。
    const scope = asString(query["scope"]) === "workspace" ? "workspace" : "focus";
    if (scope !== focus.scope) {
      void focus.setScope(scope);
    }
  }

  function buildQuery(): Record<string, string> {
    const params: Record<string, string> = {};
    if (focus.currentFn !== null) params["pin"] = focus.currentFn;
    if (focus.requestedFile !== null) params["file"] = focus.requestedFile;
    if (focus.diffRev !== null) params["diff"] = focus.diffRev;
    if (focus.scope === "workspace") params["scope"] = "workspace";
    const argParts = [palette.argNames ? "name" : null, palette.argTypes ? "type" : null].filter(
      (p): p is string => p !== null,
    );
    if (argParts.length > 0) params["args"] = argParts.join(",");
    if (palette.style === "belka") params["style"] = "belka";
    const shown = LAYERS.filter((l) => palette.layers[l].visible);
    if (shown.length < LAYERS.length) params["layers"] = shown.join(",");
    const ops = LAYERS.filter((l) => Math.abs(palette.layers[l].opacity - 1) > 1e-9)
      .map((l) => `${l}:${palette.layers[l].opacity}`)
      .join(",");
    if (ops !== "") params["op"] = ops;
    return params;
  }

  // 初期 URL → store (?fn= は希望値として先置きし、実ロードは SSE 初回イベントに任せる)
  focus.setInitialFn(asString(route.query["pin"]));
  applyQueryToStores(route.query);

  // store → URL。状態微調整は履歴を汚さない (replace)。同値ならスキップしてループを断つ。
  // 比較はキー順に依存させない (URL が ?style=..&fn=.. の順で入力されても同値と扱う)。
  const canonical = (params: Record<string, string>) =>
    JSON.stringify(Object.entries(params).sort(([a], [b]) => a.localeCompare(b)));
  // 監視ファイルの切替 (Phase 4.4.5): サーバの file が確定してから希望と比較する
  // (SSE 初回前は file = null で判定できない)。switchFile は同値なら何もしないため
  // 「POST → SSE → file 更新 → ここが再発火 → 同値」でループが止まる。
  // URL 由来の復元 POST は**1ページロードにつき最初の不一致の1回だけ** (細部修正
  // 2026-06-12): 以後の不一致は「他のタブ/操作がファイルを切り替えた」なので、
  // 主張し返さずサーバに追従する (複数タブの ?file= 綱引きで無限 POST になる穴)。
  let restoreAttempted = false;
  watch([() => focus.file, () => focus.requestedFile], ([current, requested]) => {
    if (current === null || requested === null || requested === current) return;
    if (restoreAttempted) {
      focus.setRequestedFile(current); // サーバ追従 (URL の ?file= も書き換わる)
      return;
    }
    restoreAttempted = true;
    void focus.switchFile(requested);
  });

  watch(
    [
      () => focus.currentFn,
      () => focus.requestedFile,
      () => focus.diffRev,
      () => focus.scope,
      () => palette.style,
      () => palette.layers,
      () => palette.argNames,
      () => palette.argTypes,
    ],
    () => {
      const next = buildQuery();
      const current = Object.fromEntries(
        Object.entries(route.query).flatMap(([k, v]) => (typeof v === "string" ? [[k, v]] : [])),
      );
      if (canonical(next) !== canonical(current)) {
        void router.replace({ query: next });
      }
    },
    { deep: true },
  );

  // URL → store (戻る/進む・FunctionToc の push がここを通る)
  watch(
    () => route.query,
    (query) => applyQueryToStores(query),
  );
}
