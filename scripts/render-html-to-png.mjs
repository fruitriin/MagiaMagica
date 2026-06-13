#!/usr/bin/env bun
// HTML を html2canvas で PNG に書き出すスクリプト。
//
// 使い方:
//   bun scripts/render-html-to-png.mjs <input.html> <output.png> [--width 1200] [--height 800] [--scale 2] [--selector "CSS"]
//
//   --selector を渡すと該当要素だけをキャプチャする (省略時は body 全体)
//
// 待ち合わせ:
//   1. バンドル展開ページ (#__bundler_thumbnail がある) なら、それが消えるまで待つ
//   2. document.fonts.ready で @font-face の読み込み完了を待つ
//   3. requestAnimationFrame を2回回してレイアウト確定を待つ
//   4. html2canvas (CDN) を注入して body をキャプチャ
//
// 注意:
//   - file:// から CDN を引くため、オフライン環境では失敗する
//   - Playwright は web/node_modules 側を使う (cd web && bun install --frozen-lockfile で導入)

import path from 'node:path'
import fs from 'node:fs/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { createRequire } from 'node:module'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const require = createRequire(path.join(repoRoot, 'web/package.json'))
const { chromium } = require('@playwright/test')

function parseArgs(argv) {
  const positional = []
  const opts = { width: 1200, height: 800, scale: 2, selector: null }
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--width') opts.width = Number(argv[++i])
    else if (a === '--height') opts.height = Number(argv[++i])
    else if (a === '--scale') opts.scale = Number(argv[++i])
    else if (a === '--selector') opts.selector = argv[++i]
    else positional.push(a)
  }
  return { positional, opts }
}

const { positional, opts } = parseArgs(process.argv.slice(2))
if (positional.length < 2) {
  console.error('usage: bun scripts/render-html-to-png.mjs <input.html> <output.png> [--width N] [--height N] [--scale N]')
  process.exit(1)
}

const inputPath = path.resolve(positional[0])
const outputPath = path.resolve(positional[1])

const browser = await chromium.launch()
const context = await browser.newContext({
  viewport: { width: opts.width, height: opts.height },
  deviceScaleFactor: opts.scale,
})
const page = await context.newPage()

page.on('console', (msg) => {
  if (msg.type() === 'error') console.error('[page]', msg.text())
})
page.on('pageerror', (err) => console.error('[pageerror]', err.message))

await page.goto(pathToFileURL(inputPath).href, { waitUntil: 'domcontentloaded' })

// Step 1. バンドル展開ページなら完了まで待つ (#__bundler_thumbnail が消える)。
//         バンドルでなければ即座に通過する。
await page.waitForFunction(() => !document.getElementById('__bundler_thumbnail'), { timeout: 60000 })

// バンドル展開後に新しい documentElement に差し替わるので、networkidle で
// 内部スクリプト・<link rel="stylesheet"> ・@font-face の読み込みを待つ。
await page.waitForLoadState('networkidle').catch(() => {})

// Step 2-3. フォント読み込みを待つ。
//   - document.fonts.ready は「今ロード中」のフォントが終わるまで。
//   - @font-face で宣言だけされていて要素で参照されていないものは load() を明示する。
//   - 最後に rAF を2回回してレイアウト・ペイントを確定させる。
await page.evaluate(async () => {
  document.body.offsetHeight // 強制レイアウト
  const all = Array.from(document.fonts).map((f) => f.load().catch(() => null))
  await Promise.all(all)
  await document.fonts.ready
  await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
})

// Step 4. html2canvas を注入してキャプチャ。
await page.addScriptTag({ url: 'https://cdn.jsdelivr.net/npm/html2canvas@1.4.1/dist/html2canvas.min.js' })

const dataUrl = await page.evaluate(async ({ scale, selector }) => {
  const target = selector ? document.querySelector(selector) : document.body
  if (!target) throw new Error('selector not found: ' + selector)
  const canvas = await window.html2canvas(target, {
    backgroundColor: null,
    scale,
    useCORS: true,
    logging: false,
  })
  return canvas.toDataURL('image/png')
}, { scale: opts.scale, selector: opts.selector })

await fs.mkdir(path.dirname(outputPath), { recursive: true })
const base64 = dataUrl.replace(/^data:image\/png;base64,/, '')
await fs.writeFile(outputPath, Buffer.from(base64, 'base64'))

await browser.close()
console.log('wrote', outputPath)
