// Block-level container serialization.
//
// Port of mdast-util-to-markdown/lib/util/container-flow.js.
// Serializes block children separated by blank lines, consulting join rules
// to determine spacing between adjacent nodes.

use super::State;
use crate::mdast::Node;

/// Serialize a list of block-level (flow) children with appropriate spacing.
pub(crate) fn container_flow(state: &mut State, children: &[Node]) -> String {
    let mut result = String::new();

    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            let separator = between(state, &children[i - 1], child);
            result.push_str(&separator);
        }
        let content = super::handlers::handle(state, child);
        result.push_str(&content);
    }

    result
}

/// Determine the separator between two adjacent flow nodes.
fn between(_state: &State, _previous: &Node, _current: &Node) -> String {
    // TODO: Implement join rules for context-specific spacing.
    // Default: blank line between block elements.
    "\n\n".to_string()
}
