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
                let trimmed_len = parts[i - 1].trim_end_matches(' ').len();
                parts[i - 1].truncate(trimmed_len);
            }
            if i + 1 < parts.len() {
                let trimmed = parts[i + 1].trim_start_matches(' ');
                if trimmed.len() != parts[i + 1].len() {
                    let start = parts[i + 1].len() - trimmed.len();
                    parts[i + 1].drain(..start);
                }
            }
        }
    }

    // Port of mdast-util-to-markdown unsafe entry:
    //   {character: '!', after: /\[/, inConstruct: 'phrasing'}
    // When a part ends with an unescaped `!` and the following part starts
    // with `[` (i.e., a link/image), escape the `!` as `\!` to prevent
    // `![text](url)` from being interpreted as image syntax.
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i + 1].starts_with('[') && ends_with_unescaped(&parts[i], b'!') {
            let len = parts[i].len();
            parts[i].truncate(len - 1);
            parts[i].push_str("\\!");
        }
    }

    // Port of mdast-util-to-markdown unsafe entry:
    //   {character: ']', after: /\(/, inConstruct: 'phrasing'}
    // When a part ends with `]` and the next starts with `(`, it looks like
    // `](` — accidental link syntax. Escape the `]` as `\]`.
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i + 1].starts_with('(') && ends_with_unescaped(&parts[i], b']') {
            let len = parts[i].len();
            parts[i].truncate(len - 1);
            parts[i].push_str("\\]");
        }
    }

    parts.join("")
}

/// Check whether a string ends with a given ASCII byte that is not preceded
/// by an odd number of backslashes (i.e., the character is unescaped).
fn ends_with_unescaped(s: &str, ch: u8) -> bool {
    let bytes = s.as_bytes();
    if bytes.last() != Some(&ch) {
        return false;
    }
    // Count consecutive backslashes before the final character.
    let backslashes = bytes[..bytes.len() - 1]
        .iter()
        .rev()
        .take_while(|&&b| b == b'\\')
        .count();
    // Even number of backslashes (including zero) means the char is unescaped.
    backslashes % 2 == 0
}
