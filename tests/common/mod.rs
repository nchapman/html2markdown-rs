// Shared test helpers for html-to-markdown-rs.

use std::fs;
use std::path::Path;

use html_to_markdown::Options;

/// Fixture options parsed from `index.json`.
pub struct FixtureOptions {
    pub html: String,
    pub expected_md: String,
    pub options: Options,
    pub fragment: bool,
}

/// Load a test fixture's input HTML, expected Markdown, and options.
///
/// Fixture directories contain `index.html`, `index.md`, and optionally `index.json`.
pub fn load_fixture(name: &str) -> FixtureOptions {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures").join(name);
    let html = fs::read_to_string(base.join("index.html"))
        .unwrap_or_else(|_| panic!("Missing fixture: {}/index.html", name));
    let md = fs::read_to_string(base.join("index.md"))
        .unwrap_or_else(|_| panic!("Missing fixture: {}/index.md", name));

    let mut options = Options::default();
    let mut fragment = false;

    // Parse index.json if it exists.
    if let Ok(json_str) = fs::read_to_string(base.join("index.json")) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(b) = val.get("fragment").and_then(|v| v.as_bool()) {
                fragment = b;
            }
            if let Some(s) = val.get("checked").and_then(|v| v.as_str()) {
                options.checked = Some(s.to_string());
            }
            if let Some(s) = val.get("unchecked").and_then(|v| v.as_str()) {
                options.unchecked = Some(s.to_string());
            }
            if let Some(b) = val.get("newlines").and_then(|v| v.as_bool()) {
                options.newlines = b;
            }
            if let Some(arr) = val.get("quotes").and_then(|v| v.as_array()) {
                options.quotes = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
    }

    FixtureOptions { html, expected_md: md, options, fragment }
}
