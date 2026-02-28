// Whitespace normalization for MDAST trees.
//
// Post-processing pass that merges adjacent text nodes, collapses whitespace
// around line endings, and trims leading/trailing whitespace in headings,
// paragraphs, and root nodes.

use crate::mdast::Node;

/// Run whitespace post-processing on an MDAST tree.
pub(crate) fn post_process_whitespace(node: &mut Node) {
    post_process_whitespace_inner(node, 0);
}

fn post_process_whitespace_inner(node: &mut Node, depth: usize) {
    if depth >= super::MAX_DEPTH {
        return;
    }
    // Recursively process children first.
    if let Some(children) = node.children_mut() {
        for child in children.iter_mut() {
            post_process_whitespace_inner(child, depth + 1);
        }

        // Merge adjacent text nodes.
        merge_adjacent_text(children);

        // Remove empty text nodes.
        children.retain(|child| !is_empty_text(child));

        // Normalize inline element boundaries: deduplicate spaces at
        // Link/Delete edges and trim their leading/trailing whitespace.
        // This mirrors rehype-minify-whitespace's inline whitespace handling:
        // when an inline element's last text ends with a space and the
        // following text starts with a space, the space stays inside the
        // element and is removed from the following text.
        normalize_inline_boundaries(children);
    }

    // Trim leading/trailing whitespace in specific containers.
    // Delete is included because straddling splits can produce trailing spaces
    // in delete content at block boundaries (mirrors rehype-minify-whitespace).
    let should_trim = matches!(
        node,
        Node::Heading(_) | Node::Paragraph(_) | Node::Root(_) | Node::Delete(_)
    );
    if should_trim {
        if let Some(children) = node.children_mut() {
            trim_container(children);
        }
    }
}

/// Normalize whitespace at Link/Delete boundaries within a phrasing run.
///
/// For each Link or Delete node in `children`:
///   1. Trim leading whitespace from its first text child (always).
///   2. If its last text child ends with ' ' AND the immediately following
///      sibling is a Text starting with ' ', remove the leading ' ' from that
///      sibling (deduplication: the space lives inside the inline element).
///   3. Trim trailing whitespace from the last text child ONLY when no space
///      deduplication occurred (i.e., the following sibling already supplies a
///      space, or there is no following sibling).
///
/// This replicates the subset of rehype-minify-whitespace behaviour that is
/// observable in the fixture tests.
fn normalize_inline_boundaries(children: &mut Vec<Node>) {
    let n = children.len();
    for i in 0..n {
        if !is_link_or_delete(&children[i]) {
            continue;
        }

        // Does the following sibling start with whitespace?
        let following_starts_with_space =
            i + 1 < n && matches!(&children[i + 1], Node::Text(t) if t.value.starts_with(' '));

        // Does this inline element's last text end with whitespace?
        let self_ends_with_space = inline_last_text_ends_with_space(&children[i]);

        if self_ends_with_space && following_starts_with_space {
            // Dedup: remove leading space from the following text — the space
            // belongs inside the inline element.
            if let Node::Text(ref mut t) = children[i + 1] {
                let trimmed = t.value.trim_start_matches(' ');
                if trimmed.len() != t.value.len() {
                    t.value = trimmed.to_string();
                }
            }
            // The following sibling no longer starts with space, so the
            // trailing space of the inline element is the only separator:
            // don't trim it (trim_trailing = false).
            trim_inline_leading(&mut children[i]);
            // trim_trailing intentionally skipped.
        } else {
            // No dedup occurred. Trim both ends of the inline element's
            // boundary text children.
            // Trim trailing only if the following sibling supplies no space
            // (or doesn't exist) — but since no dedup occurred, the rule is:
            // trim trailing when the following sibling provides a space OR
            // doesn't exist (i.e., trailing space is not the only separator).
            trim_inline_leading(&mut children[i]);
            // Trim trailing when there is no following sibling, or the
            // following sibling already starts with a space.
            let should_trim_trailing =
                i + 1 >= n || matches!(&children[i + 1], Node::Text(t) if t.value.starts_with(' '));
            if should_trim_trailing {
                trim_inline_trailing(&mut children[i]);
            }
        }
    }

    // Remove text nodes that became empty after trimming.
    children.retain(|child| !is_empty_text(child));
}

/// Return true if `node` is a Link or Delete.
fn is_link_or_delete(node: &Node) -> bool {
    matches!(node, Node::Link(_) | Node::Delete(_))
}

/// Return true if the last text descendant of an inline node ends with ' '.
fn inline_last_text_ends_with_space(node: &Node) -> bool {
    let children = match node.children() {
        Some(c) => c,
        None => return false,
    };
    match children.last() {
        Some(Node::Text(t)) => t.value.ends_with(' '),
        _ => false,
    }
}

/// Trim leading whitespace from the first text child of an inline node.
fn trim_inline_leading(node: &mut Node) {
    if let Some(children) = node.children_mut() {
        if let Some(Node::Text(ref mut t)) = children.first_mut() {
            let trimmed_len = t.value.trim_start_matches(' ').len();
            if trimmed_len != t.value.len() {
                let start = t.value.len() - trimmed_len;
                t.value.drain(..start);
            }
        }
    }
}

/// Trim trailing whitespace from the last text child of an inline node.
fn trim_inline_trailing(node: &mut Node) {
    if let Some(children) = node.children_mut() {
        if let Some(Node::Text(ref mut t)) = children.last_mut() {
            let trimmed_len = t.value.trim_end_matches(' ').len();
            if trimmed_len != t.value.len() {
                t.value.truncate(trimmed_len);
            }
        }
    }
}

/// Merge adjacent Text nodes into a single node.
fn merge_adjacent_text(children: &mut Vec<Node>) {
    let mut i = 0;
    while i + 1 < children.len() {
        if is_text(&children[i]) && is_text(&children[i + 1]) {
            if let Node::Text(next) = children.remove(i + 1) {
                if let Node::Text(ref mut current) = children[i] {
                    current.value.push_str(&next.value);
                }
            }
        } else {
            i += 1;
        }
    }
}

/// Trim leading/trailing whitespace (including newlines) from the first and
/// last text children. Newlines can appear at the edges of paragraph content
/// when `newlines: true` mode preserves whitespace that HTML puts between
/// block elements (e.g. blank lines between `<p>` tags become trailing `\n\n`
/// in the text of the preceding paragraph).
fn trim_container(children: &mut [Node]) {
    if let Some(Node::Text(ref mut first)) = children.first_mut() {
        let trimmed_len = first
            .value
            .trim_start_matches([' ', '\t', '\n', '\r'])
            .len();
        if trimmed_len != first.value.len() {
            let start = first.value.len() - trimmed_len;
            first.value.drain(..start);
        }
    }
    if let Some(Node::Text(ref mut last)) = children.last_mut() {
        let trimmed_len = last.value.trim_end_matches([' ', '\t', '\n', '\r']).len();
        if trimmed_len != last.value.len() {
            last.value.truncate(trimmed_len);
        }
    }
}

fn is_text(node: &Node) -> bool {
    matches!(node, Node::Text(_))
}

fn is_empty_text(node: &Node) -> bool {
    matches!(node, Node::Text(t) if t.value.is_empty())
}
