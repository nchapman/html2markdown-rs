# html-to-markdown-rs — Implementation Plan

## Goal

A CommonMark-compliant HTML-to-Markdown converter for Rust using an AST-to-AST architecture. HTML is parsed into a tree, transformed into a Markdown AST (MDAST), then serialized to a Markdown string. The two-phase design separates semantic mapping from formatting, making each phase independently testable.

## Architecture

```
HTML string
    │
    ▼
html5ever (parse)
    │
    ▼
html5ever RcDom (HTML tree)
    │
    ▼
hast-to-mdast (transform)
    │
    ▼
MDAST (Markdown AST)
    │
    ▼
mdast-stringify (serialize)
    │
    ▼
Markdown string
```

## Crate Structure

Single crate, not a workspace — consistent with readability-rs, trafilatura-rs, and justext-rs. The logical layers live in modules, not separate crates.

```
html-to-markdown-rs/
├── Cargo.toml
├── CLAUDE.md
├── PLAN.md
├── clippy.toml
├── src/
│   ├── lib.rs              # Public API: convert(), Options, builder
│   ├── error.rs            # HtmlToMarkdownError enum
│   ├── mdast.rs            # MDAST node types (enum + structs)
│   ├── hast_to_mdast/      # HTML tree → MDAST transform
│   │   ├── mod.rs          # State, one(), all(), entry point
│   │   ├── handlers.rs     # Element handlers (a, blockquote, code, etc.)
│   │   ├── wrap.rs         # Implicit paragraph detection, block-in-inline splitting
│   │   └── whitespace.rs   # Whitespace normalization (pre + post processing)
│   ├── stringify/           # MDAST → Markdown string
│   │   ├── mod.rs          # State, handle(), entry point
│   │   ├── handlers.rs     # Node type handlers (heading, paragraph, list, etc.)
│   │   ├── escape.rs       # Context-sensitive escaping (unsafe patterns)
│   │   ├── flow.rs         # Block-level container serialization + join rules
│   │   └── phrasing.rs     # Inline container serialization + peek
│   └── pulldown.rs         # pulldown-cmark Events → MDAST (for testing/interop)
├── tests/
│   ├── common/mod.rs       # Shared test helpers
│   ├── fixtures.rs         # hast-util-to-mdast fixture tests (130 cases)
│   ├── commonmark.rs       # CommonMark round-trip tests (657 examples)
│   ├── regression.rs       # Bug regression tests (grows over time)
│   └── integration.rs      # End-to-end API tests
├── test-fixtures/           # Copied from refs/hast-util-to-mdast/test/fixtures/
│   ├── a/
│   │   ├── index.html
│   │   └── index.md
│   ├── blockquote/
│   │   ├── index.html
│   │   └── index.md
│   └── ... (130 fixture directories)
└── benches/
    └── conversion.rs       # Criterion benchmarks
```

## Source References

| Module | Primary Reference | Notes |
|--------|-------------------|-------|
| `mdast.rs` | [mdast spec](../refs/mdast/) | ~25 node types as Rust enum |
| `hast_to_mdast/` | [hast-util-to-mdast](../refs/hast-util-to-mdast/) | 28 element handlers, state machine |
| `stringify/` | [mdast-util-to-markdown](../refs/mdast-util-to-markdown/) | Serializer with configurable formatting |
| `stringify/escape.rs` | [mdast-util-to-markdown unsafe.js](../refs/mdast-util-to-markdown/lib/unsafe.js) + [ESCAPING.md](../refs/html-to-markdown/ESCAPING.md) | Context-sensitive escaping |
| `pulldown.rs` | pulldown-cmark crate docs | Thin adapter for round-trip testing |
| Test fixtures | [hast-util-to-mdast fixtures](../refs/hast-util-to-mdast/test/fixtures/) | 130 input/output pairs |
| CommonMark tests | [commonmark-spec](../refs/commonmark-spec/spec.txt) | 657 examples |

## Public API Design

Following the conventions of our other `-rs` crates: builder pattern with `with_*` methods, `Default` impl, free function entry point.

```rust
use html_to_markdown::{convert, Options};

// Simple — sensible defaults
let markdown = convert("<h1>Hello</h1><p>World</p>")?;
assert_eq!(markdown, "# Hello\n\nWorld\n");

// Configured
let markdown = convert_with(html, Options::new()
    .with_heading_style(HeadingStyle::Setext)
    .with_bullet('−')
    .with_emphasis('_')
    .with_fence('~')
)?;

// AST access (for advanced users)
let mdast = html_to_mdast(html)?;
let markdown = mdast_to_string(&mdast, &StringifyOptions::default())?;
```

### Options (mirrors mdast-util-to-markdown config)

```rust
pub struct Options {
    // Serializer formatting
    pub heading_style: HeadingStyle,  // Atx (default) | Setext
    pub bullet: char,                 // '*' (default) | '+' | '-'
    pub bullet_ordered: char,         // '.' (default) | ')'
    pub emphasis: char,               // '*' (default) | '_'
    pub strong: char,                 // '*' (default) | '_'
    pub fence: char,                  // '`' (default) | '~'
    pub rule: char,                   // '*' (default) | '-' | '_'
    pub rule_repetition: u8,          // 3 (default)
    pub rule_spaces: bool,            // false (default)
    pub close_atx: bool,             // false (default)
    pub list_item_indent: ListItemIndent, // One (default) | Tab | Mixed
    pub increment_list_marker: bool,  // true (default)
    pub quote: char,                  // '"' (default) | '\''
    pub fences: bool,                 // true (default) — use fenced code blocks
    pub resource_link: bool,          // false (default) — never autolinks

    // Transformer options
    pub newlines: bool,               // false (default) — preserve newlines in whitespace normalization
}
```

### Error Type

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HtmlToMarkdownError {
    #[error("HTML parse error: {0}")]
    Parse(String),
}
```

## MDAST Node Types

Based on the [mdast spec](https://github.com/syntax-tree/mdast). ~25 node types as a Rust enum with struct variants.

```rust
/// Position in source (optional, not used in HTML→MDAST since HTML positions
/// don't map meaningfully to Markdown positions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    pub line: usize,   // 1-indexed
    pub column: usize, // 1-indexed
    pub offset: usize, // 0-indexed byte offset
}

/// Markdown AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // Document
    Root(Root),

    // Flow (block) content
    Blockquote(Blockquote),
    Code(Code),
    Heading(Heading),
    Html(Html),
    List(List),
    ListItem(ListItem),
    ThematicBreak(ThematicBreak),
    Definition(Definition),
    Paragraph(Paragraph),

    // Phrasing (inline) content
    Break(Break),
    Delete(Delete),           // GFM
    Emphasis(Emphasis),
    Image(Image),
    ImageReference(ImageReference),
    InlineCode(InlineCode),
    Link(Link),
    LinkReference(LinkReference),
    Strong(Strong),
    Text(Text),

    // Table (GFM)
    Table(Table),
    TableRow(TableRow),
    TableCell(TableCell),

    // Frontmatter (for completeness — not produced by HTML conversion)
    Yaml(Yaml),

    // Footnotes (GFM)
    FootnoteDefinition(FootnoteDefinition),
    FootnoteReference(FootnoteReference),
}
```

Each variant holds a struct with the node's data plus `children: Vec<Node>` where applicable. Leaf nodes hold `value: String` instead. Full struct definitions follow the mdast spec exactly.

### Content Model Helpers

```rust
impl Node {
    pub fn children(&self) -> Option<&[Node]> { ... }
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> { ... }
    pub fn is_phrasing(&self) -> bool { ... }
    pub fn is_flow(&self) -> bool { ... }
}
```

## Transformer: hast_to_mdast

Port of [hast-util-to-mdast](https://github.com/syntax-tree/hast-util-to-mdast). This is the core novel work.

### State

```rust
pub(crate) struct State {
    /// Base URL from <base> element (first one wins)
    pub frozen_base_url: Option<Url>,
    /// Whether we're currently inside a table (nested tables → text)
    pub in_table: bool,
    /// Nesting depth for <q> elements (cycles quote characters)
    pub q_nesting: usize,
    /// Elements indexed by their id attribute
    pub element_by_id: HashMap<String, /* node ref */>,
    /// Options
    pub options: TransformOptions,
}

impl State {
    /// Convert a single HTML node to MDAST node(s).
    pub fn one(&mut self, node: &HtmlNode, parent: Option<&HtmlNode>) -> Vec<Node>;

    /// Convert all children of an HTML node to MDAST nodes.
    pub fn all(&mut self, parent: &HtmlNode) -> Vec<Node>;

    /// Wrap nodes as flow content (implicit paragraph detection).
    pub fn to_flow(&mut self, nodes: Vec<Node>) -> Vec<Node>;

    /// Ensure all nodes match a specific type, wrapping stragglers.
    pub fn to_specific_content<F>(&mut self, nodes: Vec<Node>, build: F) -> Vec<Node>
    where F: Fn(Vec<Node>) -> Node;

    /// Resolve a URL against the frozen base URL.
    pub fn resolve(&self, url: &str) -> String;
}
```

### Element Handlers

28 handler functions, each mapping an HTML element to MDAST node(s). Port from `refs/hast-util-to-mdast/lib/handlers/`.

| Handler | HTML Elements | MDAST Output |
|---------|--------------|--------------|
| `handle_a` | `<a>` | `Link` |
| `handle_base` | `<base>` | (sets `frozen_base_url`) |
| `handle_blockquote` | `<blockquote>` | `Blockquote` |
| `handle_br` | `<br>` | `Break` |
| `handle_code_inline` | `<code>`, `<kbd>`, `<samp>`, `<tt>`, `<var>` | `InlineCode` |
| `handle_code_block` | `<pre>`, `<listing>`, `<xmp>` | `Code` |
| `handle_comment` | HTML comments | `Html` |
| `handle_del` | `<del>`, `<s>`, `<strike>` | `Delete` |
| `handle_dl` | `<dl>` | `List` (groups dt/dd pairs) |
| `handle_em` | `<em>`, `<i>`, `<mark>`, `<u>` | `Emphasis` |
| `handle_heading` | `<h1>`–`<h6>` | `Heading` (depth from tag) |
| `handle_hr` | `<hr>` | `ThematicBreak` |
| `handle_iframe` | `<iframe>` | `Link` (if src + title) |
| `handle_img` | `<img>`, `<image>` | `Image` |
| `handle_input` | `<input>` | varies by type |
| `handle_li` | `<li>`, `<dt>`, `<dd>` | `ListItem` |
| `handle_list` | `<ol>`, `<ul>`, `<dir>` | `List` |
| `handle_media` | `<audio>`, `<video>` | `Link` |
| `handle_p` | `<p>`, `<summary>` | `Paragraph` |
| `handle_q` | `<q>` | `Text` (with quotes) |
| `handle_root` | root | `Root` |
| `handle_select` | `<select>` | `Text` |
| `handle_strong` | `<strong>`, `<b>` | `Strong` |
| `handle_table` | `<table>` | `Table` or `Text` |
| `handle_table_cell` | `<td>`, `<th>` | `TableCell` |
| `handle_table_row` | `<tr>` | `TableRow` |
| `handle_text` | text nodes | `Text` |
| `handle_textarea` | `<textarea>` | `Text` |
| `handle_wbr` | `<wbr>` | `Text` (zero-width space) |

**Element categories** (no dedicated handler):

- **Ignore** (return nothing): `applet`, `area`, `basefont`, `bgsound`, `caption`, `col`, `colgroup`, `command`, `content`, `datalist`, `dialog`, `element`, `embed`, `frame`, `frameset`, `isindex`, `keygen`, `link`, `math`, `menu`, `menuitem`, `meta`, `nextid`, `noembed`, `noframes`, `optgroup`, `option`, `param`, `script`, `shadow`, `source`, `spacer`, `style`, `svg`, `template`, `title`, `track`
- **Pass-through** (recurse into children, no wrapping): `abbr`, `acronym`, `bdi`, `bdo`, `big`, `blink`, `button`, `canvas`, `cite`, `data`, `details`, `dfn`, `font`, `ins`, `label`, `map`, `marquee`, `meter`, `nobr`, `noscript`, `object`, `output`, `progress`, `rb`, `rbc`, `rp`, `rt`, `rtc`, `ruby`, `slot`, `small`, `span`, `sup`, `sub`, `tbody`, `tfoot`, `thead`, `time`
- **Flow wrappers** (children wrapped as flow content): `address`, `article`, `aside`, `body`, `center`, `div`, `fieldset`, `figcaption`, `figure`, `form`, `footer`, `header`, `hgroup`, `html`, `legend`, `main`, `multicol`, `nav`, `picture`, `section`

### Hard Problems in the Transformer

#### 1. Implicit Paragraph Detection (`wrap.rs`)

When a flow container has mixed phrasing + block children, phrasing runs must be wrapped in implicit `Paragraph` nodes. The `wrap()` function:

1. **Flatten straddling elements**: If a `Link` or `Delete` contains block content, split it — the inline wrapper distributes around each block child. Example: `<a href="x">text<h1>heading</h1>more</a>` → `[Link("text")], Heading([Link("heading")]), [Link("more")]`.
2. **Separate runs**: Walk children, group consecutive phrasing nodes vs. block nodes.
3. **Wrap phrasing runs**: Non-whitespace phrasing runs become `Paragraph` nodes. Whitespace-only runs are dropped.

#### 2. Whitespace Normalization (`whitespace.rs`)

Two phases:
1. **Pre-processing**: Collapse whitespace in the HTML tree according to CSS whitespace rules before transformation. Port the logic from `rehype-minify-whitespace`.
2. **Post-processing**: After MDAST is built, merge adjacent `Text` nodes, collapse whitespace around line endings, trim leading/trailing whitespace in headings/paragraphs/root, remove empty text nodes.

#### 3. Nested Tables

When `state.in_table` is true and a `<table>` is encountered, serialize the inner table as plain text instead of a nested `Table` node (MDAST/Markdown doesn't support nested tables).

## Serializer: stringify

Port of [mdast-util-to-markdown](https://github.com/syntax-tree/mdast-util-to-markdown). All formatting choices live here.

### State

```rust
pub(crate) struct State<'a> {
    /// Configuration
    pub options: &'a StringifyOptions,
    /// Stack of construct names for escaping scope
    pub stack: Vec<ConstructName>,
    /// Current list bullet (may switch to avoid conflicts)
    pub bullet_current: Option<char>,
    /// Previous list's bullet (for alternation)
    pub bullet_last_used: Option<char>,
    /// Unsafe patterns for context-sensitive escaping
    pub unsafe_patterns: Vec<UnsafePattern>,
    /// Join functions for flow content spacing
    pub join: Vec<JoinFn>,
}
```

### Context-Sensitive Escaping (`escape.rs`)

The hardest problem in the serializer. Only escape Markdown syntax characters when they would actually trigger formatting in context.

**UnsafePattern** definition (port of `unsafe.js`):

```rust
pub(crate) struct UnsafePattern {
    pub character: char,
    pub before: Option<Regex>,
    pub after: Option<Regex>,
    pub at_break: bool,
    pub in_construct: Vec<ConstructName>,
    pub not_in_construct: Vec<ConstructName>,
}
```

**Core patterns** (~20 patterns covering `*`, `_`, `#`, `>`, `[`, `]`, `-`, `=`, `` ` ``, `<`, `&`, `\`, `~`, `|`, `!`, `:`):

The `safe()` function:
1. Constructs virtual string: `before + input + after`
2. Checks each unsafe pattern against the current construct stack
3. For matches: uses `\` backslash for ASCII punctuation, character references for others
4. Handles attention encoding for emphasis/strong boundary characters

### Node Handlers

One handler per MDAST node type. Each returns a `String`.

| Handler | Key Behavior |
|---------|-------------|
| `heading` | ATX (`# ...`) or setext (`...\n===`), encode leading space as char ref |
| `paragraph` | `container_phrasing()` with blank lines around |
| `blockquote` | Prefix each line with `> ` |
| `list` | Delegates to list items, handles bullet alternation |
| `list_item` | Computes indent, handles `checked` for task lists |
| `code` | Fenced (`` ``` ``) or indented, chooses fence char to avoid conflicts |
| `inline_code` | Chooses backtick count to avoid conflicts, pads if starts/ends with space/backtick |
| `emphasis` | `*text*` or `_text_`, handles attention encoding |
| `strong` | `**text**` or `__text__`, handles attention encoding |
| `link` | `[text](url "title")` or autolink `<url>` |
| `image` | `![alt](url "title")` |
| `thematic_break` | `***` / `---` / `___` with configurable style |
| `break_node` | `\` + newline (or two spaces + newline) |
| `html` | Pass through raw |
| `text` | Run through `safe()` for escaping |
| `table` | Align columns, pad cells, header separator row |
| `delete` | `~~text~~` |
| `definition` | `[label]: url "title"` |

### Flow Serialization (`flow.rs`)

Block children are separated by blank lines. The `between()` function consults join rules to determine spacing:
- `0` → no blank line (tight)
- `1` → one blank line (loose)
- `false` → incompatible, insert `<!---->` HTML comment separator

### Phrasing Serialization (`phrasing.rs`)

Inline children are concatenated. Uses `peek()` on next sibling's handler to determine the `after` context for escaping.

## pulldown-cmark Bridge (`pulldown.rs`)

Thin adapter: `pulldown_cmark::Event` stream → `Node` (MDAST). Used for round-trip testing only.

Maps pulldown-cmark events to MDAST nodes:
- `Start(Tag::Heading(..))` / `End(..)` → `Heading`
- `Start(Tag::Paragraph)` / `End(..)` → `Paragraph`
- `Text(s)` → `Text`
- `Code(s)` → `InlineCode`
- etc.

Uses a stack-based builder: push on `Start`, pop on `End`, collect children.

## Dependencies

```toml
[dependencies]
html5ever = "0.29"          # HTML parsing (spec-compliant)
markup5ever = "0.14"        # HTML/DOM type definitions
regex = "1"                 # Escaping pattern matching
url = "2"                   # URL resolution (<base> support)
thiserror = "2"             # Error types
tracing = { version = "0.1", optional = true }

[features]
default = []
tracing = ["dep:tracing"]

[dev-dependencies]
pretty_assertions = "1"
pulldown-cmark = "0.12"     # Markdown → HTML for round-trip testing
serde = { version = "1", features = ["derive"] }
serde_json = "1"            # CommonMark spec JSON loading
criterion = { version = "0.5", features = ["html_reports"] }
```

Note: `pulldown-cmark` is a dev-dependency only — the bridge module is `#[cfg(test)]`.

## Testing Strategy

### Layer 1: Element-Level Fixtures (130 cases)

Copy the 130 test fixture directories from `refs/hast-util-to-mdast/test/fixtures/`. Each has `index.html` (input) and `index.md` (expected output).

Test: `convert(input_html) == expected_md` for each fixture.

These cover every element handler plus edge cases: straddling, implicit paragraphs, nested tables, whitespace handling, headless tables, colspan/rowspan, checkbox extraction, media fallbacks, quote nesting, etc.

### Layer 2: CommonMark Round-Trip (657 examples)

Parse the 657 examples from `refs/commonmark-spec/spec.txt`. For each:
1. Take the expected HTML
2. Convert to Markdown with our tool
3. Parse our Markdown back to HTML with `pulldown-cmark`
4. Compare against the original HTML for semantic equivalence

This is the objective correctness oracle. Failures indicate our converter produces Markdown that doesn't round-trip to the same HTML.

Note: Not all 657 will pass — some CommonMark constructs have no HTML→Markdown round-trip (e.g., reference links, indented code). Track expected failures explicitly.

### Layer 3: Regression File

Every bug found becomes a test case in `tests/regression.rs`. Never delete one.

### Layer 4: Real-World Corpus (future)

After the core is solid, collect 30-50 real HTML pages and benchmark against other converters using round-trip fidelity.

## Implementation Phases

### Phase 1: Foundation

**Goal**: Project skeleton, MDAST types, basic infrastructure.

Files:
- `Cargo.toml` — dependencies, metadata, edition 2021, MSRV 1.80
- `clippy.toml` — `msrv = "1.80"`
- `CLAUDE.md` — project conventions and workflow
- `src/lib.rs` — public API stubs (`convert`, `convert_with`, `html_to_mdast`, `mdast_to_string`)
- `src/error.rs` — `HtmlToMarkdownError` enum
- `src/mdast.rs` — all MDAST node types, content model helpers

Deliverable: `cargo build` succeeds, MDAST types are complete and tested.

### Phase 2: HTML Parsing Layer

**Goal**: Parse HTML string into a walkable tree using html5ever.

Files:
- `src/hast_to_mdast/mod.rs` — `State` struct, `one()`, `all()` dispatch, HTML parsing via `html5ever::parse_document`

Deliverable: HTML string → html5ever `RcDom` tree, walkable. Dispatch skeleton routes elements to handler stubs.

### Phase 3: Core Element Handlers

**Goal**: Implement the most common element handlers.

Files:
- `src/hast_to_mdast/handlers.rs`

Handlers in priority order:
1. `text`, `root` — foundational
2. `p`, `heading`, `br`, `hr` — basic block elements
3. `a`, `img`, `strong`, `em` — basic inline elements
4. `code_inline`, `code_block` — code
5. `blockquote`, `list`, `li` — nested containers
6. `del` — GFM strikethrough

Deliverable: Simple HTML documents convert to correct MDAST trees. Unit tests per handler.

### Phase 4: Whitespace Normalization

**Goal**: Handle whitespace correctly — the make-or-break for real-world HTML.

Files:
- `src/hast_to_mdast/whitespace.rs`

Implement:
1. Pre-processing: collapse whitespace in HTML tree per CSS rules
2. Post-processing: merge adjacent text nodes, trim in headings/paragraphs, remove empty text nodes

Deliverable: Whitespace-heavy HTML produces clean MDAST. Whitespace fixture tests pass.

### Phase 5: Implicit Paragraphs and Block-in-Inline

**Goal**: Handle mixed content correctly.

Files:
- `src/hast_to_mdast/wrap.rs`

Implement:
1. `wrap_needed()` — detect if nodes need wrapping
2. `flatten()` / `split()` — split straddling links/deletes around block content
3. `wrap()` — group phrasing runs into paragraphs

Deliverable: `implicit-paragraphs`, `paragraph-implicit`, `straddling` fixtures pass.

### Phase 6: Remaining Element Handlers

**Goal**: Complete all 28 handlers + element categories.

Files:
- `src/hast_to_mdast/handlers.rs` (extend)

Handlers:
- `table`, `table_row`, `table_cell` — including headless, colspan/rowspan, nested-table-as-text
- `input` — checkbox/radio/image/email/url variants
- `dl` — definition list → List with dt/dd grouping
- `select`, `textarea` — form elements as text
- `iframe`, `media` (audio/video) — link fallbacks
- `q` — quote nesting
- `wbr` — zero-width space
- `comment` — HTML comment passthrough
- `base` — frozen base URL
- Ignore, pass-through, and flow-wrapper element categories

Deliverable: All 130 hast-util-to-mdast fixture inputs produce correct MDAST trees.

### Phase 7: Basic Serializer

**Goal**: MDAST → Markdown string for simple cases.

Files:
- `src/stringify/mod.rs` — `State`, `handle()` dispatch
- `src/stringify/handlers.rs` — all node type handlers
- `src/stringify/flow.rs` — `container_flow()`, join rules
- `src/stringify/phrasing.rs` — `container_phrasing()`, peek

Deliverable: MDAST trees serialize to valid Markdown. Simple fixtures produce expected output.

### Phase 8: Context-Sensitive Escaping

**Goal**: Only escape Markdown syntax when it would actually trigger formatting.

Files:
- `src/stringify/escape.rs` — `UnsafePattern`, `safe()`, pattern definitions

Implement:
1. Define all ~20 unsafe patterns
2. `safe()` function with virtual string, scope checking, backslash/char-ref escaping
3. Attention encoding for emphasis/strong boundaries

Deliverable: `fake **bold**` escapes correctly, real `**bold**` doesn't. Edge cases from JohannesKaufmann's ESCAPING.md handled.

### Phase 9: Serializer Configuration

**Goal**: All formatting options work.

Files:
- `src/stringify/mod.rs` (extend)
- `src/stringify/handlers.rs` (extend)

Options to implement: heading style (ATX/setext), bullet chars, emphasis chars, fence chars, rule style, list item indent, close ATX, quote char, increment list markers.

Deliverable: Each option produces the expected output variation.

### Phase 10: End-to-End Integration

**Goal**: Full pipeline works. Public API is clean.

Files:
- `src/lib.rs` — wire up `convert()`, `convert_with()`, `html_to_mdast()`, `mdast_to_string()`
- Copy test fixtures from `refs/hast-util-to-mdast/test/fixtures/` into `test-fixtures/`
- `tests/fixtures.rs` — run all 130 fixture tests end-to-end
- `tests/integration.rs` — API ergonomics tests

Deliverable: All 130 fixture tests pass end-to-end (HTML string in → Markdown string out).

### Phase 11: pulldown-cmark Bridge + CommonMark Round-Trip

**Goal**: Round-trip testing infrastructure.

Files:
- `src/pulldown.rs` — `events_to_mdast()` adapter (`#[cfg(test)]`)
- `tests/commonmark.rs` — 657 round-trip tests

Deliverable: CommonMark round-trip tests running. Track pass/fail rate, document expected failures.

### Phase 12: Polish + Benchmarks

**Goal**: Production-ready.

Files:
- `benches/conversion.rs` — Criterion benchmarks (per-fixture + large pages)
- Performance profiling and optimization
- API documentation
- Edge case fixes from any remaining fixture failures

Deliverable: Clean `cargo test`, `cargo clippy`, `cargo doc`. Benchmarks show competitive performance.

## Key Design Decisions

### Single crate, not workspace
Matches readability-rs, trafilatura-rs, justext-rs. Simpler to use as a dependency. Internal modules provide the same separation of concerns.

### html5ever directly, not scraper
The transformer needs to walk an HTML tree. html5ever's `RcDom` provides this directly without the overhead of scraper's CSS selector indexing. We don't need CSS selectors here — we process every node in tree order.

### MDAST as owned tree, not arena
Each `Node` owns its `Vec<Node>` children. Simpler API, no lifetimes, easy to construct in handlers. The trees are small (hundreds of nodes, not millions), so the allocation overhead is negligible.

### pulldown-cmark as dev-dependency only
The bridge is for testing. Users who want to parse Markdown can use pulldown-cmark directly. We don't impose it as a runtime dependency.

### Port structure, not code
Unlike readability-rs and trafilatura-rs (which are faithful ports of specific Go implementations), this project ports the *architecture and test cases* from the JavaScript reference implementations. The Rust code should be idiomatic Rust, not a line-by-line transliteration of JavaScript. The reference implementations inform *what* to do and *what edge cases to handle*, but the *how* should be natural Rust.

## Non-Goals (for now)

- Markdown parser (use `pulldown-cmark`)
- MDAST → HTML renderer (trivial to add later)
- GFM extensions beyond tables, strikethrough, task lists, and footnotes
- Streaming/incremental conversion
- Plugin/extension system for custom element handlers
