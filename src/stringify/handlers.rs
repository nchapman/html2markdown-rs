// Node type handlers for MDAST → Markdown serialization.
//
// One handler per MDAST node type. Each takes a State and Node, returns a String.

use std::borrow::Cow;

use super::State;
use crate::mdast::{self, Node};

/// Dispatch to the appropriate handler for a node.
pub(crate) fn handle(state: &mut State, node: &Node) -> String {
    match node {
        Node::Root(n) => handle_root(state, n),
        Node::Paragraph(n) => handle_paragraph(state, n),
        Node::Heading(n) => handle_heading(state, n),
        Node::ThematicBreak(_) => handle_thematic_break(state),
        Node::Blockquote(n) => handle_blockquote(state, n),
        Node::List(n) => handle_list(state, n),
        Node::ListItem(n) => handle_list_item(state, n),
        Node::Code(n) => handle_code(state, n),
        Node::Html(n) => handle_html(n),
        Node::Definition(n) => handle_definition(n),
        Node::Text(n) => handle_text(state, n),
        Node::Emphasis(n) => handle_emphasis(state, n),
        Node::Strong(n) => handle_strong(state, n),
        Node::InlineCode(n) => handle_inline_code(n),
        Node::Break(_) => handle_break(),
        Node::Link(n) => handle_link(state, n),
        Node::Image(n) => handle_image(n),
        Node::LinkReference(n) => handle_link_reference(state, n),
        Node::ImageReference(n) => handle_image_reference(n),
        Node::Delete(n) => handle_delete(state, n),
        Node::Table(n) => handle_table(state, n),
        Node::TableRow(_) | Node::TableCell(_) => {
            // Handled by table handler directly.
            String::new()
        }
        Node::FootnoteDefinition(n) => handle_footnote_definition(state, n),
        Node::FootnoteReference(n) => handle_footnote_reference(n),
        Node::Yaml(n) => handle_yaml(n),
    }
}

// ---------------------------------------------------------------------------
// Flow (block) handlers
// ---------------------------------------------------------------------------

fn handle_root(state: &mut State, node: &mdast::Root) -> String {
    super::flow::container_flow(state, &node.children)
}

fn handle_paragraph(state: &mut State, node: &mdast::Paragraph) -> String {
    state.at_break = true;
    let content = super::phrasing::container_phrasing(state, &node.children);
    state.at_break = false;
    content
}

fn handle_heading(state: &mut State, node: &mdast::Heading) -> String {
    let content = super::phrasing::container_phrasing(state, &node.children);

    // Use setext for h1/h2 if: (a) setext style is configured, or (b) content
    // contains a newline (from Break nodes or text with preserved newlines).
    // ATX headings cannot span multiple lines, so setext is the only valid choice.
    let use_setext = node.depth <= 2
        && (matches!(state.options.heading_style, super::HeadingStyle::Setext)
            || content.contains('\n'));

    if use_setext {
        let marker = if node.depth == 1 { '=' } else { '-' };
        let line_len = content
            .lines()
            .last()
            .map_or(content.chars().count(), |l| l.chars().count());
        let underline_len = line_len.max(3);
        return format!("{}\n{}", content, marker.to_string().repeat(underline_len));
    }

    // ATX heading: replace hard breaks first, then bare newlines.
    // Order matters: reversing would corrupt "\\\n" (the \n would be replaced first).
    let mut content = content.replace("\\\n", " ").replace('\n', "&#xA;");

    // Encode leading space/tab so CommonMark doesn't strip it as insignificant
    // whitespace. Port of mdast-util-to-markdown heading.js.
    if content.starts_with(' ') {
        content.replace_range(..1, "&#x20;");
    } else if content.starts_with('\t') {
        content.replace_range(..1, "&#x9;");
    }

    // Escape trailing `#` sequence if preceded by a space (or content is all `#`),
    // which CommonMark parsers strip as the optional ATX closing sequence.
    let content = escape_atx_trailing_hashes(content);

    let hashes = "#".repeat(node.depth as usize);
    if state.options.close_atx {
        format!("{} {} {}", hashes, content, hashes)
    } else {
        format!("{} {}", hashes, content)
    }
}

/// Escape the trailing `#` sequence in ATX heading content so CommonMark
/// parsers don't strip it as an optional closing sequence.
///
/// The spec strips a trailing ` #+` (space then one-or-more `#`) from ATX
/// heading content. We prevent that by inserting `\` before the first `#` in
/// the trailing run, making it a backslash-escaped `#` in inline parsing.
fn escape_atx_trailing_hashes(content: String) -> String {
    if !content.ends_with('#') {
        return content;
    }
    // `trimmed` is the content with the trailing # run removed.
    // `trim_end_matches` returns a valid &str slice so `trimmed.len()` is a
    // safe byte index back into `content`.
    let trimmed = content.trim_end_matches('#');
    let hash_start_byte = trimmed.len();
    // Only escape when the # run is preceded by a space (or content is all #).
    // Use `str::ends_with` so this is correct for any Unicode prefix.
    let preceded_by_space = trimmed.is_empty() || trimmed.ends_with(' ');
    if preceded_by_space {
        format!("{}\\{}", trimmed, &content[hash_start_byte..])
    } else {
        content
    }
}

fn handle_thematic_break(state: &mut State) -> String {
    let marker = state.options.rule;
    let count = state.options.rule_repetition as usize;
    let mut s = String::with_capacity(count * 2);
    for i in 0..count {
        if state.options.rule_spaces && i > 0 {
            s.push(' ');
        }
        s.push(marker);
    }
    s
}

fn handle_blockquote(state: &mut State, node: &mdast::Blockquote) -> String {
    let content = super::flow::container_flow(state, &node.children);
    if content.is_empty() {
        return ">".to_string();
    }
    content
        .lines()
        .map(|line| {
            if line.is_empty() {
                ">".to_string()
            } else {
                format!("> {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn handle_list(state: &mut State, node: &mdast::List) -> String {
    let mut result = Vec::new();
    let old_bullet = state.bullet_current;

    // For unordered lists: alternate bullets when sibling list used the same bullet.
    // For ordered lists: alternate `.` / `)` delimiters when the previous sibling
    // was also an ordered list, to prevent CommonMark parsers from merging them.
    // (Two adjacent ordered lists with the same delimiter are indistinguishable
    // from a single list; switching to `)` forces a new list.)
    let ordered_delimiter = if node.ordered {
        let pref = state.options.bullet_ordered;
        if state.ordered_bullet_last_used.is_some() {
            if pref == '.' {
                ')'
            } else {
                '.'
            }
        } else {
            pref
        }
    } else {
        // Alternate unordered bullets.
        let bullet = if state.bullet_last_used == Some(state.options.bullet) {
            if state.options.bullet == '*' {
                '-'
            } else {
                '*'
            }
        } else {
            state.options.bullet
        };
        state.bullet_current = Some(bullet);
        '.' // unused for unordered
    };

    for (i, child) in node.children.iter().enumerate() {
        let prefix = if node.ordered {
            let number = if state.options.increment_list_marker {
                node.start.unwrap_or(1) + i as u32
            } else {
                node.start.unwrap_or(1)
            };
            format!("{}{}", number, ordered_delimiter)
        } else {
            format!("{}", state.bullet_current.unwrap_or('*'))
        };

        let content = handle_list_item_with_parent(state, child, node);
        // Reset bullet_last_used after each list item to prevent state from
        // nested lists in one item leaking into sibling items' nested lists.
        state.bullet_last_used = None;

        // Compute indent based on list_item_indent option.
        // Port of mdast-util-to-markdown list-item.js indentation logic.
        let indent_width = match state.options.list_item_indent {
            super::ListItemIndent::Tab => {
                // Round up to next multiple of 4 (tab stop), minimum prefix + 1.
                let min = prefix.len() + 1;
                min.div_ceil(4) * 4
            }
            super::ListItemIndent::Mixed => {
                // Use tab-style indent when the item is spread (multiple children),
                // otherwise use one-space indent.
                let is_spread = if let Node::ListItem(li) = child {
                    li.spread || node.spread
                } else {
                    false
                };
                if is_spread {
                    let min = prefix.len() + 1;
                    min.div_ceil(4) * 4
                } else {
                    prefix.len() + 1
                }
            }
            super::ListItemIndent::One => prefix.len() + 1,
        };
        let indent = " ".repeat(indent_width);

        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        // Don't add trailing space if the first line is empty (empty list item).
        // Pad between prefix and content to match the computed indent width.
        let padding = " ".repeat(indent_width - prefix.len());
        let first = if lines[0].is_empty() {
            prefix.clone()
        } else {
            format!("{}{}{}", prefix, padding, lines[0])
        };
        let rest: Vec<String> = lines[1..]
            .iter()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect();

        let mut item = first;
        for line in rest {
            item.push('\n');
            item.push_str(&line);
        }
        result.push(item);
    }

    // Set bullet trackers AFTER processing children.
    if node.ordered {
        state.ordered_bullet_last_used = Some(ordered_delimiter);
    } else {
        state.bullet_last_used = state.bullet_current;
    }
    state.bullet_current = old_bullet;

    let separator = if node.spread { "\n\n" } else { "\n" };
    result.join(separator)
}

/// Render a list item, respecting whether the parent list is spread.
fn handle_list_item_with_parent(state: &mut State, node: &Node, parent: &mdast::List) -> String {
    let spread = parent.spread
        || if let Node::ListItem(li) = node {
            li.spread
        } else {
            false
        };

    let content = if let Node::ListItem(li) = node {
        let mut content = super::flow::container_flow_tight(state, &li.children, spread);

        if let Some(checked) = li.checked {
            let checkbox = if checked { "[x]" } else { "[ ]" };
            if content.is_empty() {
                content = checkbox.to_string();
            } else {
                content = format!("{} {}", checkbox, content);
            }
        }
        content
    } else {
        handle(state, node)
    };

    content
}

fn handle_list_item(state: &mut State, node: &mdast::ListItem) -> String {
    // This is called directly (not via handle_list), so we don't know spread.
    // Default to the node's own spread setting.
    let mut content = super::flow::container_flow_tight(state, &node.children, node.spread);

    if let Some(checked) = node.checked {
        let checkbox = if checked { "[x]" } else { "[ ]" };
        if content.is_empty() {
            content = checkbox.to_string();
        } else {
            content = format!("{} {}", checkbox, content);
        }
    }

    content
}

fn handle_code(state: &mut State, node: &mdast::Code) -> String {
    let has_info = node.lang.is_some() || node.meta.is_some();

    // When fences are disabled and there's no info string, emit 4-space indented code
    // if the value is suitable. Port of mdast-util-to-markdown code.js +
    // format-code-as-indented.js guards: must have non-whitespace content and must
    // not start or end with a blank line (which would break indented code parsing).
    if !state.options.fences && !has_info && can_format_as_indented(&node.value) {
        let indented: String = node
            .value
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("    {}", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        return indented;
    }

    let mut fence_char = state.options.fence;

    // If the lang contains a backtick and we're using backtick fences, switch to tilde.
    // A backtick in the info string would prematurely close the fence.
    // Port of mdast-util-to-markdown code.js.
    let lang = node.lang.as_deref().unwrap_or("");
    if lang.contains('`') && fence_char == '`' {
        fence_char = '~';
    }

    // Find minimum fence length that doesn't conflict with content.
    let content_max = node
        .value
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.chars().all(|c| c == fence_char) && trimmed.len() >= 3 {
                Some(trimmed.len())
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);
    let fence_len = (content_max + 1).max(3);
    let fence: String = std::iter::repeat(fence_char).take(fence_len).collect();

    // Escape space in lang as &#x20; when meta is present, because the first
    // space in the info string separates lang from meta. Without meta, a space
    // in the lang is harmless (the entire info string IS the lang).
    // Port of mdast-util-to-markdown code.js.
    let info = if node.meta.is_some() {
        lang.replace(' ', "&#x20;")
    } else {
        lang.to_string()
    };

    // Replace newlines in meta with spaces — newlines would break the fence line.
    let meta = node
        .meta
        .as_ref()
        .map(|m| format!(" {}", m.replace('\n', " ")))
        .unwrap_or_default();

    if node.value.is_empty() {
        format!("{}{}{}\n{}", fence, info, meta, fence)
    } else {
        format!("{}{}{}\n{}\n{}", fence, info, meta, node.value, fence)
    }
}

fn handle_html(node: &mdast::Html) -> String {
    node.value.clone()
}

fn handle_definition(node: &mdast::Definition) -> String {
    let raw_label = node.label.as_deref().unwrap_or(&node.identifier);
    // Escape `]` (and other phrasing chars) so it doesn't prematurely close
    // the `[label]` bracket. Port of mdast-util-to-markdown definition.js.
    let label = super::escape::escape_link_text(raw_label);
    let url = format_link_url(&node.url);
    match &node.title {
        Some(title) => format!("[{}]: {} \"{}\"", label, url, escape_link_title(title)),
        None => format!("[{}]: {}", label, url),
    }
}

// ---------------------------------------------------------------------------
// Phrasing (inline) handlers
// ---------------------------------------------------------------------------

fn handle_text(state: &mut State, node: &mdast::Text) -> String {
    // Escape Markdown syntax characters in phrasing content.
    // Port of mdast-util-to-markdown's `safe()` function.
    // When inside link text (`[…]`), also escape `]` to prevent premature
    // bracket close. (We don't escape `]` globally because it would corrupt
    // task-list checkbox syntax like `\[ ]` emitted by the list-item handler.)
    let escaped = if state.in_link_text {
        super::escape::escape_link_text(&node.value)
    } else {
        super::escape::escape_phrasing(&node.value)
    };
    // Escape `|` inside table cells to prevent breaking table structure.
    // Port of mdast-util-to-markdown unsafe: {character: '|', inConstruct: 'tableCellContent'}
    let escaped = if state.in_table_cell {
        Cow::Owned(escaped.replace('|', "\\|"))
    } else {
        escaped
    };
    // Apply at-break escaping if this text is at the start of a block.
    if state.at_break {
        state.at_break = false;
        super::escape::escape_at_break_start(escaped.into_owned())
    } else {
        escaped.into_owned()
    }
}

fn handle_emphasis(state: &mut State, node: &mdast::Emphasis) -> String {
    let marker = state.options.emphasis;
    let content = super::phrasing::container_phrasing(state, &node.children);
    // When the inner content begins or ends with exactly ONE instance of the
    // marker (another emphasis span like `*foo*`), wrapping it with the same
    // marker would produce `**…**` which CommonMark parses as strong.
    // Switch to the alternate delimiter to get `*_foo_*` / `_*foo*_`.
    // We do NOT switch for double-marker content like `**bar**` (strong),
    // because `***bar***` is correctly parsed as `<em><strong>…</strong></em>`.
    // Marker is always ASCII (* or _), so byte indexing is safe and O(1).
    let m = marker as u8;
    let bytes = content.as_bytes();
    let starts_single = bytes.first() == Some(&m) && bytes.get(1) != Some(&m);
    let ends_single = bytes.last() == Some(&m) && bytes.len() >= 2 && bytes[bytes.len() - 2] != m;
    let actual_marker = if starts_single || ends_single {
        if marker == '*' {
            '_'
        } else {
            '*'
        }
    } else {
        marker
    };
    format!("{}{}{}", actual_marker, content, actual_marker)
}

fn handle_strong(state: &mut State, node: &mdast::Strong) -> String {
    let marker = state.options.strong;
    let content = super::phrasing::container_phrasing(state, &node.children);
    format!("{0}{0}{1}{0}{0}", marker, content)
}

fn handle_inline_code(node: &mdast::InlineCode) -> String {
    // Replace newlines with spaces — a newline inside inline code can trigger
    // block constructs when re-parsed (e.g. `\n#` becomes an ATX heading).
    // Port of mdast-util-to-markdown inline-code.js.
    let value = node.value.replace('\n', " ");

    // Choose backtick count to avoid conflicts with content.
    let max_run = longest_backtick_run(&value);
    let ticks = "`".repeat(max_run + 1);

    let needs_space = value.starts_with('`')
        || value.ends_with('`')
        || (value.starts_with(' ') && value.ends_with(' ') && !value.trim().is_empty());

    if needs_space {
        format!("{} {} {}", ticks, value, ticks)
    } else {
        format!("{}{}{}", ticks, value, ticks)
    }
}

fn handle_break() -> String {
    "\\\n".to_string()
}

fn handle_link(state: &mut State, node: &mdast::Link) -> String {
    // Trim only leading whitespace — trailing is handled by MDAST normalization
    // (normalize_inline_boundaries in whitespace.rs) which moves the space
    // inside the link when it is the sole separator before the next token.
    state.in_link_text = true;
    let content = super::phrasing::container_phrasing(state, &node.children);
    state.in_link_text = false;
    let content = content.trim_start();

    // Try to format as autolink: <url> or <email>
    // Port of mdast-util-to-markdown/lib/util/format-link-as-autolink.js
    if !state.options.resource_link
        && !node.url.is_empty()
        && node.title.is_none()
        && node.children.len() == 1
        && matches!(&node.children[0], mdast::Node::Text(_))
        && (content == node.url.as_str() || format!("mailto:{}", content) == node.url)
        && node.url.contains(':')
        && !node
            .url
            .chars()
            .any(|c| c <= ' ' || c == '<' || c == '>' || c == '\x7f')
    {
        return format!("<{}>", content);
    }

    let url = format_link_url(&node.url);
    match &node.title {
        Some(title) => format!("[{}]({} \"{}\")", content, url, escape_link_title(title)),
        None => format!("[{}]({})", content, url),
    }
}

fn handle_image(node: &mdast::Image) -> String {
    // Escape `]` and other phrasing chars in alt text to prevent premature
    // bracket close. Port of mdast-util-to-markdown image.js safe() call.
    let alt = super::escape::escape_link_text(&node.alt);
    let url = format_link_url(&node.url);
    match &node.title {
        Some(title) => format!("![{}]({} \"{}\")", alt, url, escape_link_title(title)),
        None => format!("![{}]({})", alt, url),
    }
}

/// Escape `\` and `"` in link/image titles so they don't corrupt the
/// double-quoted title delimiter. Backslash must be escaped first to avoid
/// double-escaping the backslashes introduced for `"`.
fn escape_link_title(title: &str) -> String {
    title.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Format a URL for use inside `[text](url)` syntax.
///
/// When the URL contains `)` with net-negative parenthesis depth, CommonMark
/// parsers close the link destination early, producing broken links. Wrapping
/// in `<…>` avoids this while still allowing any URL characters.
fn format_link_url(url: &str) -> String {
    if link_url_needs_angle_brackets(url) {
        format!("<{}>", url)
    } else {
        url.to_string()
    }
}

fn link_url_needs_angle_brackets(url: &str) -> bool {
    let mut depth: i32 = 0;
    for c in url.chars() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return true;
                }
            }
            // Whitespace terminates a bare link destination.
            ' ' | '\t' | '\n' => return true,
            // `<` and `>` are disallowed inside angle-bracket form too, so we
            // flag them here; callers should percent-encode them if possible.
            '<' | '>' => return true,
            _ => {}
        }
    }
    depth != 0
}

fn handle_link_reference(state: &mut State, node: &mdast::LinkReference) -> String {
    state.in_link_text = true;
    let content = super::phrasing::container_phrasing(state, &node.children);
    state.in_link_text = false;
    let raw_label = node.label.as_deref().unwrap_or(&node.identifier);
    // Escape the reference label to prevent `]` from prematurely closing
    // the `[content][label]` bracket. Port of mdast-util-to-markdown link-reference.js.
    let label = super::escape::escape_link_text(raw_label);
    match node.reference_kind {
        mdast::ReferenceKind::Shortcut => format!("[{}]", content),
        mdast::ReferenceKind::Collapsed => format!("[{}][]", content),
        mdast::ReferenceKind::Full => format!("[{}][{}]", content, label),
    }
}

fn handle_image_reference(node: &mdast::ImageReference) -> String {
    let raw_label = node.label.as_deref().unwrap_or(&node.identifier);
    // Escape alt and label to prevent `]` from prematurely closing brackets.
    // Port of mdast-util-to-markdown image-reference.js.
    let alt = super::escape::escape_link_text(&node.alt);
    let label = super::escape::escape_link_text(raw_label);
    match node.reference_kind {
        mdast::ReferenceKind::Shortcut => format!("![{}]", alt),
        mdast::ReferenceKind::Collapsed => format!("![{}][]", alt),
        mdast::ReferenceKind::Full => format!("![{}][{}]", alt, label),
    }
}

fn handle_delete(state: &mut State, node: &mdast::Delete) -> String {
    let content = super::phrasing::container_phrasing(state, &node.children);
    format!("~~{}~~", content)
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

fn handle_table(state: &mut State, node: &mdast::Table) -> String {
    if node.children.is_empty() {
        return String::new();
    }

    // Collect all cell contents. Trim leading/trailing whitespace from each cell
    // (whitespace from HTML indentation between elements within cells).
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in &node.children {
        if let Node::TableRow(tr) = row {
            let cells: Vec<String> = tr
                .children
                .iter()
                .map(|cell| {
                    if let Node::TableCell(tc) = cell {
                        state.in_table_cell = true;
                        let content = super::phrasing::container_phrasing(state, &tc.children);
                        state.in_table_cell = false;
                        // Hard breaks (\<LF>) → space; bare newlines → &#xA; escape.
                        content.trim().replace("\\\n", " ").replace('\n', "&#xA;")
                    } else {
                        String::new()
                    }
                })
                .collect();
            rows.push(cells);
        }
    }

    if rows.is_empty() {
        return String::new();
    }

    // Determine column count and widths.
    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths = vec![1usize; col_count]; // minimum 1
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                // Measures escaped string length — sequences like \| count as 2 chars
                // but render as 1. Matches JS reference behavior; parsers ignore extra padding.
                col_widths[i] = col_widths[i].max(cell.chars().count());
            }
        }
    }

    let mut lines = Vec::new();

    // Header row.
    let header = &rows[0];
    let header_line = format_row(header, &col_widths, col_count, &node.align);
    lines.push(header_line);

    // Separator row.
    let sep: Vec<String> = (0..col_count)
        .map(|i| {
            let width = col_widths[i];
            let align = node.align.get(i).copied().flatten();
            format_separator(width, align)
        })
        .collect();
    lines.push(format!("| {} |", sep.join(" | ")));

    // Data rows.
    for row in rows.iter().skip(1) {
        lines.push(format_row(row, &col_widths, col_count, &node.align));
    }

    lines.join("\n")
}

fn format_row(
    cells: &[String],
    widths: &[usize],
    col_count: usize,
    aligns: &[Option<crate::mdast::AlignKind>],
) -> String {
    let padded: Vec<String> = (0..col_count)
        .map(|i| {
            let content = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            let width = widths[i];
            let align = aligns.get(i).copied().flatten();
            pad_cell(content, width, align)
        })
        .collect();
    format!("| {} |", padded.join(" | "))
}

fn pad_cell(content: &str, width: usize, align: Option<crate::mdast::AlignKind>) -> String {
    use crate::mdast::AlignKind;
    let len = content.chars().count();
    let padding = width.saturating_sub(len);
    match align {
        Some(AlignKind::Right) => {
            format!("{}{}", " ".repeat(padding), content)
        }
        Some(AlignKind::Center) => {
            // JS uses ceiling division for left pad.
            let left_pad = padding.div_ceil(2);
            let right_pad = padding / 2;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                content,
                " ".repeat(right_pad)
            )
        }
        _ => {
            // Left-align (default): pad right
            format!("{}{}", content, " ".repeat(padding))
        }
    }
}

fn format_separator(width: usize, align: Option<crate::mdast::AlignKind>) -> String {
    use crate::mdast::AlignKind;
    match align {
        Some(AlignKind::Left) => {
            // Minimum: :- (2 chars). Extra dashes fill up to width.
            format!(":{}", "-".repeat(width.saturating_sub(1)))
        }
        Some(AlignKind::Right) => {
            // Minimum: -: (2 chars). Extra dashes fill up to width.
            format!("{}:", "-".repeat(width.saturating_sub(1)))
        }
        Some(AlignKind::Center) => {
            // Minimum: :-: (3 chars). Extra dashes between colons fill up to width.
            format!(":{}:", "-".repeat(width.saturating_sub(2)))
        }
        None => "-".repeat(width),
    }
}

// ---------------------------------------------------------------------------
// Footnotes
// ---------------------------------------------------------------------------

fn handle_footnote_definition(state: &mut State, node: &mdast::FootnoteDefinition) -> String {
    let label = node.label.as_deref().unwrap_or(&node.identifier);
    let content = super::flow::container_flow(state, &node.children);
    let indent = "    ";
    let indented: Vec<String> = content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                line.to_string()
            } else if line.is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect();
    format!("[^{}]: {}", label, indented.join("\n"))
}

fn handle_footnote_reference(node: &mdast::FootnoteReference) -> String {
    let label = node.label.as_deref().unwrap_or(&node.identifier);
    format!("[^{}]", label)
}

// ---------------------------------------------------------------------------
// Frontmatter
// ---------------------------------------------------------------------------

fn handle_yaml(node: &mdast::Yaml) -> String {
    format!("---\n{}\n---", node.value)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check whether a code value can be formatted as a 4-space indented block.
/// Port of mdast-util-to-markdown format-code-as-indented.js.
/// Returns false for empty values, all-whitespace values, or values that
/// start/end with a blank line (which break indented code block parsing).
fn can_format_as_indented(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if !value.contains(|c: char| c != ' ' && c != '\t' && c != '\r' && c != '\n') {
        return false;
    }
    // Check for blank-line margins: first or last line is blank/whitespace-only.
    let first_line = value.lines().next().unwrap_or("");
    if first_line.chars().all(|c| c == ' ' || c == '\t') {
        return false;
    }
    // Check last line: split the value at the final newline.
    if let Some(pos) = value.rfind('\n') {
        let last_line = &value[pos + 1..];
        if last_line.chars().all(|c| c == ' ' || c == '\t') {
            return false;
        }
    }
    true
}

/// Find the longest consecutive run of backticks in a string.
fn longest_backtick_run(s: &str) -> usize {
    let mut max = 0;
    let mut current = 0;
    for c in s.chars() {
        if c == '`' {
            current += 1;
            max = max.max(current);
        } else {
            current = 0;
        }
    }
    max
}
