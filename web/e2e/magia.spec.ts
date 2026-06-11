// Phase 2.x 機能 + Phase 4.0 ペアビューの E2E (M0 golden と同じシナリオを自動化)。
// 対象は本番経路: rust-embed 同梱の Vue SPA を magia バイナリが配信する。
import { expect, test } from "@playwright/test";
import { writeFileSync } from "node:fs";

const FIXTURE = "/tmp/magia-web-e2e/sample.rs";

const INITIAL = `fn greet(name: &str) -> String {
    format!("Hello, {name}")
}

fn compute(a: i32, b: i32) -> i32 {
    let sum = a + b;
    if sum > 10 {
        return sum * 2;
    }
    sum
}
`;

// 各テストを既知のファイル内容から始める (SSE テストの変更を持ち越さない)。
test.beforeEach(() => {
  writeFileSync(FIXTURE, INITIAL);
});

test("既定表示: 魔法陣・ソース・関数目次が出る (先頭関数へ fallback + URL 書き戻し)", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("svg").first()).toBeVisible();
  await expect(page.locator("header span").first()).toHaveText("greet");
  await expect(page).toHaveURL(/\?fn=greet/);
  await expect(page.locator("nav button")).toHaveText(["greet", "compute"]);
  await expect(page.locator("main section pre").first()).toContainText("fn greet");
});

test("関数目次クリックで ?fn= が切り替わり、戻るで復帰する", async ({ page }) => {
  await page.goto("/?fn=greet");
  await page.locator("nav button", { hasText: "compute" }).click();
  await expect(page).toHaveURL(/\?fn=compute/);
  await expect(page.locator("main section pre").first()).toContainText("fn compute");
  await page.goBack();
  await expect(page).toHaveURL(/\?fn=greet/);
  await expect(page.locator("header span").first()).toHaveText("greet");
});

test("レイヤー toggle: 制御フローの要素が非表示になり URL に残る", async ({ page }) => {
  await page.goto("/?fn=compute");
  // 「存在しないから hidden」の偽陽性を防ぐ: 先に表示されていることを確かめる。
  const ring = page.locator("svg circle.main-ring").first();
  await expect(ring).toBeVisible();
  await page.getByText("⚙ パレット").click();
  await page.locator("aside input[type=checkbox]").first().uncheck();
  await expect(ring).toBeHidden();
  await expect(page).toHaveURL(/layers=effects%2Ctype_info|layers=effects,type_info/);

  // URL 直開きでも同じ状態が復元される (M4 の核心)。
  await page.goto("/?fn=compute&layers=effects,type_info");
  await expect(page.locator("svg circle.op-dot").first()).toBeVisible(); // effects は出る
  await expect(page.locator("svg circle.main-ring").first()).toBeHidden();
  await expect(page.locator("aside input[type=checkbox]").first()).not.toBeChecked();
});

test("ベルカ切替: 別 SVG に差し替わり ?style=belka が付く", async ({ page }) => {
  await page.goto("/?fn=compute");
  await page.getByText("⚙ パレット").click();
  await page.getByLabel("ベルカ").check();
  await expect(page).toHaveURL(/style=belka/);
  await expect(page.locator("svg [class*=belka]").first()).toBeAttached();
});

test(".magia DSL: エクスポート → 適用の往復とエラー表示", async ({ page }) => {
  await page.goto("/?fn=compute");
  await page.getByText("⚙ パレット").click();
  await page.getByText(".magia (spec §8)").click();

  await page.getByRole("button", { name: "エクスポート" }).click();
  await expect(page.locator("aside textarea")).toHaveValue(
    "show: control_flow + effects + type_info",
  );

  await expect(page.locator("svg circle.main-ring").first()).toBeVisible();
  await page.locator("aside textarea").fill("show: effects");
  await page.getByRole("button", { name: "適用" }).click();
  await expect(page.locator("svg circle.main-ring").first()).toBeHidden();
  await expect(page).toHaveURL(/layers=effects(&|$)/);

  await page.locator("aside textarea").fill("hide: effects[io]");
  await page.getByRole("button", { name: "適用" }).click();
  await expect(page.locator("aside details details")).toContainText(
    "1行目: hide にカテゴリ指定 [...] はできません",
  );
});

test("SSE: ファイルに関数を足すと目次が自動更新される", async ({ page }) => {
  await page.goto("/?fn=greet");
  await expect(page.locator("nav button")).toHaveCount(2);
  writeFileSync(FIXTURE, `${INITIAL}\nfn newcomer() -> bool {\n    true\n}\n`);
  await expect(page.locator("nav button", { hasText: "newcomer" })).toBeVisible({
    timeout: 10_000,
  });
});

test("構文エラー: バナーが出て last-good の魔法陣を保持し、修復で復帰する", async ({ page }) => {
  await page.goto("/?fn=greet");
  await expect(page.locator("svg").first()).toBeVisible();

  writeFileSync(FIXTURE, "fn greet(name: &str) -> String { format!(\n");
  await expect(page.getByText("構文エラー:")).toBeVisible({ timeout: 10_000 });
  await expect(page.locator("svg").first()).toBeVisible(); // 図は消えない

  writeFileSync(FIXTURE, INITIAL);
  await expect(page.getByText("構文エラー:")).toBeHidden({ timeout: 10_000 });
});

test("操作ドットのホバー/選択がリアクティブに反映される (Phase 4.0.7)", async ({ page }) => {
  await page.goto("/?fn=compute");
  const dot = page.locator("svg circle.op-dot").first();
  await expect(dot).toBeVisible();
  await dot.hover();
  await expect(dot).toHaveClass(/op-hovered/);
  await dot.click();
  await expect(dot).toHaveClass(/op-selected/);
  await dot.click(); // 再クリックで選択解除
  await expect(dot).not.toHaveClass(/op-selected/);
});

test("凡例: 開閉式で色と記号の意味が参照できる (Phase 4.0.6)", async ({ page }) => {
  await page.goto("/?fn=greet");
  const legend = page.locator("details", { hasText: "凡例" }); // 魔法陣ペイン下 (4.0.6 判定)
  // 既定は閉 — 開くと効果カテゴリと図形の説明が見える。
  await expect(legend.getByText("純粋")).toBeHidden();
  await legend.locator("summary").click();
  await expect(legend.getByText("純粋")).toBeVisible();
  await expect(legend.getByText("メインリング = 関数本体", { exact: false })).toBeVisible();
  await expect(legend.getByText("シアン輪郭 = ホバー中")).toBeVisible();
  await expect(legend.getByText("生成 (値の誕生)")).toBeVisible();
});

test("書き起こし: スクリーンリーダー向け region が存在し内容を持つ", async ({ page }) => {
  await page.goto("/?fn=greet");
  const region = page.getByRole("region", { name: "呪文書き起こし" });
  await expect(region).toContainText("関数 greet", { useInnerText: false });
});
