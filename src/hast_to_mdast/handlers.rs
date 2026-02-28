// Element handlers — one function per HTML element (or element group).
//
// Port of hast-util-to-mdast/lib/handlers/.
// Each handler takes an html5ever node and returns zero or more MDAST nodes.
// Handlers only produce tree nodes — no string formatting happens here.

use markup5ever_rcdom::{Handle, NodeData};

use super::util::{drop_surrounding_breaks, is_whitespace_only};
use super::State;
use crate::mdast;

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Convert all children of an HTML node to MDAST nodes.
pub(crate) fn all(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children_ref = handle.children.borrow();
    let mut result = Vec::new();
    for child in children_ref.iter() {
        let mut nodes = one(state, child);
        result.append(&mut nodes);
    }
    result
}

/// Convert a single HTML node to MDAST node(s).
pub(crate) fn one(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    match &handle.data {
        NodeData::Text { ref contents } => {
            let raw = contents.borrow().to_string();
            // Collapse whitespace unless we're in a <pre> context.
            // With `newlines: true`, newlines are preserved (only spaces/tabs collapse).
            let text = if state.in_pre {
                raw
            } else if state.options.newlines {
                collapse_whitespace_preserving_newlines(&raw)
            } else {
                collapse_whitespace(&raw)
            };
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
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let tag = name.local.as_ref();
            // data-mdast="ignore" suppresses the element and its subtree.
            if attrs
                .borrow()
                .iter()
                .any(|a| a.name.local.as_ref() == "data-mdast" && a.value.as_ref() == "ignore")
            {
                return vec![];
            }
            dispatch_element(state, handle, tag)
        }
        NodeData::Document => all(state, handle),
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Route an element to its handler based on tag name.
fn dispatch_element(state: &mut State, handle: &Handle, tag: &str) -> Vec<mdast::Node> {
    match tag {
        // Ignore — return nothing
        "applet" | "area" | "basefont" | "bgsound" | "caption" | "col" | "colgroup" | "command"
        | "content" | "datalist" | "dialog" | "element" | "embed" | "frame" | "frameset"
        | "isindex" | "keygen" | "link" | "math" | "menu" | "menuitem" | "meta" | "nextid"
        | "noembed" | "noframes" | "optgroup" | "option" | "param" | "script" | "shadow"
        | "source" | "spacer" | "style" | "svg" | "template" | "title" | "track" => vec![],

        // Pass-through — recurse into children, no wrapping.
        // Trim leading whitespace from the first text result: in HTML, the
        // leading whitespace of an inline transparent element is insignificant
        // because the gap before it is already provided by the preceding text
        // (mirrors rehype-minify-whitespace behaviour for inline elements).
        "abbr" | "acronym" | "bdi" | "bdo" | "big" | "blink" | "button" | "canvas" | "cite"
        | "data" | "details" | "dfn" | "font" | "ins" | "label" | "map" | "marquee" | "meter"
        | "nobr" | "object" | "output" | "progress" | "rb" | "rbc" | "rp" | "rt" | "rtc"
        | "ruby" | "slot" | "small" | "span" | "sup" | "sub" | "tbody" | "tfoot" | "thead"
        | "time" => {
            let mut nodes = all(state, handle);
            if let Some(mdast::Node::Text(ref mut t)) = nodes.first_mut() {
                t.value = t.value.trim_start_matches([' ', '\t']).to_string();
                if t.value.is_empty() {
                    nodes.remove(0);
                }
            }
            nodes
        }

        // noscript: html5ever parses its content as raw text when scripting is enabled.
        // Re-parse the text content as HTML and process it.
        "noscript" => handle_noscript(state, handle),

        // Flow wrappers — children wrapped as flow content
        "address" | "article" | "aside" | "body" | "center" | "div" | "fieldset" | "figcaption"
        | "figure" | "form" | "footer" | "header" | "hgroup" | "html" | "legend" | "main"
        | "multicol" | "nav" | "picture" | "section" => {
            let children = all(state, handle);
            super::wrap::wrap(children)
        }

        // Element-specific handlers
        "a" => handle_a(state, handle),
        "base" => handle_base(state, handle),
        "blockquote" => handle_blockquote(state, handle),
        "br" => handle_br(),
        "code" | "kbd" | "samp" | "tt" | "var" => handle_code_inline(state, handle),
        "pre" | "listing" | "xmp" | "plaintext" => handle_code_block(state, handle),
        "del" | "s" | "strike" => handle_del(state, handle),
        "dl" => handle_dl(state, handle),
        "em" | "i" | "mark" | "u" => handle_em(state, handle),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => handle_heading(state, handle, tag),
        "hr" => handle_hr(),
        "iframe" => handle_iframe(state, handle),
        "img" | "image" => handle_img(handle),
        "input" => handle_input(state, handle),
        "li" | "dt" | "dd" => handle_li(state, handle),
        "ol" | "ul" | "dir" => handle_list(state, handle, tag),
        "audio" | "video" => handle_media(state, handle, tag),
        "p" | "summary" => handle_p(state, handle),
        "q" => handle_q(state, handle),
        "select" => handle_select(state, handle),
        "strong" | "b" => handle_strong(state, handle),
        "table" => handle_table(state, handle),
        "td" | "th" => handle_table_cell(state, handle),
        "tr" => handle_table_row(state, handle),
        "textarea" => handle_textarea(state, handle),
        "wbr" => handle_wbr(),

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

/// Check whether an attribute exists on an element node (avoids String allocation).
pub(crate) fn has_attr(handle: &Handle, name: &str) -> bool {
    if let NodeData::Element { ref attrs, .. } = handle.data {
        attrs.borrow().iter().any(|a| a.name.local.as_ref() == name)
    } else {
        false
    }
}

/// Check whether the element's tag name matches the given name.
pub(crate) fn is_tag(handle: &Handle, expected: &str) -> bool {
    if let NodeData::Element { ref name, .. } = handle.data {
        name.local.as_ref() == expected
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Whitespace helpers
// ---------------------------------------------------------------------------

/// Collapse sequences of whitespace (space, tab, newline) to a single space.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }
    result
}

/// Collapse sequences of horizontal whitespace (space, tab) to a single space,
/// but preserve newlines. Used when `newlines: true` is set.
fn collapse_whitespace_preserving_newlines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_hspace = false; // horizontal space (space/tab)

    for c in s.chars() {
        if c == '\r' {
            continue; // strip CR, keep LF
        } else if c == '\n' {
            // Remove any trailing horizontal space before the newline.
            while result.ends_with(' ') {
                result.pop();
            }
            result.push('\n');
            prev_was_hspace = false;
        } else if c == ' ' || c == '\t' {
            if !prev_was_hspace && !result.ends_with('\n') {
                result.push(' ');
            }
            prev_was_hspace = true;
        } else {
            result.push(c);
            prev_was_hspace = false;
        }
    }
    // Trim leading/trailing newlines that are just whitespace artifacts.
    result
}

/// Extract the text content from all descendants of an element (for <pre> blocks).
/// Port of hast-util-to-text behavior: block elements get newlines around them, <br> becomes \n.
fn to_text(handle: &Handle) -> String {
    let mut result = String::new();
    collect_text(handle, &mut result);
    result
}

/// Convert a <table> element to text using tab/newline separators.
/// Matches hast-util-to-text's inner-text algorithm for tables:
/// cells joined with \t, rows joined with \n.
fn to_table_text(handle: &Handle) -> String {
    let mut rows: Vec<String> = Vec::new();
    collect_table_rows(handle, &mut rows);
    rows.join("\n")
}

fn collect_table_rows(handle: &Handle, rows: &mut Vec<String>) {
    match &handle.data {
        NodeData::Element { ref name, .. } => {
            let tag = name.local.as_ref();
            if tag == "tr" {
                let mut cells: Vec<String> = Vec::new();
                for child in handle.children.borrow().iter() {
                    if let NodeData::Element { ref name, .. } = child.data {
                        if matches!(name.local.as_ref(), "td" | "th") {
                            let mut cell_text = String::new();
                            collect_text(child, &mut cell_text);
                            cells.push(cell_text.trim().to_string());
                        }
                    }
                }
                if !cells.is_empty() {
                    rows.push(cells.join("\t"));
                }
            } else {
                for child in handle.children.borrow().iter() {
                    collect_table_rows(child, rows);
                }
            }
        }
        _ => {
            for child in handle.children.borrow().iter() {
                collect_table_rows(child, rows);
            }
        }
    }
}

fn collect_text(handle: &Handle, result: &mut String) {
    match &handle.data {
        NodeData::Text { ref contents } => {
            result.push_str(&contents.borrow());
        }
        NodeData::Element { ref name, .. } => {
            let tag = name.local.as_ref();
            // <br> → newline
            if tag == "br" {
                result.push('\n');
                return;
            }
            // Block elements get a newline before and after their content.
            if is_block_element(tag) {
                // Only add leading \n if not at start.
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                let start_len = result.len();
                for child in handle.children.borrow().iter() {
                    collect_text(child, result);
                }
                // Add trailing \n if content was added and doesn't end with \n.
                if result.len() > start_len && !result.ends_with('\n') {
                    result.push('\n');
                }
            } else {
                for child in handle.children.borrow().iter() {
                    collect_text(child, result);
                }
            }
        }
        NodeData::Document => {
            for child in handle.children.borrow().iter() {
                collect_text(child, result);
            }
        }
        _ => {}
    }
}

/// Whether an HTML tag is a block-level display element.
fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "body"
            | "caption"
            | "center"
            | "col"
            | "colgroup"
            | "dd"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "html"
            | "legend"
            | "li"
            | "listing"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "plaintext"
            | "pre"
            | "section"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
            | "xmp"
    )
}

/// Trim trailing newlines from a string.
/// Strip exactly one trailing newline from code block content.
///
/// HTML serializers add a `\n` before `</code>` (e.g. `<pre><code>b\n</code>`).
/// That single trailing `\n` is an artifact, not part of the code content.
/// We strip it so the Markdown fenced block serializer can add its own `\n`
/// before the closing fence without doubling up.
///
/// Trimming ALL trailing newlines (the original behavior) was too aggressive:
/// it discarded intentional trailing blank lines in code content.
fn trim_trailing_lines(s: &str) -> &str {
    if let Some(stripped) = s.strip_suffix("\r\n") {
        stripped
    } else if let Some(stripped) = s.strip_suffix('\n') {
        stripped
    } else {
        s
    }
}

// ---------------------------------------------------------------------------
// Element handlers
// ---------------------------------------------------------------------------

/// <a> → Link
/// Port of hast-util-to-mdast/lib/handlers/a.js
fn handle_a(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let href = get_attr(handle, "href").unwrap_or_default();
    let url = state.resolve(&href);
    let title = get_attr(handle, "title");
    let children = all(state, handle);

    vec![mdast::Node::Link(mdast::Link {
        url,
        title,
        children,
    })]
}

/// <base> → sets frozen_base_url
/// Port of hast-util-to-mdast/lib/handlers/base.js
///
/// Per HTML5 spec, only the FIRST `<base>` element is effective.
/// If the first `<base>` has no `href`, no URL resolution is applied
/// (subsequent `<base>` elements with `href` are ignored).
fn handle_base(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    if !state.base_found {
        state.base_found = true;
        if let Some(href) = get_attr(handle, "href") {
            if let Ok(url) = url::Url::parse(&href) {
                state.frozen_base_url = Some(url);
            }
        }
    }
    vec![]
}

/// <blockquote> → Blockquote
/// Port of hast-util-to-mdast/lib/handlers/blockquote.js
fn handle_blockquote(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = all(state, handle);
    let children = super::wrap::wrap(children);
    vec![mdast::Node::Blockquote(mdast::Blockquote { children })]
}

/// <br> → Break
/// Port of hast-util-to-mdast/lib/handlers/br.js
fn handle_br() -> Vec<mdast::Node> {
    vec![mdast::Node::Break(mdast::Break)]
}

/// <code>, <kbd>, <samp>, <tt>, <var> → InlineCode
/// Port of hast-util-to-mdast/lib/handlers/inline-code.js
fn handle_code_inline(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let text = to_text(handle);
    // Inline code: collapse whitespace (not in pre context)
    let value = if state.in_pre {
        text
    } else {
        collapse_whitespace(&text)
    };
    if value.is_empty() {
        return vec![];
    }
    vec![mdast::Node::InlineCode(mdast::InlineCode { value })]
}

/// <pre>, <listing>, <xmp>, <plaintext> → Code
/// Port of hast-util-to-mdast/lib/handlers/code.js
fn handle_code_block(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    // Extract language from <code class="language-*"> child.
    let lang = if is_tag(handle, "pre") {
        find_code_language(handle)
    } else {
        None
    };

    // Get raw text content (preserving whitespace).
    let old_in_pre = state.in_pre;
    state.in_pre = true;
    let value = to_text(handle);
    state.in_pre = old_in_pre;

    let value = trim_trailing_lines(&value).to_string();

    vec![mdast::Node::Code(mdast::Code {
        value,
        lang,
        meta: None,
    })]
}

/// Find the `language-*` class on a `<code>` child of `<pre>`.
fn find_code_language(pre_handle: &Handle) -> Option<String> {
    for child in pre_handle.children.borrow().iter() {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = child.data
        {
            if name.local.as_ref() == "code" {
                for attr in attrs.borrow().iter() {
                    if attr.name.local.as_ref() == "class" {
                        let class_val = attr.value.to_string();
                        for class in class_val.split_whitespace() {
                            if let Some(lang) = class.strip_prefix("language-") {
                                return Some(lang.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// <del>, <s>, <strike> → Delete
/// Port of hast-util-to-mdast/lib/handlers/del.js
fn handle_del(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = all(state, handle);
    vec![mdast::Node::Delete(mdast::Delete { children })]
}

/// <dl> → List (grouping dt/dd pairs)
/// Port of hast-util-to-mdast/lib/handlers/dl.js
fn handle_dl(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    // Unwrap <div> children.
    let mut clean: Vec<Handle> = Vec::new();
    for child in handle.children.borrow().iter() {
        if let NodeData::Element { ref name, .. } = child.data {
            if name.local.as_ref() == "div" {
                for grandchild in child.children.borrow().iter() {
                    clean.push(grandchild.clone());
                }
                continue;
            }
        }
        clean.push(child.clone());
    }

    // Group titles (dt) and definitions (dd).
    struct Group {
        titles: Vec<Handle>,
        definitions: Vec<Handle>,
    }

    let mut groups: Vec<Group> = Vec::new();
    let mut current = Group {
        titles: Vec::new(),
        definitions: Vec::new(),
    };
    let mut prev_was_dd = false;

    for child in &clean {
        let child_tag = if let NodeData::Element { ref name, .. } = child.data {
            Some(name.local.as_ref().to_string())
        } else {
            None
        };

        if child_tag.as_deref() == Some("dt") {
            if prev_was_dd {
                groups.push(current);
                current = Group {
                    titles: Vec::new(),
                    definitions: Vec::new(),
                };
            }
            current.titles.push(child.clone());
            prev_was_dd = false;
        } else {
            current.definitions.push(child.clone());
            if child_tag.as_deref() == Some("dd") {
                prev_was_dd = true;
            }
        }
    }
    groups.push(current);

    // Convert each group to a list item.
    let mut content: Vec<mdast::Node> = Vec::new();

    for group in groups {
        let title_nodes = dl_handle_group(state, &group.titles);
        let def_nodes = dl_handle_group(state, &group.definitions);
        let mut result_children: Vec<mdast::Node> = Vec::new();
        result_children.extend(title_nodes);
        result_children.extend(def_nodes);

        if !result_children.is_empty() {
            let spread = result_children.len() > 1;
            content.push(mdast::Node::ListItem(mdast::ListItem {
                spread,
                checked: None,
                children: result_children,
            }));
        }
    }

    if content.is_empty() {
        return vec![];
    }

    let spread = list_items_spread(&content);
    vec![mdast::Node::List(mdast::List {
        ordered: false,
        start: None,
        spread,
        children: content,
    })]
}

/// Convert a set of dt or dd handles to flow content for a dl list item.
/// Port of hast-util-to-mdast/lib/handlers/dl.js `handle()` function.
fn dl_handle_group(state: &mut State, handles: &[Handle]) -> Vec<mdast::Node> {
    if handles.is_empty() {
        return vec![];
    }

    // Process each dt/dd element through `one()` (dispatches to handle_li),
    // equivalent to JS `state.all({type: 'root', children: [element]})`.
    let mut nodes: Vec<mdast::Node> = Vec::new();
    for h in handles {
        nodes.extend(one(state, h));
    }

    // Wrap phrasing in paragraphs.
    let list_items = to_specific_list_items(nodes);

    if list_items.is_empty() {
        return vec![];
    }

    if list_items.len() == 1 {
        // Return the single list item's children directly.
        if let mdast::Node::ListItem(li) = list_items.into_iter().next().unwrap() {
            return li.children;
        }
        return vec![];
    }

    // Multiple list items → wrap in a nested list.
    let spread = list_items_spread(&list_items);
    vec![mdast::Node::List(mdast::List {
        ordered: false,
        start: None,
        spread,
        children: list_items,
    })]
}

/// <em>, <i>, <mark>, <u> → Emphasis
/// Port of hast-util-to-mdast/lib/handlers/em.js
fn handle_em(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = all(state, handle);
    vec![mdast::Node::Emphasis(mdast::Emphasis { children })]
}

/// <h1>–<h6> → Heading
/// Port of hast-util-to-mdast/lib/handlers/heading.js
fn handle_heading(state: &mut State, handle: &Handle, tag: &str) -> Vec<mdast::Node> {
    let depth = tag.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1) as u8;
    let children = all(state, handle);
    let children = drop_surrounding_breaks(children);
    vec![mdast::Node::Heading(mdast::Heading { depth, children })]
}

/// <hr> → ThematicBreak
/// Port of hast-util-to-mdast/lib/handlers/hr.js
fn handle_hr() -> Vec<mdast::Node> {
    vec![mdast::Node::ThematicBreak(mdast::ThematicBreak)]
}

/// <iframe> → Link (if src + title both present)
/// Port of hast-util-to-mdast/lib/handlers/iframe.js
fn handle_iframe(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let src = get_attr(handle, "src").unwrap_or_default();
    let title = get_attr(handle, "title");

    if !src.is_empty() {
        if let Some(title_text) = title {
            let url = state.resolve(&src);
            return vec![mdast::Node::Link(mdast::Link {
                url,
                title: None,
                children: vec![mdast::Node::Text(mdast::Text { value: title_text })],
            })];
        }
    }
    vec![]
}

/// <img>, <image> → Image
/// Port of hast-util-to-mdast/lib/handlers/img.js
fn handle_img(handle: &Handle) -> Vec<mdast::Node> {
    let src = get_attr(handle, "src").unwrap_or_default();
    let alt = get_attr(handle, "alt").unwrap_or_default();
    let title = get_attr(handle, "title");

    vec![mdast::Node::Image(mdast::Image {
        url: src,
        title,
        alt,
    })]
}

/// <input> → varies by type
/// Port of hast-util-to-mdast/lib/handlers/input.js
fn handle_input(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    // disabled, hidden, file → skip
    if has_attr(handle, "disabled") {
        return vec![];
    }
    let input_type = get_attr(handle, "type")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if input_type == "hidden" || input_type == "file" {
        return vec![];
    }

    // checkbox / radio → checked symbol
    if input_type == "checkbox" || input_type == "radio" {
        let checked = has_attr(handle, "checked");
        let value = if checked {
            state.options.checked.as_deref().unwrap_or("[x]")
        } else {
            state.options.unchecked.as_deref().unwrap_or("[ ]")
        };
        return vec![mdast::Node::Text(mdast::Text {
            value: value.to_string(),
        })];
    }

    // image type → Image node
    if input_type == "image" {
        let src = get_attr(handle, "src").unwrap_or_default();
        let alt = get_attr(handle, "alt")
            .or_else(|| get_attr(handle, "value"))
            .or_else(|| get_attr(handle, "placeholder"))
            .unwrap_or_default();
        if !alt.is_empty() {
            let url = state.resolve(&src);
            let title = get_attr(handle, "title");
            return vec![mdast::Node::Image(mdast::Image { url, title, alt })];
        }
        return vec![];
    }

    // email / url → Link; others with value/placeholder → Text
    // Also handle `list` attribute: look up datalist options.
    let value = get_attr(handle, "value")
        .or_else(|| get_attr(handle, "placeholder"))
        .unwrap_or_default();

    // Determine values: from explicit value/placeholder, or from a linked datalist.
    // Each entry is (value, optional_display_label).
    let options: Vec<(String, Option<String>)> = if !value.is_empty() {
        vec![(value.clone(), None)]
    } else {
        // Unsupported list types:
        let no_list = matches!(
            input_type.as_str(),
            "button" | "file" | "password" | "reset" | "submit"
        );
        if !no_list {
            if let Some(list_id) = get_attr(handle, "list") {
                if let Some(datalist_handle) = state.element_by_id.get(&list_id).cloned() {
                    let is_multiple = has_attr(handle, "multiple");
                    let size = get_attr(handle, "size").and_then(|s| s.parse::<usize>().ok());
                    let props = ExplicitInputProps {
                        multiple: is_multiple,
                        size,
                    };
                    find_selected_options(&datalist_handle, Some(&props))
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    };

    if options.is_empty() {
        return vec![];
    }

    // Passwords: obscure value.
    let options: Vec<(String, Option<String>)> = if input_type == "password" {
        options
            .into_iter()
            .map(|(v, l)| ("•".repeat(v.chars().count()), l))
            .collect()
    } else {
        options
    };

    if input_type == "email" || input_type == "url" {
        let mut result_nodes = Vec::new();
        for (i, (v, label)) in options.iter().enumerate() {
            let url = if input_type == "email" {
                format!("mailto:{}", v)
            } else {
                state.resolve(v)
            };
            // Use label as display text if present, otherwise use the raw value.
            let display = label.as_deref().unwrap_or(v.as_str()).to_string();
            result_nodes.push(mdast::Node::Link(mdast::Link {
                url,
                title: None,
                children: vec![mdast::Node::Text(mdast::Text { value: display })],
            }));
            if i + 1 < options.len() {
                result_nodes.push(mdast::Node::Text(mdast::Text {
                    value: ", ".to_string(),
                }));
            }
        }
        return result_nodes;
    }

    let text = options
        .into_iter()
        .map(|(v, label)| match label {
            Some(l) => format!("{} ({})", l, v),
            None => v,
        })
        .collect::<Vec<_>>()
        .join(", ");
    vec![mdast::Node::Text(mdast::Text { value: text })]
}

/// <li>, <dt>, <dd> → ListItem
/// Port of hast-util-to-mdast/lib/handlers/li.js
fn handle_li(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let (mut checked, checkbox_location) = detect_leading_checkbox(handle);
    let spread = spreadout(handle);
    let children_nodes = all_except_leading_checkbox(state, handle, checkbox_location);
    let children = super::wrap::wrap(children_nodes);

    // A bare checkbox with no following text content should not be marked as a
    // checkbox item (it shows only as an empty bullet in the fixture).
    if checked.is_some() && is_whitespace_only(&children) && children.is_empty() {
        checked = None;
    }

    vec![mdast::Node::ListItem(mdast::ListItem {
        spread,
        checked,
        children,
    })]
}

/// Where the leading checkbox is located (None = no checkbox).
#[derive(Clone, Copy)]
enum CheckboxLocation {
    None,
    /// Directly as first child of the li.
    Direct,
    /// Inside the first `<p>` child of the li.
    InsideFirstP,
}

/// Detect a leading checkbox without consuming it, returning its checked state and location.
/// Skips leading whitespace-only text nodes to find the first meaningful child.
fn detect_leading_checkbox(handle: &Handle) -> (Option<bool>, CheckboxLocation) {
    let children_ref = handle.children.borrow();
    // Skip leading whitespace-only text nodes.
    let first = children_ref.iter().find(|child| match &child.data {
        NodeData::Text { ref contents } => !contents.borrow().trim().is_empty(),
        _ => true,
    });

    if let Some(first) = first {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = first.data
        {
            let tag = name.local.as_ref();
            let input_type = attrs
                .borrow()
                .iter()
                .find(|a| a.name.local.as_ref() == "type")
                .map(|a| a.value.to_string().to_lowercase())
                .unwrap_or_default();

            if tag == "input" && (input_type == "checkbox" || input_type == "radio") {
                let checked = attrs
                    .borrow()
                    .iter()
                    .any(|a| a.name.local.as_ref() == "checked");
                return (Some(checked), CheckboxLocation::Direct);
            }

            if tag == "p" {
                // Also skip leading whitespace inside the p element.
                let p_children_ref = first.children.borrow();
                let p_first = p_children_ref.iter().find(|c| match &c.data {
                    NodeData::Text { ref contents } => !contents.borrow().trim().is_empty(),
                    _ => true,
                });
                if let Some(p_first_child) = p_first {
                    if let NodeData::Element {
                        ref name,
                        ref attrs,
                        ..
                    } = p_first_child.data
                    {
                        let p_tag = name.local.as_ref();
                        let p_type = attrs
                            .borrow()
                            .iter()
                            .find(|a| a.name.local.as_ref() == "type")
                            .map(|a| a.value.to_string().to_lowercase())
                            .unwrap_or_default();
                        if p_tag == "input" && (p_type == "checkbox" || p_type == "radio") {
                            let checked = attrs
                                .borrow()
                                .iter()
                                .any(|a| a.name.local.as_ref() == "checked");
                            return (Some(checked), CheckboxLocation::InsideFirstP);
                        }
                    }
                }
            }
        }
    }
    (None, CheckboxLocation::None)
}

/// Convert all children of a li handle, skipping the leading checkbox if present.
fn all_except_leading_checkbox(
    state: &mut State,
    handle: &Handle,
    checkbox_loc: CheckboxLocation,
) -> Vec<mdast::Node> {
    let children_ref = handle.children.borrow();
    let mut result = Vec::new();

    // Find the index of the first meaningful (non-whitespace) child.
    let first_meaningful_idx = children_ref.iter().position(|child| match &child.data {
        NodeData::Text { ref contents } => !contents.borrow().trim().is_empty(),
        _ => true,
    });

    for (i, child) in children_ref.iter().enumerate() {
        let is_first_meaningful = Some(i) == first_meaningful_idx;
        match checkbox_loc {
            CheckboxLocation::Direct if is_first_meaningful => {
                // Skip the checkbox input element.
                continue;
            }
            CheckboxLocation::InsideFirstP if is_first_meaningful => {
                // Process the first <p> but skip its first meaningful child (the checkbox).
                let p_children_ref = child.children.borrow();
                let p_first_meaningful = p_children_ref.iter().position(|c| match &c.data {
                    NodeData::Text { ref contents } => !contents.borrow().trim().is_empty(),
                    _ => true,
                });
                let mut p_result = Vec::new();
                for (j, p_child) in p_children_ref.iter().enumerate() {
                    if Some(j) == p_first_meaningful {
                        continue; // skip checkbox
                    }
                    p_result.extend(one(state, p_child));
                }
                let p_result = drop_surrounding_breaks(p_result);
                if !p_result.is_empty() && !is_whitespace_only(&p_result) {
                    result.push(mdast::Node::Paragraph(mdast::Paragraph {
                        children: p_result,
                    }));
                }
            }
            _ => {
                result.extend(one(state, child));
            }
        }
    }

    result
}

/// Check if a `<li>` element should be "spread" (loose) in the MDAST sense.
///
/// A list item is spread (loose) if:
///   - it has a direct `<p>` child (CommonMark's signal for a loose item), OR
///   - it has a direct `<div>` child that itself contains a `<p>` (generic
///     container wrapping — recurse one level into `<div>` only).
///
/// We deliberately do NOT recurse into semantic block elements like
/// `<blockquote>`, `<ul>`, `<ol>`, `<pre>`, etc. because those elements
/// inherently contain `<p>` tags in CommonMark HTML even in tight lists.
/// Recursing into them produces false positives:
///   `<li>a<blockquote><p>b</p></blockquote></li>` is a TIGHT item,
///   but the `<p>` inside the blockquote would trigger a false spread.
///
/// We also do NOT use a `seenFlow` rule (two or more block children → spread)
/// because CommonMark allows tight list items with multiple block children:
///   `<li>a<blockquote>…</blockquote><pre>…</pre></li>` is tight.
fn spreadout(handle: &Handle) -> bool {
    for child in handle.children.borrow().iter() {
        if let NodeData::Element { ref name, .. } = child.data {
            let tag = name.local.as_ref();
            // Direct <p> child → always spread.
            if tag == "p" {
                return true;
            }
            // <div> is a generic container; recurse one level to find <p>.
            if tag == "div" && spreadout(child) {
                return true;
            }
        }
    }
    false
}

/// <ol>, <ul>, <dir> → List
/// Port of hast-util-to-mdast/lib/handlers/list.js
fn handle_list(state: &mut State, handle: &Handle, tag: &str) -> Vec<mdast::Node> {
    let ordered = tag == "ol";
    let start = if ordered {
        get_attr(handle, "start")
            .and_then(|s| s.parse::<u32>().ok())
            .or(Some(1))
    } else {
        None
    };

    let children_nodes = all(state, handle);
    let children = to_specific_list_items(children_nodes);
    let spread = list_items_spread(&children);

    vec![mdast::Node::List(mdast::List {
        ordered,
        start,
        spread,
        children,
    })]
}

/// <audio>, <video> → Link or fallback content
/// Port of hast-util-to-mdast/lib/handlers/media.js
fn handle_media(state: &mut State, handle: &Handle, tag: &str) -> Vec<mdast::Node> {
    let poster = if tag == "video" {
        get_attr(handle, "poster").unwrap_or_default()
    } else {
        String::new()
    };
    let src_attr = get_attr(handle, "src").unwrap_or_default();

    let nodes = all(state, handle);

    // Check if fallback content has links or non-phrasing.
    let has_link = nodes.iter().any(|n| matches!(n, mdast::Node::Link(_)));
    let needs_wrap = super::wrap::wrap_needed(&nodes);

    if has_link || needs_wrap {
        return nodes;
    }

    // Find src from <source> child if not on element.
    let source = if src_attr.is_empty() {
        find_source_src(handle)
    } else {
        src_attr
    };

    // If video with poster, create Image wrapped in a Link to the source.
    if !poster.is_empty() {
        let alt = nodes_to_text(&nodes).trim().to_string();
        let image = mdast::Node::Image(mdast::Image {
            url: state.resolve(&poster),
            title: None,
            alt,
        });
        let link_url = state.resolve(&source);
        let title = get_attr(handle, "title");
        return vec![mdast::Node::Link(mdast::Link {
            url: link_url,
            title,
            children: vec![image],
        })];
    }

    let title = get_attr(handle, "title");
    let url = state.resolve(&source);
    vec![mdast::Node::Link(mdast::Link {
        url,
        title,
        children: nodes,
    })]
}

/// Find src from a <source> child element.
fn find_source_src(handle: &Handle) -> String {
    for child in handle.children.borrow().iter() {
        if let NodeData::Element { ref name, .. } = child.data {
            if name.local.as_ref() == "source" {
                if let Some(src) = get_attr(child, "src") {
                    return src;
                }
            }
        }
    }
    String::new()
}

/// Extract plain text from MDAST nodes (for alt text).
fn nodes_to_text(nodes: &[mdast::Node]) -> String {
    let mut result = String::new();
    for node in nodes {
        match node {
            mdast::Node::Text(t) => result.push_str(&t.value),
            mdast::Node::InlineCode(c) => result.push_str(&c.value),
            _ => {
                if let Some(children) = node.children() {
                    result.push_str(&nodes_to_text(children));
                }
            }
        }
    }
    result
}

/// <p>, <summary> → Paragraph (or empty if no meaningful content)
/// Port of hast-util-to-mdast/lib/handlers/p.js
fn handle_p(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = all(state, handle);
    let children = drop_surrounding_breaks(children);
    // Drop if all children are whitespace-only text (or empty).
    if children.is_empty() || is_whitespace_only(&children) {
        return vec![];
    }
    vec![mdast::Node::Paragraph(mdast::Paragraph { children })]
}

/// <q> → Text with quotes wrapping children
/// Port of hast-util-to-mdast/lib/handlers/q.js
fn handle_q(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    // Pick quote pair based on nesting depth (cycles through the quotes array).
    // Default is a single `"` character (both open and close).
    let quotes = &state.options.quotes;
    debug_assert!(
        !quotes.is_empty(),
        "quotes should always have at least one entry"
    );
    let fallback = "\"".to_string();
    let quote_str = if quotes.is_empty() {
        &fallback
    } else {
        &quotes[state.q_nesting % quotes.len()]
    };
    let mut chars = quote_str.chars();
    let open = chars.next().unwrap_or('"');
    let close = chars.next().unwrap_or(open);

    state.q_nesting += 1;
    let mut contents = all(state, handle);
    state.q_nesting -= 1;

    // Prepend open quote to first text node (or insert new text).
    if let Some(mdast::Node::Text(t)) = contents.first_mut() {
        t.value.insert(0, open);
    } else {
        contents.insert(
            0,
            mdast::Node::Text(mdast::Text {
                value: open.to_string(),
            }),
        );
    }

    // Append close quote to last text node (or insert new text).
    if let Some(mdast::Node::Text(t)) = contents.last_mut() {
        t.value.push(close);
    } else {
        contents.push(mdast::Node::Text(mdast::Text {
            value: close.to_string(),
        }));
    }

    contents
}

/// <select> → Text (selected options)
/// Port of hast-util-to-mdast/lib/handlers/select.js
fn handle_select(_state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let options = find_selected_options(handle, None);
    if options.is_empty() {
        return vec![];
    }
    let text = options
        .into_iter()
        .map(|(value, label)| match label {
            Some(l) => format!("{} ({})", l, value),
            None => value,
        })
        .collect::<Vec<_>>()
        .join(", ");
    vec![mdast::Node::Text(mdast::Text { value: text })]
}

/// Find selected option values in a <select> or <datalist> element.
/// Port of hast-util-to-mdast/lib/util/find-selected-options.js
///
/// `explicit_props`: override properties (e.g. from the `<input list=…>` element).
/// Returns `(value, label)` tuples where label is `None` when it equals value.
/// Port of hast-util-to-mdast/lib/util/find-selected-options.js
pub(crate) fn find_selected_options(
    handle: &Handle,
    explicit_props: Option<&ExplicitInputProps>,
) -> Vec<(String, Option<String>)> {
    // Collect all options.
    let mut all_options: Vec<OptionData> = Vec::new();
    collect_options_data(handle, &mut all_options);

    // Determine which options are explicitly selected.
    let selected: Vec<usize> = all_options
        .iter()
        .enumerate()
        .filter_map(|(i, o)| if o.selected { Some(i) } else { None })
        .collect();

    // Determine size limit.
    // Per JS ref: Math.min(parseInt(size), 0) || (multiple ? 4 : 1)
    // This means positive `size` values are ignored (min with 0 → 0 → fallback).
    // Only negative sizes would be used, which is nonsensical for HTML, so
    // effectively: always use (multiple ? 4 : 1).
    let is_multiple = explicit_props.is_some_and(|p| p.multiple) || has_attr(handle, "multiple");
    let size_attr: Option<isize> = explicit_props
        .and_then(|p| p.size.map(|s| s as isize))
        .or_else(|| get_attr(handle, "size").and_then(|s| s.parse::<isize>().ok()));
    // min(size, 0): positive → 0, negative → keeps, NaN → 0.
    let capped = size_attr.map(|s| s.min(0)).unwrap_or(0);
    let size = if capped < 0 {
        (-capped) as usize
    } else {
        // 0 → use fallback
        if is_multiple {
            4
        } else {
            1
        }
    };

    // Build result from the appropriate list (selected or all), limited to size.
    let effective_indices: Vec<usize> = if !selected.is_empty() {
        selected
    } else {
        (0..all_options.len()).collect()
    };

    effective_indices
        .into_iter()
        .take(size)
        .map(|i| {
            let opt = &all_options[i];
            let label = &opt.label;
            let value = &opt.value;
            let distinct_label = if !label.is_empty() && label != value {
                Some(label.clone())
            } else {
                None
            };
            (value.clone(), distinct_label)
        })
        .collect()
}

/// Represents a parsed option element.
struct OptionData {
    value: String,
    label: String,
    selected: bool,
}

/// Collect all non-disabled option elements recursively.
fn collect_options_data(handle: &Handle, results: &mut Vec<OptionData>) {
    for child in handle.children.borrow().iter() {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = child.data
        {
            let tag = name.local.as_ref();
            if tag == "option" {
                let is_disabled = attrs
                    .borrow()
                    .iter()
                    .any(|a| a.name.local.as_ref() == "disabled");
                if is_disabled {
                    // Still recurse? Options don't have children in practice.
                    continue;
                }
                let is_selected = attrs
                    .borrow()
                    .iter()
                    .any(|a| a.name.local.as_ref() == "selected");
                let value_attr = attrs
                    .borrow()
                    .iter()
                    .find(|a| a.name.local.as_ref() == "value")
                    .map(|a| a.value.to_string());
                let text_content = collapse_whitespace(to_text(child).trim());
                let label_attr = attrs
                    .borrow()
                    .iter()
                    .find(|a| a.name.local.as_ref() == "label")
                    .map(|a| a.value.to_string());
                // JS: label = content || String(properties.label || '')
                // text content takes precedence; fall back to label attr.
                let label = if !text_content.is_empty() {
                    text_content.clone()
                } else {
                    label_attr.unwrap_or_default()
                };
                // JS: value = String(properties.value || '') || content
                // Empty string value attr is treated as missing (falsy in JS).
                let value = value_attr
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| text_content.clone());
                results.push(OptionData {
                    value,
                    label,
                    selected: is_selected,
                });
            } else {
                // Recurse into all elements (JS findOptions recurses into all children).
                collect_options_data(child, results);
            }
        }
    }
}

/// Explicit properties from an `<input list=…>` element (for datalist lookup).
pub(crate) struct ExplicitInputProps {
    pub multiple: bool,
    pub size: Option<usize>,
}

/// <strong>, <b> → Strong
/// Port of hast-util-to-mdast/lib/handlers/strong.js
fn handle_strong(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = all(state, handle);
    vec![mdast::Node::Strong(mdast::Strong { children })]
}

/// <table> → Table (or Text if nested)
/// Port of hast-util-to-mdast/lib/handlers/table.js
fn handle_table(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    // Nested table → serialize as text using tab/newline separators.
    if state.in_table {
        let text = to_table_text(handle);
        return vec![mdast::Node::Text(mdast::Text { value: text })];
    }

    state.in_table = true;

    let (align, headless) = inspect_table(handle);
    let raw_nodes = all(state, handle);
    let mut rows = to_specific_table_rows(raw_nodes);

    // Add an empty header row if headless.
    if headless {
        rows.insert(
            0,
            mdast::Node::TableRow(mdast::TableRow { children: vec![] }),
        );
    }

    // Ensure all row children are TableCells.
    for row in &mut rows {
        if let mdast::Node::TableRow(tr) = row {
            let cells = std::mem::take(&mut tr.children);
            tr.children = to_specific_table_cells(cells);
        }
    }

    // Handle colspan/rowspan expansion.
    let mut columns = 1usize;

    let row_count = rows.len();
    for row_index in 0..row_count {
        if let mdast::Node::TableRow(tr) = &rows[row_index] {
            let cell_count = tr.children.len();
            if cell_count > columns {
                columns = cell_count;
            }
        }

        // Process colspan/rowspan for each cell.
        let cells: Vec<(usize, u32, u32)> = {
            let tr = if let mdast::Node::TableRow(tr) = &rows[row_index] {
                tr
            } else {
                continue;
            };
            tr.children
                .iter()
                .enumerate()
                .filter_map(|(cell_index, cell)| {
                    if let mdast::Node::TableCell(tc) = cell {
                        let colspan = tc.colspan.unwrap_or(1);
                        let rowspan = tc.rowspan.unwrap_or(1);
                        if colspan > 1 || rowspan > 1 {
                            Some((cell_index, colspan, rowspan))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        };

        for (cell_index, colspan, rowspan) in cells {
            let end_row = (row_index + rowspan as usize).min(row_count);
            for (span_offset, row) in rows[row_index..end_row].iter_mut().enumerate() {
                let other_row_index = row_index + span_offset;
                let col_start = if other_row_index == row_index {
                    cell_index + 1
                } else {
                    cell_index
                };
                let col_end = cell_index + colspan as usize;
                if col_start < col_end {
                    let empty_cells: Vec<mdast::Node> = (col_start..col_end)
                        .map(|_| mdast::Node::TableCell(mdast::TableCell::new(vec![])))
                        .collect();
                    if let mdast::Node::TableRow(tr) = row {
                        let insert_at = col_start.min(tr.children.len());
                        for (offset, cell) in empty_cells.into_iter().enumerate() {
                            tr.children.insert(insert_at + offset, cell);
                        }
                    }
                }
            }
        }
    }

    // Determine final column count.
    for row in &rows {
        if let mdast::Node::TableRow(tr) = row {
            if tr.children.len() > columns {
                columns = tr.children.len();
            }
        }
    }

    // Pad rows with empty cells and extend align array.
    for row in &mut rows {
        if let mdast::Node::TableRow(tr) = row {
            while tr.children.len() < columns {
                tr.children
                    .push(mdast::Node::TableCell(mdast::TableCell::new(vec![])));
            }
        }
    }

    let mut align = align;
    while align.len() < columns {
        align.push(None);
    }

    // Clear colspan/rowspan data from cells (they've been expanded).
    for row in &mut rows {
        if let mdast::Node::TableRow(tr) = row {
            for cell in &mut tr.children {
                if let mdast::Node::TableCell(tc) = cell {
                    tc.colspan = None;
                    tc.rowspan = None;
                }
            }
        }
    }

    state.in_table = false;

    vec![mdast::Node::Table(mdast::Table {
        align,
        children: rows,
    })]
}

/// Inspect a <table> element to determine alignment and whether it has a header.
/// Port of `inspect` in hast-util-to-mdast/lib/handlers/table.js
fn inspect_table(handle: &Handle) -> (Vec<Option<mdast::AlignKind>>, bool) {
    let mut align: Vec<Option<mdast::AlignKind>> = vec![None];
    let mut headless = true;
    let mut row_index = 0usize;
    let mut cell_index = 0usize;

    inspect_table_node(
        handle,
        handle,
        &mut align,
        &mut headless,
        &mut row_index,
        &mut cell_index,
    );
    (align, headless)
}

fn inspect_table_node(
    root: &Handle,
    handle: &Handle,
    align: &mut Vec<Option<mdast::AlignKind>>,
    headless: &mut bool,
    row_index: &mut usize,
    cell_index: &mut usize,
) {
    for child in handle.children.borrow().iter() {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = child.data
        {
            let tag = name.local.as_ref();

            // Don't enter nested tables.
            if tag == "table" {
                // Check if this is the root table or a nested one.
                if !std::rc::Rc::ptr_eq(child, root) {
                    continue;
                }
            }

            if tag == "th" || tag == "td" {
                // Update alignment.
                if *cell_index >= align.len() {
                    align.resize(*cell_index + 1, None);
                }
                if align[*cell_index].is_none() {
                    let align_val = attrs
                        .borrow()
                        .iter()
                        .find(|a| a.name.local.as_ref() == "align")
                        .map(|a| a.value.to_string());
                    align[*cell_index] = match align_val.as_deref() {
                        Some("left") => Some(mdast::AlignKind::Left),
                        Some("right") => Some(mdast::AlignKind::Right),
                        Some("center") => Some(mdast::AlignKind::Center),
                        _ => None,
                    };
                }

                // th in first 2 rows → has header.
                if *headless && *row_index < 2 && tag == "th" {
                    *headless = false;
                }

                *cell_index += 1;
            } else if tag == "thead" {
                *headless = false;
                inspect_table_node(root, child, align, headless, row_index, cell_index);
            } else if tag == "tr" {
                *row_index += 1;
                *cell_index = 0;
                inspect_table_node(root, child, align, headless, row_index, cell_index);
            } else {
                inspect_table_node(root, child, align, headless, row_index, cell_index);
            }
        }
    }
}

/// <td>, <th> → TableCell
/// Port of hast-util-to-mdast/lib/handlers/table-cell.js
fn handle_table_cell(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let children = all(state, handle);
    let colspan = get_attr(handle, "colspan")
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|&n| n > 1);
    let rowspan = get_attr(handle, "rowspan")
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|&n| n > 1);

    vec![mdast::Node::TableCell(mdast::TableCell {
        children,
        colspan,
        rowspan,
    })]
}

/// <tr> → TableRow
/// Port of hast-util-to-mdast/lib/handlers/table-row.js
fn handle_table_row(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let child_nodes = all(state, handle);
    let children = to_specific_table_cells(child_nodes);
    vec![mdast::Node::TableRow(mdast::TableRow { children })]
}

/// <textarea> → Text (raw content)
/// Port of hast-util-to-mdast/lib/handlers/textarea.js
fn handle_textarea(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    let old_in_pre = state.in_pre;
    state.in_pre = true;
    let text = to_text(handle);
    state.in_pre = old_in_pre;
    if text.is_empty() {
        return vec![];
    }
    vec![mdast::Node::Text(mdast::Text { value: text })]
}

/// <wbr> → Text (zero-width space)
/// Port of hast-util-to-mdast/lib/handlers/wbr.js
fn handle_wbr() -> Vec<mdast::Node> {
    vec![mdast::Node::Text(mdast::Text {
        value: "\u{200B}".to_string(),
    })]
}

/// <noscript> — html5ever parses its content as raw text when scripting is enabled.
/// Collect the raw text, re-parse as HTML, and process the resulting tree.
fn handle_noscript(state: &mut State, handle: &Handle) -> Vec<mdast::Node> {
    // Collect raw text content of the noscript element.
    let mut raw_html = String::new();
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { ref contents } = child.data {
            raw_html.push_str(&contents.borrow());
        }
    }
    if raw_html.trim().is_empty() {
        return vec![];
    }
    // Re-parse the raw HTML and transform it.
    let dom = super::parse_html(&raw_html);
    let children = all(state, &dom.document);
    super::wrap::wrap(children)
}

// ---------------------------------------------------------------------------
// toSpecificContent helpers
// ---------------------------------------------------------------------------

/// Ensure all nodes are ListItems, wrapping straggler content.
/// Port of toSpecificContent with create=createListItem.
/// Non-ListItem nodes are queued and prepended to the next ListItem,
/// or placed in a new ListItem at the end if none follows.
/// Whitespace-only text nodes outside list items are discarded.
pub(crate) fn to_specific_list_items(nodes: Vec<mdast::Node>) -> Vec<mdast::Node> {
    let mut results: Vec<mdast::Node> = Vec::new();
    let mut queue: Vec<mdast::Node> = Vec::new();

    for node in nodes {
        // Skip whitespace-only text nodes that appear between list items.
        if let mdast::Node::Text(ref t) = node {
            if t.value.trim().is_empty() {
                continue;
            }
        }

        if matches!(node, mdast::Node::ListItem(_)) {
            // Prepend queued nodes into this list item's children.
            let node = if !queue.is_empty() {
                if let mdast::Node::ListItem(mut li) = node {
                    let mut new_children = std::mem::take(&mut queue);
                    new_children.extend(li.children);
                    li.children = new_children;
                    mdast::Node::ListItem(li)
                } else {
                    unreachable!()
                }
            } else {
                node
            };
            results.push(node);
        } else {
            queue.push(node);
        }
    }

    // If there's remaining queue, add to last item or create a new one.
    if !queue.is_empty() {
        // Drop if all remaining queue is whitespace-only.
        if !is_whitespace_only(&queue) {
            if let Some(last) = results.last_mut() {
                if let mdast::Node::ListItem(li) = last {
                    li.children.extend(queue);
                }
            } else {
                results.push(mdast::Node::ListItem(mdast::ListItem {
                    spread: false,
                    checked: None,
                    children: queue,
                }));
            }
        }
    }

    results
}

/// Ensure all nodes are TableRows, wrapping straggler content.
/// Port of toSpecificContent with create=createRow.
/// Whitespace-only text nodes between table rows are discarded (HTML formatting).
fn to_specific_table_rows(nodes: Vec<mdast::Node>) -> Vec<mdast::Node> {
    let mut results: Vec<mdast::Node> = Vec::new();
    let mut queue: Vec<mdast::Node> = Vec::new();

    for node in nodes {
        // Discard whitespace-only text nodes that appear between rows.
        if let mdast::Node::Text(ref t) = node {
            if t.value.trim().is_empty() {
                continue;
            }
        }
        if matches!(node, mdast::Node::TableRow(_)) {
            let node = if !queue.is_empty() {
                if let mdast::Node::TableRow(mut tr) = node {
                    let mut new_children = std::mem::take(&mut queue);
                    new_children.extend(tr.children);
                    tr.children = new_children;
                    mdast::Node::TableRow(tr)
                } else {
                    unreachable!()
                }
            } else {
                node
            };
            results.push(node);
        } else {
            queue.push(node);
        }
    }

    if !queue.is_empty() {
        if let Some(last) = results.last_mut() {
            if let mdast::Node::TableRow(tr) = last {
                tr.children.extend(queue);
            }
        } else {
            results.push(mdast::Node::TableRow(mdast::TableRow { children: queue }));
        }
    }

    results
}

/// Ensure all nodes are TableCells, wrapping straggler content.
/// Whitespace-only text nodes between table cells are discarded (HTML formatting).
fn to_specific_table_cells(nodes: Vec<mdast::Node>) -> Vec<mdast::Node> {
    let mut results: Vec<mdast::Node> = Vec::new();
    let mut queue: Vec<mdast::Node> = Vec::new();

    for node in nodes {
        // Discard whitespace-only text nodes that appear between cells.
        if let mdast::Node::Text(ref t) = node {
            if t.value.trim().is_empty() {
                continue;
            }
        }
        if matches!(node, mdast::Node::TableCell(_)) {
            let node = if !queue.is_empty() {
                if let mdast::Node::TableCell(mut tc) = node {
                    let mut new_children = std::mem::take(&mut queue);
                    new_children.extend(tc.children);
                    tc.children = new_children;
                    mdast::Node::TableCell(tc)
                } else {
                    unreachable!()
                }
            } else {
                node
            };
            results.push(node);
        } else {
            queue.push(node);
        }
    }

    if !queue.is_empty() {
        if let Some(last) = results.last_mut() {
            if let mdast::Node::TableCell(tc) = last {
                tc.children.extend(queue);
            }
        } else {
            results.push(mdast::Node::TableCell(mdast::TableCell::new(queue)));
        }
    }

    results
}

// ---------------------------------------------------------------------------
// List spread detection
// ---------------------------------------------------------------------------

/// Check if any list item in a list is spread.
/// Port of listItemsSpread in hast-util-to-mdast/lib/util/list-items-spread.js
pub(crate) fn list_items_spread(children: &[mdast::Node]) -> bool {
    if children.len() > 1 {
        for child in children {
            if let mdast::Node::ListItem(li) = child {
                if li.spread {
                    return true;
                }
            }
        }
    }
    false
}
