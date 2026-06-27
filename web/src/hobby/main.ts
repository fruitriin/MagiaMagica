// WASM デモ版のエントリ (Phase 4.12 M2)。serve 版 (main.ts) と違い router は持たない —
// 単一画面の遊びなので Pinia (palette) だけ入れる。
import "@unocss/reset/tailwind.css";
import "virtual:uno.css";

import { createPinia } from "pinia";
import { createApp } from "vue";

import HobbyApp from "./HobbyApp.vue";

createApp(HobbyApp).use(createPinia()).mount("#app");
