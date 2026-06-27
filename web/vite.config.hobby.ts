import vue from "@vitejs/plugin-vue";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite-plus";

// WASM デモ版 (Phase 4.12) の静的ビルド設定。serve 同梱 SPA (index.html, dist/) とは
// 別ターゲット — エントリは hobby.html、出力は dist-hobby/、deploy は Vercel / CF Pages 等の
// 静的ホスティング。base は相対 ("./") にして任意のパス配下でもアセットが解決できるように。
// serve のビルド (vite.config.ts) はこの設定を読まないので、serve バイナリに wasm は載らない。
export default defineConfig({
  base: "./",
  plugins: [vue(), UnoCSS()],
  build: {
    outDir: "dist-hobby",
    emptyOutDir: true,
    rollupOptions: {
      input: { hobby: "hobby.html" },
    },
  },
});
