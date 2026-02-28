uniffi::setup_scaffolding!();

/// Errors returned by `convert_with` when options contain invalid values.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum OptionsError {
    #[error("{field} must be one of {allowed}, got \"{value}\"")]
    InvalidOption {
        field: String,
        allowed: String,
        value: String,
    },
}

/// Heading style for Markdown output.
#[derive(uniffi::Enum)]
pub enum HeadingStyle {
    /// `# Heading` (default).
    Atx,
    /// Underline style — only for h1/h2; falls back to ATX for h3–h6.
    Setext,
}

/// List item indentation style.
#[derive(uniffi::Enum)]
pub enum ListItemIndent {
    /// Bullet + 1 space (default).
    One,
    /// Round up to 4 columns.
    Tab,
    /// One for tight lists, tab for spread.
    Mixed,
}

/// Serializer formatting options.
///
/// Character fields (`bullet`, `emphasis`, etc.) are represented as single-character
/// strings because UniFFI does not support Rust's `char` type.
#[derive(uniffi::Record)]
pub struct StringifyOptions {
    pub heading_style: HeadingStyle,
    /// Unordered list marker: `"*"`, `"-"`, or `"+"`.
    pub bullet: String,
    /// Ordered list marker: `"."` or `")"`.
    pub bullet_ordered: String,
    /// Emphasis marker: `"*"` or `"_"`.
    pub emphasis: String,
    /// Strong marker: `"*"` or `"_"`.
    pub strong: String,
    /// Code fence character: `` "`" `` or `"~"`.
    pub fence: String,
    /// Thematic break character: `"*"`, `"-"`, or `"_"`.
    pub rule: String,
    /// Number of thematic break markers (minimum 3).
    pub rule_repetition: u8,
    /// Whether to add spaces in thematic breaks.
    pub rule_spaces: bool,
    /// Whether to close ATX headings with trailing hashes.
    pub close_atx: bool,
    pub list_item_indent: ListItemIndent,
    /// Whether to increment ordered list markers.
    pub increment_list_marker: bool,
    /// Quote character for link titles: `"\""` or `"'"`.
    pub quote: String,
    /// Whether to always use fenced code blocks.
    pub fences: bool,
    /// Whether to always use resource links (never autolinks).
    pub resource_link: bool,
}

/// Conversion options.
#[derive(uniffi::Record)]
pub struct Options {
    /// Serializer formatting options.
    pub stringify: StringifyOptions,
    /// Whether to preserve newlines in whitespace normalization.
    pub newlines: bool,
    /// Symbol for checked checkboxes. Pass `None` for the default `"[x]"`.
    pub checked: Option<String>,
    /// Symbol for unchecked checkboxes. Pass `None` for the default `"[ ]"`.
    pub unchecked: Option<String>,
    /// Quote character pairs for `<q>` elements, cycling by nesting depth.
    pub quotes: Vec<String>,
}

/// Returns the default stringify options.
#[uniffi::export]
pub fn default_stringify_options() -> StringifyOptions {
    let d = html2markdown::StringifyOptions::default();
    StringifyOptions {
        heading_style: convert_heading_style(d.heading_style),
        bullet: d.bullet.to_string(),
        bullet_ordered: d.bullet_ordered.to_string(),
        emphasis: d.emphasis.to_string(),
        strong: d.strong.to_string(),
        fence: d.fence.to_string(),
        rule: d.rule.to_string(),
        rule_repetition: d.rule_repetition,
        rule_spaces: d.rule_spaces,
        close_atx: d.close_atx,
        list_item_indent: convert_list_item_indent(d.list_item_indent),
        increment_list_marker: d.increment_list_marker,
        quote: d.quote.to_string(),
        fences: d.fences,
        resource_link: d.resource_link,
    }
}

/// Returns the default conversion options.
#[uniffi::export]
pub fn default_options() -> Options {
    let d = html2markdown::Options::default();
    Options {
        stringify: default_stringify_options(),
        newlines: d.newlines,
        checked: d.checked,
        unchecked: d.unchecked,
        quotes: d.quotes,
    }
}

/// Convert an HTML string to Markdown using default options.
#[uniffi::export]
pub fn convert(html: String) -> String {
    html2markdown::convert(&html)
}

/// Convert an HTML string to Markdown with custom options.
///
/// Returns an `OptionsError` if any option field contains an invalid value.
#[uniffi::export]
pub fn convert_with(html: String, options: Options) -> Result<String, OptionsError> {
    let core_options = to_core_options(options)?;
    Ok(html2markdown::convert_with(&html, &core_options))
}

// --- Internal conversion helpers ---

fn parse_char(s: &str, field: &str, allowed: &[char]) -> Result<char, OptionsError> {
    let mut chars = s.chars();
    let c = match chars.next() {
        Some(c) if chars.next().is_none() && allowed.contains(&c) => c,
        _ => {
            return Err(OptionsError::InvalidOption {
                field: field.to_string(),
                allowed: allowed.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(", "),
                value: s.to_string(),
            });
        }
    };
    Ok(c)
}

fn convert_heading_style(s: html2markdown::HeadingStyle) -> HeadingStyle {
    match s {
        html2markdown::HeadingStyle::Atx => HeadingStyle::Atx,
        html2markdown::HeadingStyle::Setext => HeadingStyle::Setext,
    }
}

fn convert_list_item_indent(i: html2markdown::ListItemIndent) -> ListItemIndent {
    match i {
        html2markdown::ListItemIndent::One => ListItemIndent::One,
        html2markdown::ListItemIndent::Tab => ListItemIndent::Tab,
        html2markdown::ListItemIndent::Mixed => ListItemIndent::Mixed,
    }
}

fn to_core_stringify_options(
    opts: StringifyOptions,
) -> Result<html2markdown::StringifyOptions, OptionsError> {
    if opts.rule_repetition < 3 {
        return Err(OptionsError::InvalidOption {
            field: "rule_repetition".to_string(),
            allowed: "3..=255".to_string(),
            value: opts.rule_repetition.to_string(),
        });
    }

    Ok(html2markdown::StringifyOptions {
        heading_style: match opts.heading_style {
            HeadingStyle::Atx => html2markdown::HeadingStyle::Atx,
            HeadingStyle::Setext => html2markdown::HeadingStyle::Setext,
        },
        bullet: parse_char(&opts.bullet, "bullet", &['*', '-', '+'])?,
        bullet_ordered: parse_char(&opts.bullet_ordered, "bullet_ordered", &['.', ')'])?,
        emphasis: parse_char(&opts.emphasis, "emphasis", &['*', '_'])?,
        strong: parse_char(&opts.strong, "strong", &['*', '_'])?,
        fence: parse_char(&opts.fence, "fence", &['`', '~'])?,
        rule: parse_char(&opts.rule, "rule", &['*', '-', '_'])?,
        rule_repetition: opts.rule_repetition,
        rule_spaces: opts.rule_spaces,
        close_atx: opts.close_atx,
        list_item_indent: match opts.list_item_indent {
            ListItemIndent::One => html2markdown::ListItemIndent::One,
            ListItemIndent::Tab => html2markdown::ListItemIndent::Tab,
            ListItemIndent::Mixed => html2markdown::ListItemIndent::Mixed,
        },
        increment_list_marker: opts.increment_list_marker,
        quote: parse_char(&opts.quote, "quote", &['"', '\''])?,
        fences: opts.fences,
        resource_link: opts.resource_link,
    })
}

fn to_core_options(opts: Options) -> Result<html2markdown::Options, OptionsError> {
    Ok(html2markdown::Options {
        stringify: to_core_stringify_options(opts.stringify)?,
        newlines: opts.newlines,
        checked: opts.checked,
        unchecked: opts.unchecked,
        quotes: opts.quotes,
    })
}
