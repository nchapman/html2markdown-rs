// Implicit paragraph detection and block-in-inline resolution.
//
// Port of hast-util-to-mdast/lib/util/wrap.js.
// When a flow container has mixed phrasing + block children, phrasing runs
// are wrapped in implicit Paragraph nodes. Straddling elements (links, deletes
// containing block content) are split.

use super::util::{drop_surrounding_breaks, is_whitespace_only};
use crate::mdast::{self, Node};

/// Wrap mixed content: phrasing runs become paragraphs, block content passes through.
/// Port of hast-util-to-mdast/lib/util/wrap.js `wrap()`.
/// NOTE: Unlike our early `wrap_needed` guard, the JS reference ALWAYS wraps phrasing
/// runs into paragraphs (unless all-whitespace). Do not short-circuit here.
pub(crate) fn wrap(nodes: Vec<Node>) -> Vec<Node> {
    let nodes = flatten(nodes);
    let mut result = Vec::new();
    let mut phrasing_run: Vec<Node> = Vec::new();

    for node in nodes {
        if node.is_phrasing() {
            phrasing_run.push(node);
        } else {
            if !phrasing_run.is_empty() {
                let run = std::mem::take(&mut phrasing_run);
                let run = drop_surrounding_breaks(run);
                if !is_whitespace_only(&run) {
                    result.push(Node::Paragraph(mdast::Paragraph { children: run }));
                }
            }
            result.push(node);
        }
    }

    // Flush trailing phrasing run.
    if !phrasing_run.is_empty() {
        let run = drop_surrounding_breaks(phrasing_run);
        if !is_whitespace_only(&run) {
            result.push(Node::Paragraph(mdast::Paragraph { children: run }));
        }
    }

    result
}

/// Check whether any node in the list is non-phrasing (i.e., needs wrapping).
pub(crate) fn wrap_needed(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| {
        if !node.is_phrasing() {
            return true;
        }
        if let Some(children) = node.children() {
            wrap_needed(children)
        } else {
            false
        }
    })
}

/// Flatten straddling elements: links and deletes containing block content
/// get split so the inline wrapper distributes around each block child.
/// Port of hast-util-to-mdast/lib/util/wrap.js `flatten()`.
fn flatten(nodes: Vec<Node>) -> Vec<Node> {
    let mut result = Vec::new();
    for node in nodes {
        match &node {
            Node::Link(_) | Node::Delete(_) => {
                if let Some(children) = node.children() {
                    if wrap_needed(children) {
                        let mut split = split_straddling(node);
                        result.append(&mut split);
                        continue;
                    }
                }
                result.push(node);
            }
            _ => result.push(node),
        }
    }
    result
}

/// A template for recreating Link/Delete wrappers without cloning children.
enum WrapperTemplate {
    Link { url: String, title: Option<String> },
    Delete,
}

impl WrapperTemplate {
    fn wrap(&self, children: Vec<Node>) -> Node {
        match self {
            WrapperTemplate::Link { url, title } => Node::Link(mdast::Link {
                url: url.clone(),
                title: title.clone(),
                children,
            }),
            WrapperTemplate::Delete => Node::Delete(mdast::Delete { children }),
        }
    }
}

/// Split a straddling node (Link or Delete containing block content) into
/// multiple nodes where the inline wrapper distributes around blocks.
/// Port of hast-util-to-mdast/lib/util/wrap.js `split()`.
fn split_straddling(node: Node) -> Vec<Node> {
    // Destructure to take ownership of children without cloning.
    let (template, children) = match node {
        Node::Link(l) => (
            WrapperTemplate::Link {
                url: l.url,
                title: l.title,
            },
            l.children,
        ),
        Node::Delete(d) => (WrapperTemplate::Delete, d.children),
        _ => return vec![node],
    };

    let mut result: Vec<Node> = Vec::new();
    let mut phrasing_run: Vec<Node> = Vec::new();

    for child in flatten(children) {
        if child.is_phrasing() {
            phrasing_run.push(child);
        } else {
            if !phrasing_run.is_empty() {
                let run = std::mem::take(&mut phrasing_run);
                if !is_whitespace_only(&run) {
                    result.push(template.wrap(run));
                }
            }
            let new_node = wrap_parent_inside_child(&template, child);
            result.push(new_node);
        }
    }

    if !phrasing_run.is_empty() && !is_whitespace_only(&phrasing_run) {
        result.push(template.wrap(phrasing_run));
    }

    result
}

/// Place the `parent` (without its original children) as a wrapper inside `child`.
/// If `child` has children, the parent wraps the child's content.
fn wrap_parent_inside_child(template: &WrapperTemplate, child: Node) -> Node {
    // Destructure child to take ownership of its children without cloning.
    match child {
        Node::Heading(h) => {
            let inner = template.wrap(h.children);
            Node::Heading(mdast::Heading {
                depth: h.depth,
                children: vec![inner],
            })
        }
        Node::Paragraph(p) => {
            let inner = template.wrap(p.children);
            Node::Paragraph(mdast::Paragraph {
                children: vec![inner],
            })
        }
        Node::Blockquote(bq) => {
            let inner = template.wrap(bq.children);
            Node::Blockquote(mdast::Blockquote {
                children: vec![inner],
            })
        }
        other => other,
    }
}
