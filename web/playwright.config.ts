import { defineConfig } from "@playwright/test";

// E2E は本番経路 (rust-embed 同梱の SPA を magia バイナリが配信) を対象にする。
// webServer が一時 fixture を組み立てて magia serve を起動する (e2e/serve-fixture.sh)。
// SSE テストがファイルを書き換えるため、テストは直列 (workers: 1) で走らせる。
export default defineConfig({
  testDir: "./e2e",
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:4810",
  },
  webServer: {
    command: "bash e2e/serve-fixture.sh",
    url: "http://127.0.0.1:4810/state",
    reuseExistingServer: false,
    timeout: 180_000, // 初回は cargo build (+ bun build) を含む
  },
});
