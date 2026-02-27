// Context-sensitive escaping for Markdown serialization.
//
// Port of mdast-util-to-markdown/lib/unsafe.js and lib/util/safe.js.
// Escapes Markdown syntax characters in text content to prevent unintended
// formatting. Implements the subset of unsafe patterns needed for phrasing content.

use std::sync::LazyLock;

use regex::Regex;

/// Escape special Markdown characters in phrasing (inline) text content.
///
/// In phrasing context, these characters can trigger Markdown constructs:
/// - `\` → `\\` (backslash escape prefix)
/// - `[` → `\[` (can start link or image reference)
/// - `_` → `\_` (can start emphasis or strong)
/// - `*` → `\*` (can start emphasis or strong)
/// - `` ` `` → `` \` `` (can start code span)
/// - `<` → `\<` (can start autolink or inline HTML)
/// - `!` before `[` → `\!` (can start image)
///
/// Port of mdast-util-to-markdown's `safe()` function for phrasing context.
/// Note: `]` is intentionally NOT escaped here — a standalone `]` without a
/// preceding `[` is harmless, and escaping it breaks task-list checkbox syntax
/// (`\[ ]`, `\[x]`) produced by the list-item serializer.
pub(crate) fn escape_phrasing(text: &str) -> String {
    // These patterns are based on the `unsafe` array in mdast-util-to-markdown/lib/unsafe.js:
    // - {character: '[', inConstruct: 'phrasing'} — can start links/images
    // - {character: '_', inConstruct: 'phrasing'} — can start emphasis/strong
    // - {character: '*', inConstruct: 'phrasing'} — can start emphasis/strong
    // - {character: '`', inConstruct: 'phrasing'} — can start code span
    // - {character: '<', inConstruct: 'phrasing'} — can start autolink/HTML

    static NEEDS_ESCAPE: LazyLock<Regex> = LazyLock::new(|| {
        // Characters that need escaping in phrasing content.
        // `\` must come first to avoid double-escaping.
        Regex::new(r"[\\`*_\[!&<]").unwrap()
    });

    // Fast path: no special characters.
    if !NEEDS_ESCAPE.is_match(text) {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len() + 8);
    let mut last = 0;
    let bytes = text.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        let escape = match b {
            b'\\' => true,
            b'[' => true,
            b'_' => true,
            b'*' => true,
            b'`' => true,
            b'<' => true,
            // `!` only needs escaping before `[` (potential image)
            b'!' => bytes.get(i + 1) == Some(&b'['),
            // `&` before alphanumeric or `#` (character reference)
            b'&' => matches!(bytes.get(i + 1), Some(b'#') | Some(b'A'..=b'Z') | Some(b'a'..=b'z')),
            _ => false,
        };

        if escape {
            result.push_str(&text[last..i]);
            result.push('\\');
            last = i;
        }
    }

    result.push_str(&text[last..]);
    result
}

/// Escape special Markdown characters in link text (the `[…]` part of a link).
///
/// Same as `escape_phrasing` but also escapes `]`, which prematurely closes
/// the link text bracket. We don't escape `]` globally in phrasing because
/// standalone `]` is harmless outside link context and escaping it breaks
/// task-list checkbox syntax (`\[ ]`, `\[x]`) produced by the list handler.
pub(crate) fn escape_link_text(text: &str) -> String {
    static NEEDS_ESCAPE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[\\`*_\[\]!&<]").unwrap()
    });

    if !NEEDS_ESCAPE.is_match(text) {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len() + 8);
    let mut last = 0;
    let bytes = text.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        let escape = match b {
            b'\\' => true,
            b'[' => true,
            b']' => true,
            b'_' => true,
            b'*' => true,
            b'`' => true,
            b'<' => true,
            b'!' => bytes.get(i + 1) == Some(&b'['),
            b'&' => matches!(bytes.get(i + 1), Some(b'#') | Some(b'A'..=b'Z') | Some(b'a'..=b'z')),
            _ => false,
        };

        if escape {
            result.push_str(&text[last..i]);
            result.push('\\');
            last = i;
        }
    }

    result.push_str(&text[last..]);
    result
}

/// Escape a character at the start of a block if it would trigger a Markdown construct.
///
/// Port of the `atBreak` patterns in mdast-util-to-markdown/lib/unsafe.js.
/// Returns the escaped version of content whose first character is at a line break.
pub(crate) fn escape_at_break_start(content: String) -> String {
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return content;
    }

    // Check if the first character needs escaping based on what follows it.
    let first = bytes[0];
    let second = bytes.get(1).copied();

    let needs_escape = match first {
        // `#` → always (could start ATX heading)
        b'#' => true,
        // `>` → always (blockquote)
        b'>' => true,
        // `*` → when followed by [ \t\r\n*]
        b'*' => second.map_or(true, |c| matches!(c, b' ' | b'\t' | b'\r' | b'\n' | b'*')),
        // `+` → when followed by [ \t\r\n]
        b'+' => second.map_or(true, |c| matches!(c, b' ' | b'\t' | b'\r' | b'\n')),
        // `-` → when followed by [ \t\r\n-]
        b'-' => second.map_or(true, |c| matches!(c, b' ' | b'\t' | b'\r' | b'\n' | b'-')),
        // `=` → when followed by [ \t] or end of string
        b'=' => second.map_or(true, |c| matches!(c, b' ' | b'\t')),
        // `_` → when followed by _
        b'_' => second == Some(b'_'),
        // `` ` `` → when followed by `` ` ``
        b'`' => second == Some(b'`'),
        // `~` → when followed by `~`
        b'~' => second == Some(b'~'),
        _ => false,
    };

    if needs_escape {
        format!("\\{}", content)
    } else {
        content
    }
}
