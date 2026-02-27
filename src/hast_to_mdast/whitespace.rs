// Whitespace normalization for MDAST trees.
//
// Post-processing pass that merges adjacent text nodes, collapses whitespace
// around line endings, and trims leading/trailing whitespace in headings,
// paragraphs, and root nodes.

use crate::mdast::Node;

/// Run whitespace post-processing on an MDAST tree.
pub(crate) fn post_process_whitespace(node: &mut Node) {
    // Recursively process children first.
    if let Some(children) = node.children_mut() {
        for child in children.iter_mut() {
            post_process_whitespace(child);
        }

        // Merge adjacent text nodes.
        merge_adjacent_text(children);

        // Remove empty text nodes.
        children.retain(|child| !is_empty_text(child));
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
        first.value = first.value.trim_start_matches([' ', '\t', '\n', '\r']).to_string();
    }
    if let Some(Node::Text(ref mut last)) = children.last_mut() {
        last.value = last.value.trim_end_matches([' ', '\t', '\n', '\r']).to_string();
    }
}

fn is_text(node: &Node) -> bool {
    matches!(node, Node::Text(_))
}

fn is_empty_text(node: &Node) -> bool {
    matches!(node, Node::Text(t) if t.value.is_empty())
}
