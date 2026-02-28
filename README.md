# html2markdown

[![Crates.io](https://img.shields.io/crates/v/html2markdown.svg)](https://crates.io/crates/html2markdown)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

HTML to Markdown converter using AST-to-AST transformation.

Ports the architecture and test cases from
[hast-util-to-mdast](https://github.com/syntax-tree/hast-util-to-mdast) (transformer) and
[mdast-util-to-markdown](https://github.com/syntax-tree/mdast-util-to-markdown) (serializer).

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
html2markdown = "0.2"
```

```rust
let md = html2markdown::convert("<h1>Hello</h1><p>World</p>");
assert_eq!(md, "# Hello\n\nWorld");
```

### With options

```rust
use html2markdown::{convert_with, Options, HeadingStyle};

let opts = Options::new().with_heading_style(HeadingStyle::Setext);

let md = convert_with("<h1>Hello</h1><p>World</p>", &opts);
assert_eq!(md, "Hello\n=====\n\nWorld");
```

## What it handles

- Headings, paragraphs, blockquotes, lists (ordered, unordered, task lists)
- Inline formatting: bold, italic, strikethrough, code
- Links, images, and reference-style links
- Tables (with alignment)
- Code blocks (fenced) with language hints
- Horizontal rules, line breaks
- Nested structures and edge cases from 130 fixture tests
- Context-sensitive escaping to prevent false Markdown syntax

## Architecture

The conversion is a two-phase pipeline:

1. **HTML tree -> MDAST** — html5ever parses the HTML into a DOM, then element
   handlers transform each node into typed Markdown AST nodes. Whitespace is
   normalized during this phase.

2. **MDAST -> Markdown string** — the AST is serialized with configurable
   formatting (heading style, bullet character, list indent, emphasis marker)
   and context-sensitive escaping.

The two phases are independent: the transformer knows nothing about string
formatting, and the serializer knows nothing about HTML.

## Optional features

| Feature | Description |
|---------|-------------|
| `tracing` | Enable debug/trace logging (zero-cost when disabled) |

```toml
html2markdown = { version = "0.2", features = ["tracing"] }
```

## Benchmarks

Throughput comparison (MiB/s, higher is better):

| Fixture | Rust | html2md (Rust) | Go | hast (JS) | turndown (JS) |
|---------|-----:|---------------:|---:|----------:|--------------:|
| article | 68.5 | 58.3 | 29.3 | 4.1 | 15.8 |
| table | 21.1 | 17.9 | 21.7 | 1.8 | ERR |
| lists | 19.8 | 21.3 | 18.0 | 1.7 | 4.6 |
| code | 62.4 | 55.7 | 43.2 | 5.2 | 15.2 |
| large | 48.7 | 43.3 | 28.8 | 3.1 | ERR |

Measured on Apple M4 Max, Rust 1.93, Go 1.25, Node 22, macOS 15.7.

Reproduce:

```sh
cargo bench                  # Criterion benchmarks
./benches/compare.sh         # full cross-language comparison table
```

## License

MIT
