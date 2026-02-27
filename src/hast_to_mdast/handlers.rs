// Element handlers — one function per HTML element (or element group).
//
// Port of hast-util-to-mdast/lib/handlers/.
// Each handler takes an html5ever node and returns zero or more MDAST nodes.
// Handlers only produce tree nodes — no string formatting happens here.

use markup5ever_rcdom::{Handle, NodeData};

use super::State;
use crate::mdast;

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Convert all children of an HTML node to MDAST nodes.
pub(crate) fn all(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = handle.children.borrow();
    let mut result = Vec::new();
    for child in children.iter() {
        let mut nodes = one(state, child);
        result.append(&mut nodes);
    }
    result
}

/// Convert a single HTML node to MDAST node(s).
pub(crate) fn one(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    match &handle.data {
        NodeData::Text { ref contents } => {
            let text = contents.borrow().to_string();
            if text.is_empty() {
                vec![]
            } else {
                vec![mdast::Node::Text(mdast::Text { value: text })]
            }
        }
        NodeData::Comment { ref contents } => {
            let value = format!("<!--{}-->", contents);
            vec![mdast::Node::Html(mdast::Html { value })]
        }
        NodeData::Element { ref name, .. } => {
            let tag = name.local.as_ref();
            dispatch_element(state, handle, tag)
        }
        NodeData::Document => all(state, handle),
        _ => vec![],
    }
}

/// Route an element to its handler based on tag name.
fn dispatch_element(state: &mut State, handle: &Handle, tag: &str) -> Vec<mdast::Node> {
    match tag {
        // Ignore — return nothing
        "applet" | "area" | "basefont" | "bgsound" | "caption" | "col" | "colgroup"
        | "command" | "content" | "datalist" | "dialog" | "element" | "embed" | "frame"
        | "frameset" | "isindex" | "keygen" | "link" | "math" | "menu" | "menuitem"
        | "meta" | "nextid" | "noembed" | "noframes" | "optgroup" | "option" | "param"
        | "script" | "shadow" | "source" | "spacer" | "style" | "svg" | "template"
        | "title" | "track" => vec![],

        // Pass-through — recurse into children, no wrapping
        "abbr" | "acronym" | "bdi" | "bdo" | "big" | "blink" | "button" | "canvas"
        | "cite" | "data" | "details" | "dfn" | "font" | "ins" | "label" | "map"
        | "marquee" | "meter" | "nobr" | "noscript" | "object" | "output" | "progress"
        | "rb" | "rbc" | "rp" | "rt" | "rtc" | "ruby" | "slot" | "small" | "span"
        | "sup" | "sub" | "tbody" | "tfoot" | "thead" | "time" => all(state, handle),

        // Flow wrappers — children wrapped as flow content
        "address" | "article" | "aside" | "body" | "center" | "div" | "fieldset"
        | "figcaption" | "figure" | "form" | "footer" | "header" | "hgroup" | "html"
        | "legend" | "main" | "multicol" | "nav" | "picture" | "section" => {
            let children = all(state, handle);
            super::wrap::wrap(children)
        }

        // Element-specific handlers
        // TODO: implement each handler
        "a" | "blockquote" | "br" | "code" | "kbd" | "samp" | "tt" | "var" | "pre"
        | "listing" | "xmp" | "plaintext" | "del" | "s" | "strike" | "dl" | "em" | "i"
        | "mark" | "u" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "hr" | "iframe"
        | "img" | "image" | "input" | "li" | "dt" | "dd" | "ol" | "ul" | "dir" | "audio"
        | "video" | "p" | "summary" | "q" | "select" | "strong" | "b" | "table" | "td"
        | "th" | "tr" | "textarea" | "wbr" | "base" => {
            // Temporary: fall through to children until handlers are implemented
            all(state, handle)
        }

        // Unknown elements — recurse into children
        _ => all(state, handle),
    }
}

// ---------------------------------------------------------------------------
// Attribute helpers
// ---------------------------------------------------------------------------

/// Get the value of an attribute on an element node.
pub(crate) fn get_attr(handle: &Handle, name: &str) -> Option<String> {
    if let NodeData::Element { ref attrs, .. } = handle.data {
        for attr in attrs.borrow().iter() {
            if attr.name.local.as_ref() == name {
                return Some(attr.value.to_string());
            }
        }
    }
    None
}

/// Get the tag name of an element node.
pub(crate) fn tag_name(handle: &Handle) -> Option<String> {
    if let NodeData::Element { ref name, .. } = handle.data {
        Some(name.local.as_ref().to_string())
    } else {
        None
    }
}
