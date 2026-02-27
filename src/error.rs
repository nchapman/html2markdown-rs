/// Errors that can occur during HTML-to-Markdown conversion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HtmlToMarkdownError {
    #[error("HTML parse error: {0}")]
    Parse(String),
}
