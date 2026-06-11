import vue from "@vitejs/plugin-vue";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite-plus";

// magia serve (Rust, ポート 4747) が IR/レイアウト計算と SSE を担い、
// vite+ dev (5173) はフロントの HMR だけを担う二段構成 (Phase 4.0.5 計画)。
// ブラウザは 5173 だけを見る。API は proxy で 4747 へ転送する。
const MAGIA_SERVE = "http://127.0.0.1:4747";

export default defineConfig({
  plugins: [vue(), UnoCSS()],
  server: {
    // 既定 5173。他プロジェクトと衝突する環境では PORT で差し替えられるようにする
    // (preview ハーネスの autoPort も PORT 経由でポートを割り当てる)
    port: Number(process.env["PORT"]) || 5173,
    proxy: {
      "/state": MAGIA_SERVE,
      "/spell": MAGIA_SERVE,
      "/events": MAGIA_SERVE,
    },
  },
  test: {
    // svgToSchema が DOMParser を使うため DOM 実装が要る (Phase 4.0.7)
    environment: "happy-dom",
    // e2e/ は Playwright の領分 (test 設定を書くと vitest の既定 exclude が外れて拾われる)
    exclude: ["**/node_modules/**", "e2e/**"],
  },
  fmt: {},
  lint: {
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: {
      "vite-plus/prefer-vite-plus-imports": "error",
      // プロジェクト規約: 型は `type` で宣言する (interface は宣言マージが必要な公開 API のみ、
      // 例外時は `// ts: interface required because <理由>` コメント必須)
      "typescript/consistent-type-definitions": ["error", "type"],
    },
    options: { typeAware: true, typeCheck: true },
  },
});
