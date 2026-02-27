// End-to-end API tests for html-to-markdown.

use html_to_markdown::{convert, convert_with, Options, HeadingStyle};

#[test]
fn test_empty_input() {
    let result = convert("").unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_plain_text() {
    let result = convert("Hello, world!").unwrap();
    assert!(result.contains("Hello, world!"));
}

#[test]
fn test_options_are_applied() {
    let options = Options::new()
        .with_heading_style(HeadingStyle::Atx)
        .with_bullet('-');
    let result = convert_with("<h1>Title</h1>", &options).unwrap();
    assert!(result.contains("Title"));
}
