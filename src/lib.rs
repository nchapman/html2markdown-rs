// html-to-markdown — HTML to Markdown converter using AST-to-AST transformation.
//
// Architecture:
//   HTML string → html5ever parse → HTML tree → hast_to_mdast → MDAST → stringify → Markdown
//
// Reference implementations:
//   - hast-util-to-mdast (transformer): https://github.com/syntax-tree/hast-util-to-mdast
//   - mdast-util-to-markdown (serializer): https://github.com/syntax-tree/mdast-util-to-markdown

mod error;
pub mod mdast;
mod hast_to_mdast;
mod stringify;

pub use error::HtmlToMarkdownError;
pub use stringify::{HeadingStyle, ListItemIndent, StringifyOptions};

/// Conversion options.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Serializer formatting options.
    pub stringify: StringifyOptions,
    /// Whether to preserve newlines in whitespace normalization.
    pub newlines: bool,
    /// Symbol for checked checkboxes/radio buttons. Default: `"[x]"`.
    pub checked: Option<String>,
    /// Symbol for unchecked checkboxes/radio buttons. Default: `"[ ]"`.
    pub unchecked: Option<String>,
    /// Quote character pairs for `<q>` elements, cycling by nesting depth.
    /// Each entry is 1 or 2 chars: open (and optionally close).
    /// Default: `['"']` (plain ASCII double-quote for both open and close).
    pub quotes: Vec<String>,
}

impl Options {
    /// Create a new Options with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the heading style.
    pub fn with_heading_style(mut self, style: HeadingStyle) -> Self {
        self.stringify.heading_style = style;
        self
    }

    /// Set the unordered list bullet character.
    pub fn with_bullet(mut self, bullet: char) -> Self {
        self.stringify.bullet = bullet;
        self
    }

    /// Set the ordered list bullet character.
    pub fn with_bullet_ordered(mut self, bullet: char) -> Self {
        self.stringify.bullet_ordered = bullet;
        self
    }

    /// Set the emphasis marker character.
    pub fn with_emphasis(mut self, marker: char) -> Self {
        self.stringify.emphasis = marker;
        self
    }

    /// Set the strong marker character.
    pub fn with_strong(mut self, marker: char) -> Self {
        self.stringify.strong = marker;
        self
    }

    /// Set the fenced code block marker character.
    pub fn with_fence(mut self, fence: char) -> Self {
        self.stringify.fence = fence;
        self
    }

    /// Set the thematic break rule character.
    pub fn with_rule(mut self, rule: char) -> Self {
        self.stringify.rule = rule;
        self
    }

    /// Set the number of thematic break markers.
    pub fn with_rule_repetition(mut self, count: u8) -> Self {
        self.stringify.rule_repetition = count;
        self
    }

    /// Set whether to use spaces in thematic breaks.
    pub fn with_rule_spaces(mut self, spaces: bool) -> Self {
        self.stringify.rule_spaces = spaces;
        self
    }

    /// Set whether to close ATX headings with trailing hashes.
    pub fn with_close_atx(mut self, close: bool) -> Self {
        self.stringify.close_atx = close;
        self
    }

    /// Set the list item indentation style.
    pub fn with_list_item_indent(mut self, indent: ListItemIndent) -> Self {
        self.stringify.list_item_indent = indent;
        self
    }

    /// Set whether to increment ordered list markers.
    pub fn with_increment_list_marker(mut self, increment: bool) -> Self {
        self.stringify.increment_list_marker = increment;
        self
    }

    /// Set the quote character for titles.
    pub fn with_quote(mut self, quote: char) -> Self {
        self.stringify.quote = quote;
        self
    }

    /// Set whether to always use fenced code blocks.
    pub fn with_fences(mut self, fences: bool) -> Self {
        self.stringify.fences = fences;
        self
    }

    /// Set whether to always use resource links (never autolinks).
    pub fn with_resource_link(mut self, resource: bool) -> Self {
        self.stringify.resource_link = resource;
        self
    }

    /// Set whether to preserve newlines in whitespace normalization.
    pub fn with_newlines(mut self, newlines: bool) -> Self {
        self.newlines = newlines;
        self
    }
}

/// Convert an HTML string to Markdown using default options.
///
/// # Examples
///
/// ```
/// let md = html_to_markdown::convert("<h1>Hello</h1><p>World</p>").unwrap();
/// assert!(md.contains("Hello"));
/// ```
pub fn convert(html: &str) -> Result<String, HtmlToMarkdownError> {
    convert_with(html, &Options::default())
}

/// Convert an HTML string to Markdown with custom options.
///
/// # Examples
///
/// ```
/// use html_to_markdown::{convert_with, Options, HeadingStyle};
///
/// let options = Options::new().with_heading_style(HeadingStyle::Setext);
/// let md = convert_with("<h1>Hello</h1>", &options).unwrap();
/// assert!(md.contains("Hello"));
/// ```
pub fn convert_with(html: &str, options: &Options) -> Result<String, HtmlToMarkdownError> {
    let mdast = html_to_mdast(html, options)?;
    Ok(mdast_to_string(&mdast, &options.stringify))
}

/// Parse HTML and transform it into an MDAST tree.
pub fn html_to_mdast(
    html: &str,
    options: &Options,
) -> Result<mdast::Node, HtmlToMarkdownError> {
    let transform_options = hast_to_mdast::TransformOptions {
        newlines: options.newlines,
        checked: options.checked.clone(),
        unchecked: options.unchecked.clone(),
        quotes: options.quotes.clone(),
    };
    Ok(hast_to_mdast::transform(html, transform_options))
}

/// Serialize an MDAST tree to a Markdown string.
pub fn mdast_to_string(node: &mdast::Node, options: &StringifyOptions) -> String {
    stringify::stringify(node, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_empty() {
        let result = convert("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_simple_paragraph() {
        let result = convert("<p>Hello, world!</p>").unwrap();
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn test_convert_heading() {
        let result = convert("<h1>Title</h1>").unwrap();
        assert!(result.contains("Title"));
    }

    #[test]
    fn test_options_builder() {
        let options = Options::new()
            .with_heading_style(HeadingStyle::Setext)
            .with_bullet('-')
            .with_emphasis('_')
            .with_fence('~');

        assert_eq!(options.stringify.heading_style, HeadingStyle::Setext);
        assert_eq!(options.stringify.bullet, '-');
        assert_eq!(options.stringify.emphasis, '_');
        assert_eq!(options.stringify.fence, '~');
    }

    #[test]
    fn test_default_options() {
        let options = Options::default();
        assert_eq!(options.stringify.heading_style, HeadingStyle::Atx);
        assert_eq!(options.stringify.bullet, '*');
        assert_eq!(options.stringify.emphasis, '*');
        assert_eq!(options.stringify.fence, '`');
        assert!(!options.newlines);
    }
}
