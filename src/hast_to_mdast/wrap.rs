// Implicit paragraph detection and block-in-inline resolution.
//
// Port of hast-util-to-mdast/lib/util/wrap.js.
// When a flow container has mixed phrasing + block children, phrasing runs
// are wrapped in implicit Paragraph nodes. Straddling elements (links, deletes
// containing block content) are split.

use crate::mdast::{self, Node};

/// Wrap mixed content: phrasing runs become paragraphs, block content passes through.
pub(crate) fn wrap(nodes: Vec<Node>) -> Vec<Node> {
    if !wrap_needed(&nodes) {
        return nodes;
    }

    let nodes = flatten(nodes);
    let mut result = Vec::new();
    let mut phrasing_run: Vec<Node> = Vec::new();

    for node in nodes {
        if node.is_phrasing() {
            phrasing_run.push(node);
        } else {
            if !phrasing_run.is_empty() {
                let run = std::mem::take(&mut phrasing_run);
                if !is_whitespace_only(&run) {
                    result.push(Node::Paragraph(mdast::Paragraph { children: run }));
                }
            }
            result.push(node);
        }
    }

    // Flush trailing phrasing run.
    if !phrasing_run.is_empty() && !is_whitespace_only(&phrasing_run) {
        result.push(Node::Paragraph(mdast::Paragraph {
            children: phrasing_run,
        }));
    }

    result
}

/// Check whether any node in the list is non-phrasing (i.e., needs wrapping).
fn wrap_needed(nodes: &[Node]) -> bool {
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

/// Split a straddling node (Link or Delete containing block content) into
/// multiple nodes where the inline wrapper distributes around blocks.
fn split_straddling(node: Node) -> Vec<Node> {
    // TODO: Implement full straddling logic.
    // For now, just return the node's children as a fallback.
    node.children()
        .map(|c| c.to_vec())
        .unwrap_or_else(|| vec![node])
}

/// Check if a list of nodes contains only whitespace text.
fn is_whitespace_only(nodes: &[Node]) -> bool {
    nodes.iter().all(|node| match node {
        Node::Text(t) => t.value.trim().is_empty(),
        _ => false,
    })
}
