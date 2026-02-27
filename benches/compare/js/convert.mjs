// Convert an HTML file to Markdown.
// Usage: node convert.mjs <impl> <path-to-file>
//   impl: hast | turndown

import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { fromHtml } from 'hast-util-from-html'
import { toMdast } from 'hast-util-to-mdast'
import { gfmToMarkdown } from 'mdast-util-gfm'
import { toMarkdown } from 'mdast-util-to-markdown'

const require = createRequire(import.meta.url)
const TurndownService = require('turndown')
const { gfm } = require('turndown-plugin-gfm')

const [impl, file] = process.argv.slice(2)
const html = readFileSync(file, 'utf8')

let md
if (impl === 'turndown') {
  const td = new TurndownService()
  td.use(gfm)
  try {
    md = td.turndown(html)
  } catch (e) {
    process.stderr.write(`turndown error: ${e.message}\n`)
    process.exit(1)
  }
} else {
  md = toMarkdown(toMdast(fromHtml(html)), { extensions: [gfmToMarkdown()] })
}

process.stdout.write(md)
