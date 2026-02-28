"""Tests for default options and options validation."""

import pytest

from html2markdown_uniffi import (
    HeadingStyle,
    ListItemIndent,
    OptionsError,
    convert_with,
    default_options,
    default_stringify_options,
)


def test_default_stringify_options():
    opts = default_stringify_options()
    assert opts.heading_style == HeadingStyle.ATX
    assert opts.bullet == "*"
    assert opts.bullet_ordered == "."
    assert opts.emphasis == "*"
    assert opts.strong == "*"
    assert opts.fence == "`"
    assert opts.rule == "*"
    assert opts.rule_repetition == 3
    # UniFFI bool fields — explicit `is` checks guard against non-bool truthy values
    assert opts.rule_spaces is False
    assert opts.close_atx is False
    assert opts.list_item_indent == ListItemIndent.ONE
    assert opts.increment_list_marker is True
    assert opts.quote == '"'
    assert opts.fences is True
    assert opts.resource_link is False


def test_default_options():
    opts = default_options()
    assert opts.newlines is False
    assert opts.checked is None
    assert opts.unchecked is None
    assert opts.quotes == ['"']


@pytest.mark.parametrize("char", ["*", "-", "+"])
def test_valid_bullet_chars(char):
    opts = default_options()
    opts.stringify.bullet = char
    result = convert_with("<ul><li>A</li></ul>", opts)
    assert f"{char} A" in result


@pytest.mark.parametrize("char", ["*", "_"])
def test_valid_emphasis_chars(char):
    opts = default_options()
    opts.stringify.emphasis = char
    result = convert_with("<em>A</em>", opts)
    assert f"{char}A{char}" in result


@pytest.mark.parametrize("char", ["`", "~"])
def test_valid_fence_chars(char):
    opts = default_options()
    opts.stringify.fence = char
    opts.stringify.fences = True
    result = convert_with("<pre><code>x</code></pre>", opts)
    assert char * 3 in result


def test_invalid_bullet_ordered():
    opts = default_options()
    opts.stringify.bullet_ordered = "x"
    with pytest.raises(OptionsError.InvalidOption, match="bullet_ordered"):
        convert_with("<ol><li>A</li></ol>", opts)


def test_invalid_rule():
    opts = default_options()
    opts.stringify.rule = "x"
    with pytest.raises(OptionsError.InvalidOption, match="rule"):
        convert_with("<hr>", opts)


def test_invalid_rule_repetition():
    opts = default_options()
    opts.stringify.rule_repetition = 2
    with pytest.raises(OptionsError.InvalidOption, match="rule_repetition"):
        convert_with("<hr>", opts)


def test_setext_headings():
    opts = default_options()
    opts.stringify.heading_style = HeadingStyle.SETEXT
    result = convert_with("<h1>Title</h1>", opts)
    assert "====" in result


def test_close_atx():
    opts = default_options()
    opts.stringify.close_atx = True
    result = convert_with("<h1>Title</h1>", opts)
    assert result.strip() == "# Title #"
