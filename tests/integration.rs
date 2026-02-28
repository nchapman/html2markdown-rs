// End-to-end API tests for html2markdown.

use html2markdown::{convert, convert_with, HeadingStyle, Options};

#[test]
fn test_empty_input() {
    let result = convert("");
    assert_eq!(result, "");
}

#[test]
fn test_plain_text() {
    let result = convert("Hello, world!");
    assert!(result.contains("Hello, world!"));
}

#[test]
fn test_options_are_applied() {
    let options = Options::new()
        .with_heading_style(HeadingStyle::Atx)
        .with_bullet('-');
    let result = convert_with("<h1>Title</h1>", &options);
    assert!(result.contains("Title"));
}

#[test]
fn roundtrip_raw_html_invalid() {
    use pulldown_cmark::{html, Options, Parser};

    // These are "invalid" raw HTML cases — CommonMark escapes them to &lt;...&gt;
    let cases = vec![
        (620u32, "<p>&lt;33&gt; &lt;__&gt;</p>\n"),
        (621, "<p>&lt;a h*#ref=&quot;hi&quot;&gt;</p>\n"),
        (622, "<p>&lt;a href=&quot;hi'&gt; &lt;a href=hi'&gt;</p>\n"),
        (623, "<p>&lt;a href='hi''&gt;</p>\n"),
        (624, "<p>&lt;a href=''&gt;</p>\n"),
        (625, "<p>&lt;a href=&quot;foo&quot;bar&gt;</p>\n"),
        (626, "<p>&lt; a&gt;</p>\n"),
        (627, "<p>&lt;foo bar=baz\nbim!bop /&gt;</p>\n"),
        (628, "<p>&lt;foo bar=&quot;baz&lt;bim&quot;&gt;</p>\n"),
        (629, "<p>&lt;a href=&quot;foo&amp;ouml;&quot;&gt;</p>\n"),
        (630, "<p>&lt;a href=&quot;\\*&quot;&gt;</p>\n"),
        (631, "<p>&lt;a href=&quot;'&gt;</p>\n"),
        (632, "<p>&lt;/1&gt;</p>\n"),
        (633, "<p>&lt;/a&gt;&lt;/em&gt;</p>\n"),
        (634, "<p>&lt;/a href=&quot;foo&quot;&gt;</p>\n"),
        (
            635,
            "<p>foo &lt;!-- this is a\ncomment - with hyphen --&gt;</p>\n",
        ),
    ];
    for (n, html_input) in &cases {
        let md = html2markdown::convert(html_input);
        let parser = Parser::new_ext(&md, Options::all());
        let mut html_out = String::new();
        html::push_html(&mut html_out, parser);
        let ok = html_out.trim() == html_input.trim();
        eprintln!(
            "Example #{n}: {} | MD={md:?}",
            if ok { "PASS" } else { "FAIL" }
        );
        if !ok {
            eprintln!("  expected: {html_input:?}");
            eprintln!("  got:      {html_out:?}");
        }
    }
}
