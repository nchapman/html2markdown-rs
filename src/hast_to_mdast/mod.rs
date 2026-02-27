// HTML tree → MDAST transform
//
// Port of hast-util-to-mdast (https://github.com/syntax-tree/hast-util-to-mdast).
// Parses HTML via html5ever and walks the resulting tree, dispatching each
// element to a handler that produces MDAST nodes.

pub(crate) mod handlers;
pub(crate) mod whitespace;
pub(crate) mod wrap;

use std::collections::HashMap;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::ParseOpts;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use url::Url;

use crate::mdast;

/// Options for the HTML → MDAST transformation.
#[derive(Debug, Clone, Default)]
pub struct TransformOptions {
    /// Whether to preserve newlines in whitespace normalization.
    pub newlines: bool,
    /// Symbol for checked checkboxes (default: "[x]").
    pub checked: Option<String>,
    /// Symbol for unchecked checkboxes (default: "[ ]").
    pub unchecked: Option<String>,
    /// Quote character pairs for `<q>` elements, in nesting order.
    /// Each string is 1 or 2 characters: open (and close if different).
    /// Default: `['"']` (ASCII double-quote).
    pub quotes: Vec<String>,
}


/// Transformation state threaded through all handlers.
pub(crate) struct State {
    /// Base URL from the first `<base>` element encountered.
    pub frozen_base_url: Option<Url>,
    /// Whether the first `<base>` element has been seen (regardless of href).
    /// Per HTML5 spec, only the first `<base>` element is effective.
    pub base_found: bool,
    /// Whether we're currently inside a table (nested tables → text).
    pub in_table: bool,
    /// Whether we're inside a whitespace-preserving element (<pre>, etc.).
    pub in_pre: bool,
    /// Nesting depth for `<q>` elements (cycles quote characters).
    pub q_nesting: usize,
    /// Elements indexed by their `id` attribute.
    pub element_by_id: HashMap<String, Handle>,
    /// Transform options.
    pub options: TransformOptions,
}

impl State {
    fn new(options: TransformOptions) -> Self {
        Self {
            frozen_base_url: None,
            base_found: false,
            in_table: false,
            in_pre: false,
            q_nesting: 0,
            element_by_id: HashMap::new(),
            options,
        }
    }

    /// Resolve a URL against the frozen base URL.
    pub fn resolve(&self, raw: &str) -> String {
        if raw.is_empty() {
            return String::new();
        }
        if let Some(base) = &self.frozen_base_url {
            if let Ok(resolved) = base.join(raw) {
                return resolved.to_string();
            }
        }
        raw.to_string()
    }
}

/// Parse an HTML string and transform it into an MDAST tree.
pub(crate) fn transform(html: &str, options: TransformOptions) -> mdast::Node {
    let dom = parse_html(html);
    let mut state = State::new(options);

    // Pre-pass: index elements by id.
    index_ids(&dom.document, &mut state.element_by_id);

    // Transform.
    let children = handlers::all(&mut state, &dom.document);
    let children = wrap::wrap(children);
    let mut root = mdast::Node::Root(mdast::Root { children });
    whitespace::post_process_whitespace(&mut root);

    root
}

/// Parse an HTML string into an html5ever RcDom.
pub(crate) fn parse_html(html: &str) -> RcDom {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    parse_document(RcDom::default(), opts)
        .from_utf8()
        .one(html.as_bytes())
}

/// Recursively index all elements by their `id` attribute.
fn index_ids(handle: &Handle, map: &mut HashMap<String, Handle>) {
    if let NodeData::Element { ref attrs, .. } = handle.data {
        for attr in attrs.borrow().iter() {
            if attr.name.local.as_ref() == "id" {
                let id = attr.value.to_string();
                if !id.is_empty() {
                    map.entry(id).or_insert_with(|| handle.clone());
                }
            }
        }
    }
    for child in handle.children.borrow().iter() {
        index_ids(child, map);
    }
}
