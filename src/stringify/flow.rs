// Block-level container serialization.
//
// Port of mdast-util-to-markdown/lib/util/container-flow.js.
// Serializes block children separated by blank lines, consulting join rules
// to determine spacing between adjacent nodes.

use super::State;
use crate::mdast::Node;

/// Serialize a list of block-level (flow) children with blank lines between them.
/// Used for root, blockquote, and similar containers.
/// Port of mdast-util-to-markdown/lib/util/container-flow.js.
pub(crate) fn container_flow(state: &mut State, children: &[Node]) -> String {
    let mut result = String::new();

    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            result.push_str("\n\n");
        }
        let content = super::handlers::handle(state, child);
        result.push_str(&content);

        // Reset bullet_last_used after any non-list node so sibling lists
        // don't unnecessarily alternate bullets (port of JS containerFlow behavior:
        // `if (child.type !== 'list') state.bulletLastUsed = undefined`).
        if !matches!(child, Node::List(_)) {
            state.bullet_last_used = None;
        }
    }

    result
}

/// Serialize block-level children for a list item, respecting tight/spread.
/// `spread` = true → blank line between children, false → single newline.
pub(crate) fn container_flow_tight(state: &mut State, children: &[Node], spread: bool) -> String {
    let mut result = String::new();

    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            if spread {
                result.push_str("\n\n");
            } else {
                result.push('\n');
            }
        }
        let content = super::handlers::handle(state, child);
        result.push_str(&content);
    }

    result
}
