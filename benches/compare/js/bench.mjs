// Benchmark two JS pipelines:
//   hast  — hast-util-from-html → hast-util-to-mdast → mdast-util-to-markdown
//   turndown — TurndownService with GFM plugin
//
// Run from this directory:
//   node bench.mjs
//
// Fixture files are loaded from ../../fixtures/ (benches/fixtures/ in the Rust project).

import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'

import { fromHtml } from 'hast-util-from-html'
import { toMdast } from 'hast-util-to-mdast'
import { gfmToMarkdown } from 'mdast-util-gfm'
import { toMarkdown } from 'mdast-util-to-markdown'

// Turndown ships CJS only; load it via createRequire so it works in an ESM file.
const require = createRequire(import.meta.url)
const TurndownService = require('turndown')
const { gfm } = require('turndown-plugin-gfm')
const turndown = new TurndownService()
turndown.use(gfm)

const __dirname = dirname(fileURLToPath(import.meta.url))
const fixturesDir = join(__dirname, '..', '..', 'fixtures')

const FIXTURE_NAMES = ['article', 'table', 'lists', 'code', 'large']
const WARMUP_MS = 1000
const BENCH_MS = 3000

function loadFixtures() {
  return FIXTURE_NAMES.map(name => {
    const html = readFileSync(join(fixturesDir, `${name}.html`), 'utf8')
    return {
      name,
      html,
      // Use UTF-8 byte length (not UTF-16 code units) to match Rust's len() for throughput.
      byteLen: Buffer.byteLength(html, 'utf8'),
    }
  })
}

function convertHast(html) {
  return toMarkdown(toMdast(fromHtml(html)), { extensions: [gfmToMarkdown()] })
}

function convertTurndown(html) {
  try {
    return turndown.turndown(html)
  } catch (e) {
    return null  // some fixtures trigger a turndown-plugin-gfm bug on degenerate tables
  }
}

function benchOne(label, fn, html, byteLen) {
  // Check if this fn works at all on this input before committing benchmark time
  if (fn(html) === null) {
    process.stdout.write(`${label.padEnd(25)}      ERROR (unsupported input)\n`)
    return
  }

  const warmupEnd = performance.now() + WARMUP_MS
  while (performance.now() < warmupEnd) fn(html)

  let iters = 0
  const start = performance.now()
  const end = start + BENCH_MS
  while (performance.now() < end) {
    fn(html)
    iters++
  }
  const elapsed = (performance.now() - start) / 1000
  const throughput = (byteLen * iters) / elapsed / (1024 * 1024)
  const nsPerOp = (elapsed / iters) * 1e9

  process.stdout.write(
    `${label.padEnd(25)} ${throughput.toFixed(2).padStart(8)} MiB/s  (${Math.round(nsPerOp / 1000)} µs/op)\n`
  )
}

const fixtures = loadFixtures()

for (const { name, html, byteLen } of fixtures) {
  benchOne(`hast/${name}`,     convertHast,     html, byteLen)
  benchOne(`turndown/${name}`, convertTurndown, html, byteLen)
}
