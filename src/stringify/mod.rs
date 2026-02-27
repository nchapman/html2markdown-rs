// MDAST → Markdown string serializer.
//
// Port of mdast-util-to-markdown (https://github.com/syntax-tree/mdast-util-to-markdown).
// Walks an MDAST tree and emits a Markdown string. All formatting choices
// (heading style, list markers, emphasis characters, etc.) live here.

pub(crate) mod escape;
pub(crate) mod flow;
pub(crate) mod handlers;
pub(crate) mod phrasing;

use crate::mdast::Node;

/// Heading style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingStyle {
    /// `# Heading` (default).
    #[default]
    Atx,
    /// Only for h1/h2; falls back to ATX for h3–h6.
    Setext,
}

/// List item indentation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListItemIndent {
    /// Bullet + 1 space (default).
    #[default]
    One,
    /// Round up to 4 columns.
    Tab,
    /// One for tight lists, tab for spread.
    Mixed,
}

/// Serializer configuration.
#[derive(Debug, Clone)]
pub struct StringifyOptions {
    pub heading_style: HeadingStyle,
    pub bullet: char,
    pub bullet_ordered: char,
    pub emphasis: char,
    pub strong: char,
    pub fence: char,
    pub rule: char,
    pub rule_repetition: u8,
    pub rule_spaces: bool,
    pub close_atx: bool,
    pub list_item_indent: ListItemIndent,
    pub increment_list_marker: bool,
    pub quote: char,
    pub fences: bool,
    pub resource_link: bool,
}

impl Default for StringifyOptions {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::Atx,
            bullet: '*',
            bullet_ordered: '.',
            emphasis: '*',
            strong: '*',
            fence: '`',
            rule: '*',
            rule_repetition: 3,
            rule_spaces: false,
            close_atx: false,
            list_item_indent: ListItemIndent::One,
            increment_list_marker: true,
            quote: '"',
            fences: true,
            resource_link: false,
        }
    }
}

/// Serializer state threaded through all handlers.
pub(crate) struct State<'a> {
    pub options: &'a StringifyOptions,
    /// Current list bullet (may switch to avoid conflicts).
    pub bullet_current: Option<char>,
    /// Previous list's bullet (for alternation).
    pub bullet_last_used: Option<char>,
    /// Whether the next text to be emitted is at the start of a block (atBreak).
    /// Used to apply at-break character escaping (e.g. `+` before space → `\+`).
    pub at_break: bool,
}

impl<'a> State<'a> {
    pub fn new(options: &'a StringifyOptions) -> Self {
        Self {
            options,
            bullet_current: None,
            bullet_last_used: None,
            at_break: false,
        }
    }
}

/// Serialize an MDAST tree to a Markdown string.
pub(crate) fn stringify(node: &Node, options: &StringifyOptions) -> String {
    let mut state = State::new(options);
    let mut output = handlers::handle(&mut state, node);

    // Ensure trailing newline (only if non-empty).
    // Port of mdast-util-to-markdown: `result && !result.endsWith('\n') → result += '\n'`
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    output
}
