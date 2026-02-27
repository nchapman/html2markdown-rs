// Fixture tests — 127 input/output pairs from hast-util-to-mdast.
//
// Each fixture directory contains:
//   index.html — HTML input
//   index.md   — expected Markdown output
//
// These tests run the full pipeline: HTML string → MDAST → Markdown string.

mod common;

use pretty_assertions::assert_eq;

fn fixture_test(name: &str) {
    let f = common::load_fixture(name);
    let result = html_to_markdown::convert_with(&f.html, &f.options).unwrap();
    assert_eq!(result, f.expected_md, "fixture: {}", name);
}

// ---------------------------------------------------------------------------
// Basic elements
// ---------------------------------------------------------------------------

#[test]
fn fixture_a() { fixture_test("a"); }

#[test]
fn fixture_abbr_acronym() { fixture_test("abbr-acronym"); }

#[test]
fn fixture_address() { fixture_test("address"); }

#[test]
fn fixture_area() { fixture_test("area"); }

#[test]
fn fixture_article_footer_header_section() { fixture_test("article-footer-header-section"); }

#[test]
fn fixture_aside() { fixture_test("aside"); }

#[test]
fn fixture_audio() { fixture_test("audio"); }

#[test]
fn fixture_b() { fixture_test("b"); }

#[test]
fn fixture_base() { fixture_test("base"); }

#[test]
fn fixture_base_invalid() { fixture_test("base-invalid"); }

#[test]
fn fixture_basefont() { fixture_test("basefont"); }

#[test]
fn fixture_bdi_bdo() { fixture_test("bdi-bdo"); }

#[test]
fn fixture_bgsound() { fixture_test("bgsound"); }

#[test]
fn fixture_big() { fixture_test("big"); }

#[test]
fn fixture_blink() { fixture_test("blink"); }

#[test]
fn fixture_blockquote() { fixture_test("blockquote"); }

#[test]
fn fixture_body_html() { fixture_test("body-html"); }

#[test]
fn fixture_br() { fixture_test("br"); }

#[test]
fn fixture_button() { fixture_test("button"); }

#[test]
fn fixture_canvas() { fixture_test("canvas"); }

#[test]
fn fixture_caption_col_colgroup() { fixture_test("caption-col-colgroup"); }

#[test]
fn fixture_center() { fixture_test("center"); }

#[test]
fn fixture_cite() { fixture_test("cite"); }

#[test]
fn fixture_code() { fixture_test("code"); }

#[test]
fn fixture_command() { fixture_test("command"); }

#[test]
fn fixture_comment() { fixture_test("comment"); }

#[test]
fn fixture_content() { fixture_test("content"); }

#[test]
fn fixture_data() { fixture_test("data"); }

#[test]
fn fixture_datalist_input_option() { fixture_test("datalist-input-option"); }

#[test]
fn fixture_del() { fixture_test("del"); }

#[test]
fn fixture_details_summary() { fixture_test("details-summary"); }

#[test]
fn fixture_dfn() { fixture_test("dfn"); }

#[test]
fn fixture_dialog() { fixture_test("dialog"); }

#[test]
fn fixture_dir() { fixture_test("dir"); }

#[test]
fn fixture_div() { fixture_test("div"); }

#[test]
fn fixture_dl() { fixture_test("dl"); }

#[test]
fn fixture_doctype() { fixture_test("doctype"); }

#[test]
fn fixture_document_a() { fixture_test("document-a"); }

#[test]
fn fixture_document_b() { fixture_test("document-b"); }

#[test]
fn fixture_document_c() { fixture_test("document-c"); }

#[test]
fn fixture_em() { fixture_test("em"); }

#[test]
fn fixture_embed() { fixture_test("embed"); }

#[test]
fn fixture_fieldset() { fixture_test("fieldset"); }

#[test]
fn fixture_figure_figcaption() { fixture_test("figure-figcaption"); }

#[test]
fn fixture_font() { fixture_test("font"); }

#[test]
fn fixture_form() { fixture_test("form"); }

#[test]
fn fixture_frame_frameset_noframes() { fixture_test("frame-frameset-noframes"); }

#[test]
fn fixture_gh_27() { fixture_test("gh-27"); }

#[test]
fn fixture_head() { fixture_test("head"); }

#[test]
fn fixture_heading() { fixture_test("heading"); }

#[test]
fn fixture_hgroup() { fixture_test("hgroup"); }

#[test]
fn fixture_hr() { fixture_test("hr"); }

#[test]
fn fixture_i() { fixture_test("i"); }

#[test]
fn fixture_iframe() { fixture_test("iframe"); }

#[test]
fn fixture_ignore() { fixture_test("ignore"); }

#[test]
fn fixture_image() { fixture_test("image"); }

#[test]
fn fixture_img() { fixture_test("img"); }

#[test]
fn fixture_implicit_paragraphs() { fixture_test("implicit-paragraphs"); }

#[test]
fn fixture_input() { fixture_test("input"); }

#[test]
fn fixture_input_checkbox_radio() { fixture_test("input-checkbox-radio"); }

#[test]
fn fixture_ins() { fixture_test("ins"); }

#[test]
fn fixture_kbd() { fixture_test("kbd"); }

#[test]
fn fixture_keygen() { fixture_test("keygen"); }

#[test]
fn fixture_listing() { fixture_test("listing"); }

#[test]
fn fixture_main() { fixture_test("main"); }

#[test]
fn fixture_map() { fixture_test("map"); }

#[test]
fn fixture_mark() { fixture_test("mark"); }

#[test]
fn fixture_marquee() { fixture_test("marquee"); }

#[test]
fn fixture_math() { fixture_test("math"); }

#[test]
fn fixture_menu_menuitem() { fixture_test("menu-menuitem"); }

#[test]
fn fixture_meter() { fixture_test("meter"); }

#[test]
fn fixture_multicol() { fixture_test("multicol"); }

#[test]
fn fixture_nav() { fixture_test("nav"); }

#[test]
fn fixture_newlines_off() { fixture_test("newlines-off"); }

#[test]
fn fixture_newlines_on() { fixture_test("newlines-on"); }

#[test]
fn fixture_nobr() { fixture_test("nobr"); }

#[test]
fn fixture_noembed() { fixture_test("noembed"); }

#[test]
fn fixture_noscript() { fixture_test("noscript"); }

#[test]
fn fixture_object() { fixture_test("object"); }

#[test]
fn fixture_ol() { fixture_test("ol"); }

#[test]
fn fixture_output() { fixture_test("output"); }

#[test]
fn fixture_paragraph() { fixture_test("paragraph"); }

#[test]
fn fixture_paragraph_implicit() { fixture_test("paragraph-implicit"); }

#[test]
fn fixture_picture() { fixture_test("picture"); }

#[test]
fn fixture_plaintext() { fixture_test("plaintext"); }

#[test]
fn fixture_pre() { fixture_test("pre"); }

#[test]
fn fixture_progress() { fixture_test("progress"); }

#[test]
fn fixture_q() { fixture_test("q"); }

#[test]
fn fixture_quotes() { fixture_test("quotes"); }

#[test]
fn fixture_quotes_alt() { fixture_test("quotes-alt"); }

#[test]
fn fixture_root() { fixture_test("root"); }

#[test]
fn fixture_ruby_rt_rp_rbc_rtc_rb() { fixture_test("ruby-rt-rp-rbc-rtc-rb"); }

#[test]
fn fixture_s() { fixture_test("s"); }

#[test]
fn fixture_samp() { fixture_test("samp"); }

#[test]
fn fixture_script() { fixture_test("script"); }

#[test]
fn fixture_select_optgroup_option() { fixture_test("select-optgroup-option"); }

#[test]
fn fixture_shadow() { fixture_test("shadow"); }

#[test]
fn fixture_slot() { fixture_test("slot"); }

#[test]
fn fixture_small() { fixture_test("small"); }

#[test]
fn fixture_spacer() { fixture_test("spacer"); }

#[test]
fn fixture_span() { fixture_test("span"); }

#[test]
fn fixture_straddling() { fixture_test("straddling"); }

#[test]
fn fixture_strike() { fixture_test("strike"); }

#[test]
fn fixture_strong() { fixture_test("strong"); }

#[test]
fn fixture_style() { fixture_test("style"); }

#[test]
fn fixture_sup_sub() { fixture_test("sup-sub"); }

#[test]
fn fixture_svg() { fixture_test("svg"); }

#[test]
fn fixture_table() { fixture_test("table"); }

#[test]
fn fixture_table_extra_elements() { fixture_test("table-extra-elements"); }

#[test]
fn fixture_table_headless() { fixture_test("table-headless"); }

#[test]
fn fixture_table_in_table() { fixture_test("table-in-table"); }

#[test]
fn fixture_table_missing_elements() { fixture_test("table-missing-elements"); }

#[test]
fn fixture_table_rowspan() { fixture_test("table-rowspan"); }

#[test]
fn fixture_template() { fixture_test("template"); }

#[test]
fn fixture_text_wrap() { fixture_test("text-wrap"); }

#[test]
fn fixture_textarea() { fixture_test("textarea"); }

#[test]
fn fixture_time() { fixture_test("time"); }

#[test]
fn fixture_title() { fixture_test("title"); }

#[test]
fn fixture_tt() { fixture_test("tt"); }

#[test]
fn fixture_u() { fixture_test("u"); }

#[test]
fn fixture_ul() { fixture_test("ul"); }

#[test]
fn fixture_ul_ul() { fixture_test("ul-ul"); }

#[test]
fn fixture_var() { fixture_test("var"); }

#[test]
fn fixture_video() { fixture_test("video"); }

#[test]
fn fixture_wbr() { fixture_test("wbr"); }

#[test]
fn fixture_whitespace() { fixture_test("whitespace"); }

#[test]
fn fixture_xmp() { fixture_test("xmp"); }
