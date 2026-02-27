// Shared test helpers for html-to-markdown-rs.

use std::fs;
use std::path::Path;

/// Load a test fixture's input HTML and expected Markdown output.
///
/// Fixture directories contain `index.html` and `index.md`.
pub fn load_fixture(name: &str) -> (String, String) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures").join(name);
    let html = fs::read_to_string(base.join("index.html"))
        .unwrap_or_else(|_| panic!("Missing fixture: {}/index.html", name));
    let md = fs::read_to_string(base.join("index.md"))
        .unwrap_or_else(|_| panic!("Missing fixture: {}/index.md", name));
    (html, md)
}
