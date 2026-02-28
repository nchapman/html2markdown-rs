"""Tests for convert() and convert_with() API."""

import pytest

from html2markdown_uniffi import (
    OptionsError,
    convert,
    convert_with,
    default_options,
)


def test_convert_heading():
    assert convert("<h1>Hello</h1>") == "# Hello\n"


def test_convert_empty_string():
    assert convert("") == ""


def test_convert_with_default_options():
    html = "<h1>Hello</h1>"
    assert convert_with(html, default_options()) == convert(html)


def test_convert_paragraph():
    assert convert("<p>Hello</p>") == "Hello\n"


def test_convert_emphasis():
    assert convert("<em>Hello World.</em>") == "*Hello World.*\n"


def test_convert_strong():
    assert convert("<strong>Hello World.</strong>") == "**Hello World.**\n"


def test_convert_link():
    html = '<a href="http://example.com" title="example">example</a>'
    assert convert(html) == '[example](http://example.com "example")\n'


def test_convert_image():
    html = '<img src="http://example.com" alt="example">'
    assert convert(html) == "![example](http://example.com)\n"


def test_convert_code():
    assert convert("<code>toString()</code>") == "`toString()`\n"


def test_convert_blockquote():
    html = "<blockquote><p>This is a blockquote.</p></blockquote>"
    assert convert(html) == "> This is a blockquote.\n"


def test_convert_unordered_list():
    html = "<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>"
    assert convert(html) == "* Alpha\n* Bravo\n* Charlie\n"


def test_convert_ordered_list():
    html = "<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>"
    assert convert(html) == "1. Alpha\n2. Bravo\n3. Charlie\n"


def test_convert_invalid_option_raises():
    opts = default_options()
    opts.stringify.bullet = "x"
    with pytest.raises(OptionsError.InvalidOption):
        convert_with("<p>hi</p>", opts)


def test_convert_empty_option_raises():
    opts = default_options()
    opts.stringify.bullet = ""
    with pytest.raises(OptionsError.InvalidOption):
        convert_with("<p>hi</p>", opts)
