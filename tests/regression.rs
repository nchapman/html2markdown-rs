// Regression tests — every bug found becomes a test case here.
// Never delete a test from this file.

use pretty_assertions::assert_eq;

/// Image alt text containing `]` must be escaped to prevent premature bracket
/// close in the `![alt](url)` syntax.
#[test]
fn image_alt_with_bracket() {
    let md = html2markdown::convert(r#"<img src="foo.png" alt="a]b">"#);
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
    let md = html2markdown::convert(r#"<a href="http://example.com">foo]bar</a>"#);
    assert!(
        md.contains("foo\\]bar"),
        "link text ] should be escaped: {md:?}"
    );
}

/// Image alt text containing `*` must be escaped to prevent accidental
/// emphasis in the `![alt](url)` syntax context.
#[test]
fn image_alt_with_asterisk() {
    let md = html2markdown::convert(r#"<img src="foo.png" alt="a*b">"#);
    assert_eq!(md, "![a\\*b](foo.png)\n");
}

/// Double-tilde in text must be escaped to prevent accidental GFM strikethrough.
/// Only the first `~` of each `~~` pair is escaped (consistent with JS reference).
#[test]
fn double_tilde_escape_in_phrasing() {
    let md = html2markdown::convert("<p>foo ~~bar~~ baz</p>");
    // First `~` of each `~~` pair is escaped; single `~` is left alone.
    assert_eq!(md, "foo \\~~bar\\~~ baz\n");
}

/// A single tilde should NOT be escaped (it's not strikethrough syntax alone).
#[test]
fn single_tilde_not_escaped() {
    let md = html2markdown::convert("<p>~/.bashrc</p>");
    assert_eq!(md, "~/.bashrc\n");
}

/// Pipe characters in table cells must be escaped to prevent breaking table structure.
#[test]
fn pipe_in_table_cell_escaped() {
    let md =
        html2markdown::convert("<table><tr><th>Header</th></tr><tr><td>a|b</td></tr></table>");
    assert!(
        md.contains("a\\|b"),
        "pipe in table cell should be escaped: {md:?}"
    );
}

/// Pipe escaping should not apply outside of tables.
#[test]
fn pipe_not_escaped_outside_table() {
    let md = html2markdown::convert("<p>a|b</p>");
    assert_eq!(md, "a|b\n");
}

/// Newlines inside inline code must be replaced with spaces to prevent
/// block constructs from triggering when the markdown is re-parsed.
#[test]
fn inline_code_newline_replaced_with_space() {
    // Build MDAST directly since html5ever normalizes newlines in <code>.
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Paragraph(Paragraph {
            children: vec![Node::InlineCode(InlineCode {
                value: "foo\nbar".to_string(),
            })],
        })],
    });
    let md = html2markdown::mdast_to_string(&node, &StringifyOptions::default());
    assert_eq!(md, "`foo bar`\n");
}

/// Code fence info string with backtick must switch to tilde fences.
#[test]
fn code_fence_lang_with_backtick() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Code(Code {
            lang: Some("a`b".to_string()),
            meta: None,
            value: "code".to_string(),
        })],
    });
    let md = html2markdown::mdast_to_string(&node, &StringifyOptions::default());
    assert!(
        md.starts_with("~~~a`b\n"),
        "should use tilde fence when lang has backtick: {md:?}"
    );
}

/// Code fence lang space should be encoded as &#x20; when meta is present.
#[test]
fn code_fence_lang_space_with_meta() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Code(Code {
            lang: Some("a b".to_string()),
            meta: Some("meta".to_string()),
            value: "code".to_string(),
        })],
    });
    let md = html2markdown::mdast_to_string(&node, &StringifyOptions::default());
    assert!(
        md.starts_with("```a&#x20;b meta\n"),
        "lang space should be encoded when meta present: {md:?}"
    );
}

/// Code fence meta with newline should have newline replaced with space.
#[test]
fn code_fence_meta_newline() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Code(Code {
            lang: Some("js".to_string()),
            meta: Some("a\nb".to_string()),
            value: "code".to_string(),
        })],
    });
    let md = html2markdown::mdast_to_string(&node, &StringifyOptions::default());
    assert!(
        md.starts_with("```js a b\n"),
        "meta newline should become space: {md:?}"
    );
}

/// ATX heading with leading space should encode it as &#x20;.
#[test]
fn atx_heading_leading_space() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Heading(Heading {
            depth: 1,
            children: vec![Node::Text(Text {
                value: " foo".to_string(),
            })],
        })],
    });
    let md = html2markdown::mdast_to_string(&node, &StringifyOptions::default());
    assert_eq!(md, "# &#x20;foo\n");
}

/// Text starting with `1. ` at block start must escape the `.` to prevent
/// ordered list interpretation.
#[test]
fn ordered_list_marker_escaped_dot() {
    let md = html2markdown::convert("<p>1. foo</p>");
    assert_eq!(md, "1\\. foo\n");
}

/// Text starting with `1) ` at block start must escape the `)`.
#[test]
fn ordered_list_marker_escaped_paren() {
    let md = html2markdown::convert("<p>1) foo</p>");
    assert_eq!(md, "1\\) foo\n");
}

/// Multi-digit ordered list markers should also be escaped.
#[test]
fn ordered_list_marker_multi_digit() {
    let md = html2markdown::convert("<p>10. foo</p>");
    assert_eq!(md, "10\\. foo\n");
}

/// Indented code blocks when fences option is false.
#[test]
fn indented_code_when_fences_false() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Code(Code {
            lang: None,
            meta: None,
            value: "hello\nworld".to_string(),
        })],
    });
    let opts = StringifyOptions {
        fences: false,
        ..Default::default()
    };
    let md = html2markdown::mdast_to_string(&node, &opts);
    assert_eq!(md, "    hello\n    world\n");
}

/// Even with fences:false, code with a lang should still use fences.
#[test]
fn fenced_code_with_lang_even_when_fences_false() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Code(Code {
            lang: Some("js".to_string()),
            meta: None,
            value: "x".to_string(),
        })],
    });
    let opts = StringifyOptions {
        fences: false,
        ..Default::default()
    };
    let md = html2markdown::mdast_to_string(&node, &opts);
    assert!(
        md.starts_with("```js\n"),
        "code with lang should use fences even when fences:false: {md:?}"
    );
}

/// Tab indent style rounds up to 4-column tab stops.
#[test]
fn list_item_indent_tab() {
    use html2markdown::{convert_with, ListItemIndent, Options};
    let options = Options::new().with_list_item_indent(ListItemIndent::Tab);
    let md = convert_with("<ul><li>item</li></ul>", &options);
    // `* ` is 2 chars, tab stop rounds to 4, so 2 extra spaces of padding.
    assert_eq!(md, "*   item\n");
}

/// Mixed indent: tight items use 1-space, spread items use tab.
#[test]
fn list_item_indent_mixed() {
    use html2markdown::{convert_with, ListItemIndent, Options};
    let options = Options::new().with_list_item_indent(ListItemIndent::Mixed);
    // Single-child tight item → one space
    let md = convert_with("<ul><li>item</li></ul>", &options);
    assert_eq!(md, "* item\n");
}

/// Mixed indent: spread list items use tab-width indent.
#[test]
fn list_item_indent_mixed_spread() {
    use html2markdown::{convert_with, ListItemIndent, Options};
    let options = Options::new().with_list_item_indent(ListItemIndent::Mixed);
    // A spread list item (paragraph + paragraph) → tab indent
    let md = convert_with(
        "<ul><li><p>first</p><p>second</p></li></ul>",
        &options,
    );
    assert!(
        md.starts_with("*   first"),
        "spread item should use tab indent: {md:?}"
    );
}

/// `]` at end of one phrasing node and `(` at start of the next must be
/// escaped to prevent accidental link syntax.
#[test]
fn bracket_before_paren_cross_node_escaped() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Paragraph(Paragraph {
            children: vec![
                Node::Text(Text {
                    value: "foo]".to_string(),
                }),
                Node::Text(Text {
                    value: "(bar)".to_string(),
                }),
            ],
        })],
    });
    let md = html2markdown::mdast_to_string(&node, &StringifyOptions::default());
    assert!(
        md.contains("foo\\](bar)"),
        "] before ( should be escaped across nodes: {md:?}"
    );
}

/// `<` followed by a letter at block start must be escaped to prevent HTML block.
#[test]
fn less_than_before_tag_escaped_at_break() {
    let md = html2markdown::convert("<p>&lt;div&gt; text</p>");
    assert!(
        md.starts_with("\\<div>"),
        "< before tag name should be escaped at break: {md:?}"
    );
}

/// Empty code value with fences:false should fall back to fenced code.
#[test]
fn indented_code_empty_value_falls_back_to_fenced() {
    use html2markdown::mdast::*;
    use html2markdown::StringifyOptions;
    let node = Node::Root(Root {
        children: vec![Node::Code(Code {
            lang: None,
            meta: None,
            value: String::new(),
        })],
    });
    let opts = StringifyOptions {
        fences: false,
        ..Default::default()
    };
    let md = html2markdown::mdast_to_string(&node, &opts);
    assert!(
        md.contains("```"),
        "empty code with fences:false should use fenced block: {md:?}"
    );
}

/// Deeply nested HTML should not cause a stack overflow.
#[test]
fn deep_nesting_no_stack_overflow() {
    // 3000 nested divs — well beyond the depth limit. Must not panic.
    let html = "<div>".repeat(3000) + "deep text" + &"</div>".repeat(3000);
    let _ = html2markdown::convert(&html);

    // Text at shallow depth (within limit) must still be converted.
    let shallow = "<div>".repeat(100) + "shallow text" + &"</div>".repeat(100);
    let md = html2markdown::convert(&shallow);
    assert!(
        md.contains("shallow text"),
        "shallow content should survive depth limit: {md:?}"
    );
}
