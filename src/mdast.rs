// MDAST node types — based on https://github.com/syntax-tree/mdast
//
// ~25 node types representing the Markdown abstract syntax tree.
// Each node is a variant of the `Node` enum. Parent nodes own their children.
// Leaf nodes hold a `value: String`.

/// Alignment of a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignKind {
    Left,
    Right,
    Center,
}

/// How a reference (link or image) is written in Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    /// `[text]` — identifier inferred from content.
    Shortcut,
    /// `[text][]` — explicit empty brackets.
    Collapsed,
    /// `[text][id]` — explicit identifier.
    Full,
}

// ---------------------------------------------------------------------------
// Node structs
// ---------------------------------------------------------------------------

/// Document root.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Root {
    pub children: Vec<Node>,
}

/// Block quote (`> ...`).
#[derive(Debug, Clone, PartialEq)]
pub struct Blockquote {
    pub children: Vec<Node>,
}

/// Fenced or indented code block.
#[derive(Debug, Clone, PartialEq)]
pub struct Code {
    pub value: String,
    pub lang: Option<String>,
    pub meta: Option<String>,
}

/// ATX or setext heading.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub depth: u8, // 1–6
    pub children: Vec<Node>,
}

/// Raw HTML.
#[derive(Debug, Clone, PartialEq)]
pub struct Html {
    pub value: String,
}

/// Ordered or unordered list.
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub ordered: bool,
    pub start: Option<u32>,
    pub spread: bool,
    pub children: Vec<Node>,
}

/// Item inside a list.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub spread: bool,
    pub checked: Option<bool>,
    pub children: Vec<Node>,
}

/// Thematic break (`***`, `---`, `___`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThematicBreak;

/// Link reference definition (`[label]: url "title"`).
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub identifier: String,
    pub label: Option<String>,
    pub url: String,
    pub title: Option<String>,
}

/// Paragraph.
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub children: Vec<Node>,
}

/// Plain text.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub value: String,
}

/// Emphasis (`*text*` or `_text_`).
#[derive(Debug, Clone, PartialEq)]
pub struct Emphasis {
    pub children: Vec<Node>,
}

/// Strong emphasis (`**text**` or `__text__`).
#[derive(Debug, Clone, PartialEq)]
pub struct Strong {
    pub children: Vec<Node>,
}

/// Inline code (`` `code` ``).
#[derive(Debug, Clone, PartialEq)]
pub struct InlineCode {
    pub value: String,
}

/// Hard line break (`\` or two spaces at end of line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Break;

/// Hyperlink (`[text](url "title")`).
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub url: String,
    pub title: Option<String>,
    pub children: Vec<Node>,
}

/// Image (`![alt](url "title")`).
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub url: String,
    pub title: Option<String>,
    pub alt: String,
}

/// Link via reference (`[text][id]`).
#[derive(Debug, Clone, PartialEq)]
pub struct LinkReference {
    pub identifier: String,
    pub label: Option<String>,
    pub reference_kind: ReferenceKind,
    pub children: Vec<Node>,
}

/// Image via reference (`![alt][id]`).
#[derive(Debug, Clone, PartialEq)]
pub struct ImageReference {
    pub identifier: String,
    pub label: Option<String>,
    pub reference_kind: ReferenceKind,
    pub alt: String,
}

// GFM extensions ---------------------------------------------------------

/// Strikethrough (`~~text~~`).
#[derive(Debug, Clone, PartialEq)]
pub struct Delete {
    pub children: Vec<Node>,
}

/// GFM table.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub align: Vec<Option<AlignKind>>,
    pub children: Vec<Node>, // TableRow
}

/// Row in a GFM table.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub children: Vec<Node>, // TableCell
}

/// Cell in a GFM table row.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub children: Vec<Node>,
    /// Column span (from HTML colspan attribute); used during transformation, not serialization.
    #[doc(hidden)]
    pub colspan: Option<u32>,
    /// Row span (from HTML rowspan attribute); used during transformation, not serialization.
    #[doc(hidden)]
    pub rowspan: Option<u32>,
}

/// Footnote definition (`[^id]: ...`).
#[derive(Debug, Clone, PartialEq)]
pub struct FootnoteDefinition {
    pub identifier: String,
    pub label: Option<String>,
    pub children: Vec<Node>,
}

/// Footnote reference (`[^id]`).
#[derive(Debug, Clone, PartialEq)]
pub struct FootnoteReference {
    pub identifier: String,
    pub label: Option<String>,
}

// Frontmatter ------------------------------------------------------------

/// YAML frontmatter block.
#[derive(Debug, Clone, PartialEq)]
pub struct Yaml {
    pub value: String,
}

// ---------------------------------------------------------------------------
// Node enum
// ---------------------------------------------------------------------------

/// A node in the Markdown abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // Document
    Root(Root),

    // Flow (block) content
    Blockquote(Blockquote),
    Code(Code),
    Heading(Heading),
    Html(Html),
    List(List),
    ListItem(ListItem),
    ThematicBreak(ThematicBreak),
    Definition(Definition),
    Paragraph(Paragraph),

    // Phrasing (inline) content
    Break(Break),
    Delete(Delete),
    Emphasis(Emphasis),
    Image(Image),
    ImageReference(ImageReference),
    InlineCode(InlineCode),
    Link(Link),
    LinkReference(LinkReference),
    Strong(Strong),
    Text(Text),

    // Table (GFM)
    Table(Table),
    TableRow(TableRow),
    TableCell(TableCell),

    // Footnotes (GFM)
    FootnoteDefinition(FootnoteDefinition),
    FootnoteReference(FootnoteReference),

    // Frontmatter
    Yaml(Yaml),
}

impl Node {
    /// Returns a reference to this node's children, if it has any.
    pub fn children(&self) -> Option<&[Node]> {
        match self {
            Node::Root(n) => Some(&n.children),
            Node::Blockquote(n) => Some(&n.children),
            Node::Heading(n) => Some(&n.children),
            Node::List(n) => Some(&n.children),
            Node::ListItem(n) => Some(&n.children),
            Node::Paragraph(n) => Some(&n.children),
            Node::Emphasis(n) => Some(&n.children),
            Node::Strong(n) => Some(&n.children),
            Node::Delete(n) => Some(&n.children),
            Node::Link(n) => Some(&n.children),
            Node::LinkReference(n) => Some(&n.children),
            Node::Table(n) => Some(&n.children),
            Node::TableRow(n) => Some(&n.children),
            Node::TableCell(n) => Some(&n.children),
            Node::FootnoteDefinition(n) => Some(&n.children),
            _ => None,
        }
    }

    /// Returns a mutable reference to this node's children, if it has any.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Node::Root(n) => Some(&mut n.children),
            Node::Blockquote(n) => Some(&mut n.children),
            Node::Heading(n) => Some(&mut n.children),
            Node::List(n) => Some(&mut n.children),
            Node::ListItem(n) => Some(&mut n.children),
            Node::Paragraph(n) => Some(&mut n.children),
            Node::Emphasis(n) => Some(&mut n.children),
            Node::Strong(n) => Some(&mut n.children),
            Node::Delete(n) => Some(&mut n.children),
            Node::Link(n) => Some(&mut n.children),
            Node::LinkReference(n) => Some(&mut n.children),
            Node::Table(n) => Some(&mut n.children),
            Node::TableRow(n) => Some(&mut n.children),
            Node::TableCell(n) => Some(&mut n.children),
            Node::FootnoteDefinition(n) => Some(&mut n.children),
            _ => None,
        }
    }

    /// Whether this node is phrasing (inline) content.
    ///
    /// Note: `Html` is flow content (block-level), not phrasing. HTML comments
    /// and raw HTML between block elements should be treated as block nodes.
    /// This matches mdast-util-phrasing behavior.
    pub fn is_phrasing(&self) -> bool {
        matches!(
            self,
            Node::Break(_)
                | Node::Delete(_)
                | Node::Emphasis(_)
                | Node::Image(_)
                | Node::ImageReference(_)
                | Node::InlineCode(_)
                | Node::Link(_)
                | Node::LinkReference(_)
                | Node::Strong(_)
                | Node::Text(_)
                | Node::FootnoteReference(_)
        )
    }

    /// Whether this node is flow (block) content.
    pub fn is_flow(&self) -> bool {
        matches!(
            self,
            Node::Blockquote(_)
                | Node::Code(_)
                | Node::Heading(_)
                | Node::Html(_)
                | Node::List(_)
                | Node::ThematicBreak(_)
                | Node::Definition(_)
                | Node::Paragraph(_)
                | Node::Table(_)
                | Node::FootnoteDefinition(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_is_phrasing() {
        let node = Node::Text(Text {
            value: "hello".into(),
        });
        assert!(node.is_phrasing());
        assert!(!node.is_flow());
    }

    #[test]
    fn test_paragraph_is_flow() {
        let node = Node::Paragraph(Paragraph {
            children: vec![Node::Text(Text {
                value: "hello".into(),
            })],
        });
        assert!(node.is_flow());
        assert!(!node.is_phrasing());
    }

    #[test]
    fn test_html_is_flow_not_phrasing() {
        let node = Node::Html(Html {
            value: "<!-- comment -->".into(),
        });
        assert!(!node.is_phrasing());
        assert!(node.is_flow());
    }

    #[test]
    fn test_children_access() {
        let node = Node::Paragraph(Paragraph {
            children: vec![Node::Text(Text {
                value: "hello".into(),
            })],
        });
        assert_eq!(node.children().unwrap().len(), 1);
    }

    #[test]
    fn test_leaf_has_no_children() {
        let node = Node::Text(Text {
            value: "hello".into(),
        });
        assert!(node.children().is_none());
    }

    #[test]
    fn test_root_default() {
        let root = Root::default();
        assert!(root.children.is_empty());
    }
}
