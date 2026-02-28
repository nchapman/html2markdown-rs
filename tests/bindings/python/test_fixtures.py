"""Parametrized fixture tests — same HTML→MD as the Rust test suite."""

import json
from pathlib import Path

import pytest

from html2markdown_uniffi import convert

FIXTURES_DIR = Path(__file__).resolve().parents[3] / "test-fixtures"

FIXTURE_NAMES = [
    "a",
    "blockquote",
    "br",
    "code",
    "em",
    "heading",
    "img",
    "ol",
    "paragraph",
    "strong",
    "table",
    "ul",
]


def _load_fixture(name: str) -> tuple[str, str, dict]:
    d = FIXTURES_DIR / name
    html = (d / "index.html").read_text()
    md = (d / "index.md").read_text()
    config = json.loads((d / "index.json").read_text())
    return html, md, config


@pytest.mark.parametrize("name", FIXTURE_NAMES)
def test_fixture(name: str):
    html, expected_md, config = _load_fixture(name)

    if not config.get("fragment", False):
        pytest.skip("non-fragment fixtures not tested via bindings")

    result = convert(html)
    assert result == expected_md, (
        f"Fixture '{name}' mismatch:\n"
        f"--- expected ---\n{expected_md}\n"
        f"--- got ---\n{result}"
    )
