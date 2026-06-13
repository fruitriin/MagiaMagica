#!/usr/bin/env bun
// docs/images/MagiaMagica ロゴ（単体HTML）.html の 4 案を 1案1ファイルで PNG 化する
// (README 用 1280×360)。
//
// render-social-previews.mjs と同じく Playwright の elementHandle.screenshot() を
// 使う (html2canvas は金色グラデーション文字 `background-clip: text` を扱えないため)。
//
// 出力 (deviceScaleFactor=2 で 2560×720):
//   - docs/images/logo-readme.png                (案1: まぎあ☆マギカ · 地色あり)
//   - docs/images/logo-readme-transparent.png    (案2: まぎあ☆マギカ · 透過PNG)
//   - docs/images/magia-readme-gold.png          (案3: Magia Magica · 濃紺地)
//   - docs/images/magia-readme-gold-transparent.png  (案4: Magia Magica · 透過PNG)

import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { createRequire } from 'node:module'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const require = createRequire(path.join(repoRoot, 'web/package.json'))
const { chromium } = require('@playwright/test')

const INPUT = path.join(repoRoot, 'docs/images/MagiaMagica ロゴ（単体HTML）.html')
const OUT_DIR = path.join(repoRoot, 'docs/images')

const VARIANTS = [
  { idx: 1, file: 'logo-readme.png',                     label: 'まぎあ☆マギカ — README · 地色あり' },
  { idx: 2, file: 'logo-readme-transparent.png',         label: 'まぎあ☆マギカ — README · 透過PNG' },
  { idx: 3, file: 'magia-readme-gold.png',               label: 'Magia Magica — README · 濃紺地' },
  { idx: 4, file: 'magia-readme-gold-transparent.png',   label: 'Magia Magica — README · 透過PNG' },
]

const NATIVE_W = 1280
const NATIVE_H = 360
const SCALE = 2

const browser = await chromium.launch()
const page = await browser.newPage({
  viewport: { width: 1600, height: 1200 },
  deviceScaleFactor: SCALE,
})

page.on('pageerror', (e) => console.error('[pageerror]', e.message))

await page.goto(pathToFileURL(INPUT).href, { waitUntil: 'domcontentloaded' })
await page.waitForFunction(() => !document.getElementById('__bundler_thumbnail'), { timeout: 60000 })
await page.waitForLoadState('networkidle').catch(() => {})

await page.evaluate(async () => {
  document.body.offsetHeight
  await Promise.all(Array.from(document.fonts).map((f) => f.load().catch(() => null)))
  await document.fonts.ready
  await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
})

// プレビューの transform: scale() を剥がして .frame を実寸 1280×360 にする。
// 親 (.thumb / .scaler) の overflow:hidden + 縮小済みサイズも上書きする。
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
  const out = path.join(OUT_DIR, v.file)
  // 透過案は omitBackground:true で body の地色を抜く。
  const omit = v.file.includes('transparent')
  await handle.screenshot({ path: out, omitBackground: omit })
  console.log('wrote', path.relative(repoRoot, out), '—', v.label)
}

await browser.close()
