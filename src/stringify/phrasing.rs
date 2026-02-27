// Inline container serialization.
//
// Port of mdast-util-to-markdown/lib/util/container-phrasing.js.
// Serializes inline children flush together, using peek() on the next
// sibling's handler to determine the `after` context for escaping.

use super::State;
use crate::mdast::Node;

/// Serialize a list of inline (phrasing) children.
pub(crate) fn container_phrasing(state: &mut State, children: &[Node]) -> String {
    let mut result = String::new();

    for child in children {
        let content = super::handlers::handle(state, child);
        result.push_str(&content);
    }

    result
}
