#!/usr/bin/env bun
// docs/images/MagiaMagica Social Preview 4案（単体HTML）.html の 4 案を
// 1案1ファイルで PNG 化する (GitHub Social Preview 想定 1280×640)。
//
// 出力: docs/images/social-{bright,gold}-<variant>.png
//
// 描画手段: Playwright の page.screenshot()。html2canvas を最初は使ったが、
// 金色変形文字 (`background-clip: text` + `linear-gradient`) を扱えないため
// 実ブラウザのラスタライズに切り替えた。フォント・グラデーション・SVG
// マスク全てが意匠通りに出る。

import path from 'node:path'
import fs from 'node:fs/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { createRequire } from 'node:module'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const require = createRequire(path.join(repoRoot, 'web/package.json'))
const { chromium } = require('@playwright/test')

const INPUT = path.join(repoRoot, 'docs/images/MagiaMagica Social Preview 4案（単体HTML）.html')
const OUT_DIR = path.join(repoRoot, 'docs/images')

const VARIANTS = [
  { idx: 1, file: 'social-bright-center.png',      label: 'まぎあ☆マギカ — Social · 中央' },
  { idx: 2, file: 'social-bright-side.png',        label: 'まぎあ☆マギカ — Social · サイド大シンボル' },
  { idx: 3, file: 'social-gold-center.png',        label: 'Magia Magica — Social · 中央' },
  { idx: 4, file: 'social-gold-watermark.png',     label: 'Magia Magica — Social · M透かし左寄せ' },
]

const NATIVE_W = 1280
const NATIVE_H = 640
// GitHub Social Preview は 1280×640・1MB 以下が規格。2倍解像度にすると
// 数 MB に膨れて GitHub の上限を超えるので、scale=1 で書き出す。
const SCALE = 1

const browser = await chromium.launch()
const page = await browser.newPage({
  viewport: { width: 1600, height: 1200 },
  deviceScaleFactor: SCALE,
})

page.on('pageerror', (e) => console.error('[pageerror]', e.message))

await page.goto(pathToFileURL(INPUT).href, { waitUntil: 'domcontentloaded' })
await page.waitForFunction(() => !document.getElementById('__bundler_thumbnail'), { timeout: 60000 })
await page.waitForLoadState('networkidle').catch(() => {})

// フォント読み込みを待つ。
await page.evaluate(async () => {
  document.body.offsetHeight
  await Promise.all(Array.from(document.fonts).map((f) => f.load().catch(() => null)))
  await document.fonts.ready
  await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
})

// プレビューの transform: scale() を剥がして .frame を実寸 1280×640 にする。
// 親 (.thumb / .scaler) の overflow:hidden + 縮小済みサイズも残るので、
// 1280×640 がフルに見えるよう親もサイズと overflow を上書きする。
await page.evaluate(({ w, h }) => {
  document.querySelectorAll('.gallery > figure.cell').forEach((cell) => {
    const frame = cell.querySelector('.frame')
    const scaler = cell.querySelector('.scaler')
    const thumb = cell.querySelector('.thumb')
    if (frame) {
      frame.style.transform = 'none'
      frame.style.transformOrigin = 'top left'
    }
    for (const el of [thumb, scaler]) {
      if (!el) continue
      el.style.width = w + 'px'
      el.style.height = h + 'px'
      el.style.overflow = 'visible'
      el.style.maxWidth = 'none'
    }
  })
}, { w: NATIVE_W, h: NATIVE_H })

for (const v of VARIANTS) {
  const handle = await page.locator(`.gallery > figure.cell:nth-of-type(${v.idx}) .frame`).elementHandle()
  if (!handle) throw new Error('cell not found: ' + v.idx)

  // .frame は実寸 1280×640 で配置されているので、要素そのままを撮る。
  // deviceScaleFactor=2 が乗って 2560×1280 になる。
  const out = path.join(OUT_DIR, v.file)
  await handle.screenshot({ path: out, omitBackground: false })
  console.log('wrote', path.relative(repoRoot, out), '—', v.label)
}

await browser.close()
