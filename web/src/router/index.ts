import { createRouter, createWebHistory } from "vue-router";

import HomeView from "../views/HomeView.vue";

// ルーティング方針 (オーナー判定 2026-06-11): ファイルベースルーティング
// (Nuxt 風自動ルーティング) が基本的な好みだが、本プロジェクトのビューは
// `?pin / ?theme / ?diff / ?style / ?scope` のようにクエリ軸で状態が積層する
// 「上位から一意に定まらない」複雑さを持つため、今回は明示的・プログラマブルな
// クエリベースで進める。フェーズが進んでも複雑性が十分に収まるなら、
// ファイルベースルーティング化のリファクタリングも検討する。
//
// ルートは1本。`?fn=` クエリでの関数選択 (Phase 4.0 互換) は M2 で配線し、
// `?pin=` へのリネームは Phase 4.1 で行う。
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/", component: HomeView }],
});
