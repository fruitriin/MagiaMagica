import { createRouter, createWebHistory } from "vue-router";

import HomeView from "../views/HomeView.vue";

// ルートは1本。`?fn=` クエリでの関数選択 (Phase 4.0 互換) は M2 で配線し、
// `?pin=` へのリネームは Phase 4.1 で行う。
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/", component: HomeView }],
});
