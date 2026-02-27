// Node type handlers for MDAST → Markdown serialization.
//
// One handler per MDAST node type. Each takes a State and Node, returns a String.

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
        let line_len = content.lines().last().map_or(content.chars().count(), |l| l.chars().count());
        let underline_len = line_len.max(3);
        return format!(
            "{}\n{}",
            content,
            marker.to_string().repeat(underline_len)
        );
    }

    // ATX heading: replace hard breaks first, then bare newlines.
    // Order matters: reversing would corrupt "\\\n" (the \n would be replaced first).
    let content = content.replace("\\\n", " ").replace('\n', "&#xA;");
    let hashes = "#".repeat(node.depth as usize);
    if state.options.close_atx {
        format!("{} {} {}", hashes, content, hashes)
    } else {
        format!("{} {}", hashes, content)
    }
}

fn handle_thematic_break(state: &mut State) -> String {
    let marker = state.options.rule;
    let count = state.options.rule_repetition as usize;
    if state.options.rule_spaces {
        let parts: Vec<String> = std::iter::repeat(marker.to_string()).take(count).collect();
        parts.join(" ")
    } else {
        std::iter::repeat(marker).take(count).collect()
    }
}

fn handle_blockquote(state: &mut State, node: &mdast::Blockquote) -> String {
    let content = super::flow::container_flow(state, &node.children);
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

    if !node.ordered {
        // Alternate bullets only when bullet_last_used == our preferred bullet.
        // bullet_last_used is set AFTER children are processed (matches JS behavior),
        // so it reflects the PREVIOUSLY completed sibling list's bullet.
        // Between non-list flow children, bullet_last_used is reset to None.
        let bullet = if state.bullet_last_used == Some(state.options.bullet) {
            // Use an alternate bullet to avoid ambiguity.
            if state.options.bullet == '*' { '-' } else { '*' }
        } else {
            state.options.bullet
        };
        state.bullet_current = Some(bullet);
        // bullet_last_used will be set AFTER processing children (below).
    }

    for (i, child) in node.children.iter().enumerate() {
        let prefix = if node.ordered {
            let number = if state.options.increment_list_marker {
                node.start.unwrap_or(1) + i as u32
            } else {
                node.start.unwrap_or(1)
            };
            format!("{}{}", number, state.options.bullet_ordered)
        } else {
            format!("{}", state.bullet_current.unwrap_or('*'))
        };

        let content = handle_list_item_with_parent(state, child, node);
        // Reset bullet_last_used after each list item to prevent state from
        // nested lists in one item leaking into sibling items' nested lists.
        state.bullet_last_used = None;
        let indent_width = prefix.len() + 1; // +1 for the space after bullet
        let indent = " ".repeat(indent_width);

        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        // Don't add trailing space if the first line is empty (empty list item).
        let first = if lines[0].is_empty() {
            prefix.clone()
        } else {
            format!("{} {}", prefix, lines[0])
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

    // Set bullet_last_used AFTER processing children (same as JS: `state.bulletLastUsed = bullet`).
    if !node.ordered {
        state.bullet_last_used = state.bullet_current;
    }
    state.bullet_current = old_bullet;

    let separator = if node.spread { "\n\n" } else { "\n" };
    result.join(separator)
}

/// Render a list item, respecting whether the parent list is spread.
fn handle_list_item_with_parent(state: &mut State, node: &Node, parent: &mdast::List) -> String {
    let spread = parent.spread || if let Node::ListItem(li) = node { li.spread } else { false };

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
    let fence_char = state.options.fence;
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

    let info = node.lang.as_deref().unwrap_or("");
    let meta = node
        .meta
        .as_ref()
        .map(|m| format!(" {}", m))
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
    let label = node.label.as_deref().unwrap_or(&node.identifier);
    match &node.title {
        Some(title) => format!("[{}]: {} \"{}\"", label, node.url, title),
        None => format!("[{}]: {}", label, node.url),
    }
}

// ---------------------------------------------------------------------------
// Phrasing (inline) handlers
// ---------------------------------------------------------------------------

fn handle_text(state: &mut State, node: &mdast::Text) -> String {
    // Escape Markdown syntax characters in phrasing content.
    // Port of mdast-util-to-markdown's `safe()` function.
    let escaped = super::escape::escape_phrasing(&node.value);
    // Apply at-break escaping if this text is at the start of a block.
    if state.at_break {
        state.at_break = false;
        super::escape::escape_at_break_start(escaped)
    } else {
        escaped
    }
}

fn handle_emphasis(state: &mut State, node: &mdast::Emphasis) -> String {
    let marker = state.options.emphasis;
    let content = super::phrasing::container_phrasing(state, &node.children);
    format!("{}{}{}", marker, content, marker)
}

fn handle_strong(state: &mut State, node: &mdast::Strong) -> String {
    let marker = state.options.strong;
    let content = super::phrasing::container_phrasing(state, &node.children);
    format!("{0}{0}{1}{0}{0}", marker, content)
}

fn handle_inline_code(node: &mdast::InlineCode) -> String {
    // Choose backtick count to avoid conflicts with content.
    let max_run = longest_backtick_run(&node.value);
    let ticks = "`".repeat(max_run + 1);

    let needs_space = node.value.starts_with('`')
        || node.value.ends_with('`')
        || (node.value.starts_with(' ') && node.value.ends_with(' ') && !node.value.trim().is_empty());

    if needs_space {
        format!("{} {} {}", ticks, node.value, ticks)
    } else {
        format!("{}{}{}", ticks, node.value, ticks)
    }
}

fn handle_break() -> String {
    "\\\n".to_string()
}

fn handle_link(state: &mut State, node: &mdast::Link) -> String {
    // Trim only leading whitespace — trailing is handled by MDAST normalization
    // (normalize_inline_boundaries in whitespace.rs) which moves the space
    // inside the link when it is the sole separator before the next token.
    let content = super::phrasing::container_phrasing(state, &node.children);
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
        && !node.url.chars().any(|c| c <= ' ' || c == '<' || c == '>' || c == '\x7f')
    {
        return format!("<{}>", content);
    }

    match &node.title {
        Some(title) => format!("[{}]({} \"{}\")", content, node.url, title),
        None => format!("[{}]({})", content, node.url),
    }
}

fn handle_image(node: &mdast::Image) -> String {
    match &node.title {
        Some(title) => format!("![{}]({} \"{}\")", node.alt, node.url, title),
        None => format!("![{}]({})", node.alt, node.url),
    }
}

fn handle_link_reference(state: &mut State, node: &mdast::LinkReference) -> String {
    let content = super::phrasing::container_phrasing(state, &node.children);
    let label = node.label.as_deref().unwrap_or(&node.identifier);
    match node.reference_kind {
        mdast::ReferenceKind::Shortcut => format!("[{}]", content),
        mdast::ReferenceKind::Collapsed => format!("[{}][]", content),
        mdast::ReferenceKind::Full => format!("[{}][{}]", content, label),
    }
}

fn handle_image_reference(node: &mdast::ImageReference) -> String {
    let label = node.label.as_deref().unwrap_or(&node.identifier);
    match node.reference_kind {
        mdast::ReferenceKind::Shortcut => format!("![{}]", node.alt),
        mdast::ReferenceKind::Collapsed => format!("![{}][]", node.alt),
        mdast::ReferenceKind::Full => format!("![{}][{}]", node.alt, label),
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
                        let content = super::phrasing::container_phrasing(state, &tc.children);
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
                col_widths[i] = col_widths[i].max(cell.len());
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

fn format_row(cells: &[String], widths: &[usize], col_count: usize, aligns: &[Option<crate::mdast::AlignKind>]) -> String {
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
            format!("{}{}{}", " ".repeat(left_pad), content, " ".repeat(right_pad))
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
