// Shared utilities for the hast_to_mdast module.

use crate::mdast;

/// Remove leading and trailing Break nodes and whitespace-only Text nodes.
/// Port of hast-util-to-mdast/lib/util/drop-surrounding-breaks.js
pub(crate) fn drop_surrounding_breaks(mut nodes: Vec<mdast::Node>) -> Vec<mdast::Node> {
    fn is_droppable_edge(n: &mdast::Node) -> bool {
        matches!(n, mdast::Node::Break(_))
            || matches!(n, mdast::Node::Text(t) if t.value.trim().is_empty())
    }

    // Find the first non-droppable node.
    let start = nodes
        .iter()
        .position(|n| !is_droppable_edge(n))
        .unwrap_or(nodes.len());
    if start > 0 {
        nodes.drain(..start);
    }

    // Find the last non-droppable node.
    while nodes.last().is_some_and(is_droppable_edge) {
        nodes.pop();
    }

    nodes
}

/// Check if a list of nodes contains only whitespace-only text.
pub(crate) fn is_whitespace_only(nodes: &[mdast::Node]) -> bool {
    nodes.iter().all(|n| match n {
        mdast::Node::Text(t) => t.value.trim().is_empty(),
        _ => false,
    })
}
