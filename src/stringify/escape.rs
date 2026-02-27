// Context-sensitive escaping for Markdown serialization.
//
// Port of mdast-util-to-markdown/lib/unsafe.js and lib/util/safe.js.
// Only escapes Markdown syntax characters when they would actually trigger
// formatting in the current context.
//
// TODO: Implement unsafe patterns and the safe() function.
// See also: refs/html-to-markdown/ESCAPING.md for edge case documentation.
