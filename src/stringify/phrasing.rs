// Inline container serialization.
//
// Port of mdast-util-to-markdown/lib/util/container-phrasing.js.
// Serializes inline children flush together, using peek() on the next
// sibling's handler to determine the `after` context for escaping.

use super::State;
use crate::mdast::Node;

/// Serialize a list of inline (phrasing) children.
pub(crate) fn container_phrasing(state: &mut State, children: &[Node]) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(children.len());

    for child in children {
        parts.push(super::handlers::handle(state, child));
    }

    // Trim whitespace adjacent to hard breaks ("\\\n"):
    // trailing spaces before the break, leading spaces after it.
    // This matches behavior of the JS reference which normalizes whitespace
    // around <br> during the hast→mdast transformation.
    for i in 0..parts.len() {
        if parts[i] == "\\\n" {
            if i > 0 {
                let prev = parts[i - 1].trim_end_matches(' ').to_string();
                parts[i - 1] = prev;
            }
            if i + 1 < parts.len() {
                let next = parts[i + 1].trim_start_matches(' ').to_string();
                parts[i + 1] = next;
            }
        }
    }

    // Port of mdast-util-to-markdown unsafe entry:
    //   {character: '!', after: /\[/, inConstruct: 'phrasing'}
    // When a part ends with an unescaped `!` and the following part starts
    // with `[` (i.e., a link/image), escape the `!` as `\!` to prevent
    // `![text](url)` from being interpreted as image syntax.
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i + 1].starts_with('[') && parts[i].ends_with('!') && !parts[i].ends_with("\\!") {
            let len = parts[i].len();
            parts[i].truncate(len - 1);
            parts[i].push_str("\\!");
        }
    }

    parts.join("")
}
