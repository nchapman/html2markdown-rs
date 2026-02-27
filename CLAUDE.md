# html-to-markdown-rs

HTML-to-Markdown converter using AST-to-AST transformation. Ports the architecture and test cases from [hast-util-to-mdast](https://github.com/syntax-tree/hast-util-to-mdast) (transformer) and [mdast-util-to-markdown](https://github.com/syntax-tree/mdast-util-to-markdown) (serializer).

## Source References

- **Transformer reference**: `../refs/hast-util-to-mdast/` — handlers, fixtures, state machine
- **Serializer reference**: `../refs/mdast-util-to-markdown/` — formatting, escaping, join rules
- **MDAST spec**: `../refs/mdast/` — node type definitions
- **Escaping reference**: `../refs/html-to-markdown/ESCAPING.md` — edge case documentation
- **CommonMark spec**: `../refs/commonmark-spec/spec.txt` — 657 round-trip test examples
- **Implementation plan**: `PLAN.md` in this repo

## Porting Philosophy

Unlike readability-rs and trafilatura-rs (faithful line-by-line ports of Go), this project ports the **architecture and test cases** from JavaScript reference implementations. The Rust code should be idiomatic Rust, not a transliteration. The reference implementations inform *what* to do and *what edge cases to handle*, but the *how* should be natural Rust.

### Key principles

- **Study the reference, write Rust**: Read the JS handler, understand the semantics, implement in idiomatic Rust
- **Port the tests faithfully**: The 130 fixture test cases define correctness — same inputs, same expected outputs
- **Two-phase architecture**: Transformer produces MDAST nodes (no strings). Serializer formats MDAST to Markdown (no HTML knowledge). Keep them independent.
- **Comment what you port**: Note which JS file/function a Rust function corresponds to

### Write idiomatic Rust

- Use `Result<T, E>` and `?` operator
- Use `Option<T>` instead of null checks
- Use iterators and combinators where clearer than loops
- Use `&str` for borrowed strings, `String` for owned
- Use `std::sync::LazyLock` for compiled regex patterns (stable since Rust 1.80, do not use `once_cell`)
- Derive `Debug`, `Clone`, `Default` on public types where appropriate
- Use `thiserror` for error types
- No `unwrap()` in library code (only in tests and static regex compilation inside `LazyLock`)

### What NOT to do

- Do not transliterate JavaScript line-by-line — understand the semantics and write Rust
- Do not add features beyond what the reference implementations support
- Do not use `async` — conversion is synchronous
- Do not add `serde` as a runtime dependency — it's only for test fixture loading
- Do not use `scraper` — html5ever's `RcDom` is sufficient and lighter weight

## Workflow

### Cycle for each section of work

1. **Read the JS source** for the handler/module you're implementing
2. **Understand the semantics** — what does this handler do? what edge cases does it handle?
3. **Implement** the Rust equivalent
4. **Write tests** — port corresponding fixture tests, add edge cases
5. **Run `cargo test` and `cargo clippy`** — fix all warnings
6. **Request a code review** (use the `code-reviewer` agent)
7. **Fix review findings**, re-run tests
8. **Commit** with a clean, descriptive message

### Commit discipline

- Commit after completing each coherent piece of work
- Do **not** reference plan phases or milestones in commit messages
- Write clear, specific descriptions:
  - Good: `Implement heading and paragraph handlers with fixtures`
  - Good: `Add context-sensitive escaping for emphasis markers`
  - Good: `Port table handler with colspan/rowspan support`
  - Bad: `Phase 3 complete`
  - Bad: `WIP`
- Use imperative mood: "Add", "Implement", "Fix", "Port"
- Include a brief bullet list when the commit touches multiple concerns

### Testing standards

- **Port fixture tests faithfully**. The 130 fixture directories (`test-fixtures/`) define expected behavior.
- **Use `pretty_assertions`** for string comparison — the diff output is essential.
- **Test each handler in isolation** before integration.
- **Round-trip test with pulldown-cmark**: convert HTML → Markdown → HTML, compare semantically.
- **Every bug becomes a regression test** in `tests/regression.rs`. Never delete one.
- Run `cargo test` after every change.

## Architecture

```
HTML string
    │
    ▼
html5ever::parse_document()        — src/hast_to_mdast/mod.rs
    │
    ▼
html5ever RcDom (HTML tree)
    │
    ▼
State::one() / State::all()        — src/hast_to_mdast/handlers.rs
    │                                 src/hast_to_mdast/wrap.rs
    │                                 src/hast_to_mdast/whitespace.rs
    ▼
MDAST (Node enum)                  — src/mdast.rs
    │
    ▼
stringify::handle()                — src/stringify/handlers.rs
    │                                 src/stringify/escape.rs
    │                                 src/stringify/flow.rs
    │                                 src/stringify/phrasing.rs
    ▼
Markdown string
```

## Key crate choices

| Crate | Purpose | Why |
|-------|---------|-----|
| `html5ever` | HTML parsing | Spec-compliant HTML5, used by Servo |
| `markup5ever` | DOM types | QualName, LocalName — html5ever companion |
| `regex` | Escaping patterns | Fast, linear time |
| `url` | URL resolution | `<base>` support |
| `thiserror` | Error types | Derive Error implementations |
| `tracing` | Logging (optional) | Zero-cost when disabled |
| `pretty_assertions` | Test diffs (dev) | Readable assertion failures |
| `pulldown-cmark` | Round-trip testing (dev) | Markdown → HTML for validation |
| `criterion` | Benchmarks (dev) | Standard Rust benchmark framework |

## Commands

```bash
cargo test                    # Run all tests
cargo clippy                  # Lint
cargo fmt --check             # Format check
cargo bench                   # Run benchmarks
```

## File layout

```
src/
├── lib.rs                    # Public API: convert(), convert_with(), Options
├── error.rs                  # HtmlToMarkdownError enum
├── mdast.rs                  # MDAST node types (Node enum + structs)
├── hast_to_mdast/            # HTML tree → MDAST transform
│   ├── mod.rs                # State, parse, transform entry point
│   ├── handlers.rs           # Element handlers (dispatch + all handlers)
│   ├── wrap.rs               # Implicit paragraph detection, block-in-inline
│   └── whitespace.rs         # Whitespace normalization
└── stringify/                # MDAST → Markdown string
    ├── mod.rs                # State, stringify entry point, options
    ├── handlers.rs           # Node type handlers
    ├── escape.rs             # Context-sensitive escaping
    ├── flow.rs               # Block-level serialization
    └── phrasing.rs           # Inline serialization

tests/
├── common/mod.rs             # Shared test helpers
├── fixtures.rs               # 130 fixture tests (from hast-util-to-mdast)
├── commonmark.rs             # 657 CommonMark round-trip tests
├── regression.rs             # Bug regression tests
└── integration.rs            # End-to-end API tests

test-fixtures/                # Copied from refs/hast-util-to-mdast/test/fixtures/
benches/
└── conversion.rs             # Criterion benchmarks
```
