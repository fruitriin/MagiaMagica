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
        return helper(sum);
    }
    sum
}

fn helper(value: i32) -> i32 {
    value * 2
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
  await expect(page).toHaveURL(/\?pin=greet/);
  // 関数目次は既定で「表示中のみ」(ピン + 周辺) をシグネチャ表記で列挙する (Phase 4.1)。
  const tocButtons = page.locator("nav button");
  await expect(tocButtons).toHaveCount(3);
  await expect(tocButtons.first()).toContainText("fn greet");
  await expect(page.locator("main section pre").first()).toContainText("fn greet");
});

test("関数目次クリックで ?pin= が切り替わり、戻るで復帰する", async ({ page }) => {
  await page.goto("/?pin=greet");
  await page.locator("nav button", { hasText: "compute" }).click();
  await expect(page).toHaveURL(/\?pin=compute/);
  await expect(page.locator("main section pre").first()).toContainText("fn compute");
  await page.goBack();
  await expect(page).toHaveURL(/\?pin=greet/);
  await expect(page.locator("header span").first()).toHaveText("greet");
});

test("レイヤー toggle: 制御フローの要素が非表示になり URL に残る", async ({ page }) => {
  await page.goto("/?pin=compute");
  // 「存在しないから hidden」の偽陽性を防ぐ: 先に表示されていることを確かめる。
  const ring = page.locator("svg circle.main-ring").first();
  await expect(ring).toBeVisible();
  await page.getByText("⚙ パレット").click();
  await page.locator("aside input[type=checkbox]").first().uncheck();
  await expect(ring).toBeHidden();
  await expect(page).toHaveURL(/layers=effects%2Ctype_info|layers=effects,type_info/);

  // URL 直開きでも同じ状態が復元される (M4 の核心)。
  await page.goto("/?pin=compute&layers=effects,type_info");
  await expect(page.locator("svg circle.op-dot").first()).toBeVisible(); // effects は出る
  await expect(page.locator("svg circle.main-ring").first()).toBeHidden();
  await expect(page.locator("aside input[type=checkbox]").first()).not.toBeChecked();
});

test("ベルカ切替: 別 SVG に差し替わり ?style=belka が付く", async ({ page }) => {
  await page.goto("/?pin=compute");
  await page.getByText("⚙ パレット").click();
  await page.getByLabel("ベルカ").check();
  await expect(page).toHaveURL(/style=belka/);
  await expect(page.locator("svg [class*=belka]").first()).toBeAttached();
});

test(".magia DSL: エクスポート → 適用の往復とエラー表示", async ({ page }) => {
  await page.goto("/?pin=compute");
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
  await page.goto("/?pin=greet");
  await expect(page.locator("nav button")).toHaveCount(3);
  writeFileSync(FIXTURE, `${INITIAL}\nfn newcomer() -> bool {\n    true\n}\n`);
  await expect(page.locator("nav button", { hasText: "newcomer" })).toBeVisible({
    timeout: 10_000,
  });
});

test("構文エラー: バナーが出て last-good の魔法陣を保持し、修復で復帰する", async ({ page }) => {
  await page.goto("/?pin=greet");
  await expect(page.locator("svg").first()).toBeVisible();

  writeFileSync(FIXTURE, "fn greet(name: &str) -> String { format!(\n");
  await expect(page.getByText("構文エラー:")).toBeVisible({ timeout: 10_000 });
  await expect(page.locator("svg").first()).toBeVisible(); // 図は消えない

  writeFileSync(FIXTURE, INITIAL);
  await expect(page.getByText("構文エラー:")).toBeHidden({ timeout: 10_000 });
});

test("操作ドットのホバー/選択がリアクティブに反映される (Phase 4.0.7)", async ({ page }) => {
  await page.goto("/?pin=compute");
  const dot = page.locator("svg circle.op-dot").first();
  await expect(dot).toBeVisible();
  await dot.hover();
  await expect(dot).toHaveClass(/op-hovered/);
  await dot.click();
  await expect(dot).toHaveClass(/op-selected/);
  await dot.click(); // 再クリックで選択解除
  await expect(dot).not.toHaveClass(/op-selected/);
});

test("ピン中心ビュー: 周辺チップが出てクリックでピンが移動し、履歴で戻れる (Phase 4.1)", async ({
  page,
}) => {
  await page.goto("/?pin=greet");
  // 周辺チップ (compute) が表示される。
  const chip = page.locator("svg g[role=button]", { hasText: "compute" });
  await expect(chip).toBeVisible();
  // チップクリックでピンが移動し、URL とヘッダが追従する。
  await chip.click();
  await expect(page).toHaveURL(/\?pin=compute/);
  await expect(page.locator("header span").first()).toHaveText("compute");
  // 旧フォーカス (greet) が今度はチップとして現れる。
  await expect(page.locator("svg g[role=button]", { hasText: "greet" })).toBeVisible();
  // ブラウザ戻るでピンが戻る。
  await page.goBack();
  await expect(page).toHaveURL(/\?pin=greet/);
  await expect(page.locator("header span").first()).toHaveText("greet");
});

test("ピン中心ビュー: reduced-motion では遷移が即時 (transition なし)", async ({ browser }) => {
  const context = await browser.newContext({ reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:4810/?pin=greet");
  const node = page.locator("svg g.pin-node").first();
  await expect(node).toBeVisible();
  const transition = await node.evaluate((el) => getComputedStyle(el).transitionDuration);
  expect(transition).toBe("0s");
  await context.close();
});

test("関数目次: 「表示中のみ」フィルタはピン + 周辺を距離順でシグネチャ表示する", async ({
  page,
}) => {
  await page.goto("/?pin=greet");
  // 既定 ON: シグネチャ表記 (fn greet...) で列挙される。
  await expect(page.locator("nav button").first()).toContainText("fn greet");
  // OFF にすると従来の全関数 (qualified 名) 列挙に戻る。
  await page.getByLabel("表示中のみ").uncheck();
  await expect(page.locator("nav button").first()).toHaveText("greet");
});

test("召喚印インスペクタ: クリックでコードが出て、そこからピンできる (Phase 4.1)", async ({
  page,
}) => {
  await page.goto("/?pin=compute");
  // compute の魔法陣には helper 呼び出しの召喚印がある。
  const glyph = page.locator(".pin-view circle.summon-glyph").first();
  await expect(glyph).toBeVisible();
  await glyph.click();
  // ポップオーバーに呼び出し名・呼び出し式 (引数込み原文)・コード断片が出る。
  const popover = page.getByRole("dialog", { name: "呼び出し先" });
  await expect(popover).toBeVisible();
  await expect(popover.locator("code").first()).toContainText("helper");
  await expect(popover.locator(".call-excerpt")).toContainText("helper(sum)");
  await expect(popover).toContainText("fn helper");
  // コード断片クリックでピン遷移し、ポップオーバーは閉じる。
  await popover.locator("[title*='をピン']").click();
  await expect(page).toHaveURL(/\?pin=helper/);
  await expect(page.locator("header span").first()).toHaveText("helper");
  await expect(popover).toBeHidden();
});

test("召喚印インスペクタ: 外部呼び出しは定義なしの案内になる", async ({ page }) => {
  await page.goto("/?pin=greet");
  // greet の召喚印は format! (マクロ、ファイル内に定義なし)。
  await page.locator(".pin-view circle.summon-glyph").first().click();
  const popover = page.getByRole("dialog", { name: "呼び出し先" });
  await expect(popover).toContainText("定義がない外部呼び出し");
  // 外部呼び出しでも呼び出し式 (引数込み原文) は出る。
  await expect(popover.locator(".call-excerpt")).toContainText('format!("Hello, {name}")');
  // Esc ではなく外側クリックで閉じる。
  await page.mouse.click(10, 500);
  await expect(popover).toBeHidden();
});

test("ホバープレビュー: 召喚印で呼び出し式 + 固定ヒント、操作ドットで文の断片が出る", async ({
  page,
}) => {
  await page.goto("/?pin=compute");
  const preview = page.locator(".hover-preview");
  // 召喚印 (helper 呼び出し): 呼び出し式 + 「クリックで固定」ヒント。
  const glyph = page.locator(".pin-view circle.summon-glyph").first();
  await expect(glyph).toBeVisible();
  await glyph.hover();
  await expect(preview).toContainText("helper(sum)");
  await expect(preview).toContainText("クリックで固定");
  // 操作ドット (plain statement): 文全体の断片。ピン操作はないのでヒントなし。
  await page.locator(".pin-view circle.op-dot").first().hover();
  await expect(preview).toContainText("let sum = a + b;");
  await expect(preview).not.toContainText("クリックで固定");
  // 分岐の補助リング (線上ホバー): 腕のガード条件が出る (追加要望4)。
  await page.locator(".pin-view circle.aux-ring").first().hover();
  await expect(preview).toContainText("if sum > 10");
  // 離れると消える。
  await page.mouse.move(10, 500);
  await expect(preview).toBeHidden();
});

test("固定インスペクタの上にホバープレビューが重なり、固定は出しっぱなしになる", async ({
  page,
}) => {
  await page.goto("/?pin=compute");
  const glyph = page.locator(".pin-view circle.summon-glyph").first();
  await glyph.click(); // 固定
  const popover = page.getByRole("dialog", { name: "呼び出し先" });
  await expect(popover).toBeVisible();
  // 固定したままホバー → プレビューが固定の上に併存する (薄幕なし)。
  // ポップオーバーはクリック地点 +12px に出るため、クリックした召喚印自体は
  // 覆われずホバーできる (覆われたドットに届かないのは実ブラウザと同じ)。
  await page.mouse.move(10, 500); // いったん離れて mouseenter を再発火させる
  await glyph.hover();
  await expect(page.locator(".hover-preview")).toBeVisible();
  await expect(popover).toBeVisible();
});

test("シグネチャ円弧はクリック判定を持たない (pointer-events: none)", async ({ page }) => {
  await page.goto("/?pin=compute");
  const signature = page.locator(".pin-view text.signature").first();
  await expect(signature).toBeVisible();
  const pointerEvents = await signature.evaluate((el) => getComputedStyle(el).pointerEvents);
  expect(pointerEvents).toBe("none");
});

test("Spell Diff on web: rev 指定で差分ハローが出て、保存で live に追従する (Phase 4.3.7)", async ({
  page,
}) => {
  await page.goto("/?pin=compute");
  // パレットから HEAD を基準に指定 (fixture は HEAD = 初期内容に固定)。
  await page.getByText("⚙ パレット").click();
  await page.getByLabel("diff 基準リビジョン").fill("HEAD");
  await page.getByRole("button", { name: "比較" }).click();
  await expect(page).toHaveURL(/diff=HEAD/);
  // 初期内容と同一なのでハローなし + 要約に変化なしが出る。
  await expect(page.locator(".overlay-diff circle")).toHaveCount(0);
  // ファイルを変更すると SSE → 再計算で金ハロー (追加) が現れる — live diff。
  writeFileSync(
    FIXTURE,
    INITIAL.replace("let sum = a + b;", "let sum = a + b;\n    let doubled = sum * 2;"),
  );
  await expect(page.locator(".overlay-diff circle.diff-changed").first()).toBeVisible();
  await expect(page.locator("aside")).toContainText("操作数");
  // クリアで従来表示へ。
  await page.getByRole("button", { name: "クリア" }).click();
  await expect(page).not.toHaveURL(/diff=/);
  await expect(page.locator(".overlay-diff")).toHaveCount(0);
});

test("Spell Diff on web: 不正な rev は案内文で受ける (UI を壊さない)", async ({ page }) => {
  await page.goto("/?pin=greet&diff=no_such_rev");
  await page.getByText("⚙ パレット").click();
  await expect(page.locator("aside")).toContainText("no_such_rev");
  // 魔法陣自体は通常表示のまま。
  await expect(page.locator(".pin-view circle.main-ring").first()).toBeVisible();
});

test("監視ファイル切替: ヘッダのドロップダウンで別ファイルへ移り URL で復元できる (Phase 4.4.5)", async ({
  page,
}) => {
  await page.goto("/?pin=greet");
  const picker = page.getByLabel("監視ファイル");
  await expect(picker).toHaveValue("sample.rs");
  // 切替 → SSE 経由で関数一覧・ヘッダが新ファイルに追従し、?file= が付く。
  await picker.selectOption("orbit.rs");
  await expect(page.locator("header span").first()).toHaveText("orbit", { timeout: 10_000 });
  await expect(page).toHaveURL(/file=orbit\.rs/);
  await expect(page.locator("nav button", { hasText: "orbit" })).toBeVisible();
  // URL 直開きでも復元される (希望 → POST → SSE 追従)。
  await page.goto("/?file=orbit.rs");
  await expect(page.locator("header span").first()).toHaveText("orbit", { timeout: 10_000 });
  await expect(picker).toHaveValue("orbit.rs");
  // 戻し (他テストは sample.rs 前提 — beforeEach はファイル内容しか戻さないため)。
  await picker.selectOption("sample.rs");
  await expect(page.locator("header span").first()).toHaveText("greet", { timeout: 10_000 });
});

test("呼び出しジャンプ: チップに関係マークが出て、召喚印ホバーで対応チップが光る (Phase 4.4)", async ({
  page,
}) => {
  await page.goto("/?pin=compute");
  // compute は helper を呼ぶ → helper チップに「→」(フォーカスが呼ぶ)。
  const helperChip = page.locator("svg g[role=button]", { hasText: "helper" });
  await expect(helperChip).toBeVisible();
  await expect(helperChip.locator("text.relation-mark")).toHaveText("→");
  // 無関係な greet チップにはマークなし。
  const greetChip = page.locator("svg g[role=button]", { hasText: "greet" });
  await expect(greetChip.locator("text.relation-mark")).toHaveCount(0);
  // 召喚印 (helper 呼び出し) をホバー → 対応する helper チップがリンク強調。
  await page.locator(".pin-view circle.summon-glyph").first().hover();
  await expect(helperChip.locator("circle.link-highlight")).toBeVisible();
  await page.mouse.move(10, 500);
  await expect(helperChip.locator("circle.link-highlight")).toHaveCount(0);
  // 逆方向: helper を pin すると compute チップに「←」(フォーカスを呼ぶ)。
  await page.goto("/?pin=helper");
  const computeChip = page.locator("svg g[role=button]", { hasText: "compute" });
  await expect(computeChip.locator("text.relation-mark")).toHaveText("←");
});

test("凡例: 開閉式で色と記号の意味が参照できる (Phase 4.0.6)", async ({ page }) => {
  await page.goto("/?pin=greet");
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
  await page.goto("/?pin=greet");
  const region = page.getByRole("region", { name: "呪文書き起こし" });
  await expect(region).toContainText("関数 greet", { useInnerText: false });
});

test("俯瞰: トグルで全ファイルカードが並び、カードクリックでズームインする (Phase 4.5 M1)", async ({
  page,
}) => {
  await page.goto("/?pin=greet");
  // ヘッダの俯瞰トグル → ファイルカード一覧 (?scope=workspace が付く)。
  await page.getByRole("button", { name: "俯瞰" }).click();
  await expect(page).toHaveURL(/scope=workspace/);
  const orbitCard = page.locator("button.file-card", { hasText: "orbit.rs" });
  await expect(orbitCard).toBeVisible();
  // 現在ファイル (sample.rs) のカードはハイライト + 関数名抜粋が載る。
  const sampleCard = page.locator("button.file-card", { hasText: "sample.rs" });
  await expect(sampleCard).toHaveClass(/border-cyan-600/);
  await expect(sampleCard).toContainText("greet");
  // ファイル横断の呼び出し関係 (M2 前段): orbit.rs は現在ファイル (sample.rs の
  // greet) を呼ぶ → 「←」マーク (4.4 のチップマークと同じ語彙)。
  await expect(orbitCard.locator(".file-relation")).toHaveText("←");
  await expect(sampleCard.locator(".file-relation")).toHaveCount(0);
  // カードクリック → ファイル切替 (4.4.5) + ピン中心ビューへズームイン。
  await orbitCard.click();
  await expect(page.locator("header span").first()).toHaveText("orbit", { timeout: 10_000 });
  await expect(page).not.toHaveURL(/scope=workspace/);
  await expect(page.locator(".pin-view")).toBeVisible();
  // URL 直開きでも俯瞰が復元される。
  await page.goto("/?scope=workspace");
  await expect(page.locator("button.file-card", { hasText: "sample.rs" })).toBeVisible();
  // 戻し (他テストは sample.rs 前提)。
  await page.getByRole("button", { name: "ピンに戻る" }).click();
  await page.getByLabel("監視ファイル").selectOption("sample.rs");
  await expect(page.locator("header span").first()).toHaveText("greet", { timeout: 10_000 });
});
