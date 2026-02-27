// Regression tests — every bug found becomes a test case here.
// Never delete a test from this file.

use pretty_assertions::assert_eq;

/// Image alt text containing `]` must be escaped to prevent premature bracket
/// close in the `![alt](url)` syntax.
#[test]
fn image_alt_with_bracket() {
    let md = html_to_markdown::convert(r#"<img src="foo.png" alt="a]b">"#).unwrap();
    assert_eq!(md, "![a\\]b](foo.png)\n");
}

/// Definition labels and link-reference labels are escaped via `escape_link_text`
/// to prevent `]` from prematurely closing the bracket.
/// Unit-tested in src/stringify/escape.rs (can't test via convert() because our
/// HTML→MDAST transformer never produces Definition/LinkReference nodes).
/// The fix is: `handle_definition` and `handle_link_reference` call
/// `escape_link_text(raw_label)` before formatting the output.
#[test]
fn definition_label_escaping_documented() {
    // Smoke test: a link whose text contains `]` should be escaped.
    let md = html_to_markdown::convert(r#"<a href="http://example.com">foo]bar</a>"#).unwrap();
    assert!(md.contains("foo\\]bar"), "link text ] should be escaped: {md:?}");
}

/// Image alt text containing `*` must be escaped to prevent accidental
/// emphasis in the `![alt](url)` syntax context.
#[test]
fn image_alt_with_asterisk() {
    let md = html_to_markdown::convert(r#"<img src="foo.png" alt="a*b">"#).unwrap();
    assert_eq!(md, "![a\\*b](foo.png)\n");
}

/// Double-tilde in text must be escaped to prevent accidental GFM strikethrough.
/// Only the first `~` of each `~~` pair is escaped (consistent with JS reference).
#[test]
fn double_tilde_escape_in_phrasing() {
    let md = html_to_markdown::convert("<p>foo ~~bar~~ baz</p>").unwrap();
    // First `~` of each `~~` pair is escaped; single `~` is left alone.
    assert_eq!(md, "foo \\~~bar\\~~ baz\n");
}

/// A single tilde should NOT be escaped (it's not strikethrough syntax alone).
#[test]
fn single_tilde_not_escaped() {
    let md = html_to_markdown::convert("<p>~/.bashrc</p>").unwrap();
    assert_eq!(md, "~/.bashrc\n");
}
