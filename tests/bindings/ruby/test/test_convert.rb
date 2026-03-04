# frozen_string_literal: true

require_relative "test_helper"

class TestConvert < Minitest::Test
  def test_heading
    assert_equal "# Hello\n", Html2markdownUniffi.convert("<h1>Hello</h1>")
  end

  def test_empty_string
    assert_equal "", Html2markdownUniffi.convert("")
  end

  def test_paragraph
    assert_equal "Hello\n", Html2markdownUniffi.convert("<p>Hello</p>")
  end

  def test_emphasis
    assert_equal "*Hello World.*\n", Html2markdownUniffi.convert("<em>Hello World.</em>")
  end

  def test_strong
    assert_equal "**Hello World.**\n", Html2markdownUniffi.convert("<strong>Hello World.</strong>")
  end

  def test_link
    html = '<a href="http://example.com" title="example">example</a>'
    assert_equal "[example](http://example.com \"example\")\n", Html2markdownUniffi.convert(html)
  end

  def test_image
    html = '<img src="http://example.com" alt="example">'
    assert_equal "![example](http://example.com)\n", Html2markdownUniffi.convert(html)
  end

  def test_code
    assert_equal "`toString()`\n", Html2markdownUniffi.convert("<code>toString()</code>")
  end

  def test_blockquote
    html = "<blockquote><p>This is a blockquote.</p></blockquote>"
    assert_equal "> This is a blockquote.\n", Html2markdownUniffi.convert(html)
  end

  def test_unordered_list
    html = "<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>"
    assert_equal "* Alpha\n* Bravo\n* Charlie\n", Html2markdownUniffi.convert(html)
  end

  def test_ordered_list
    html = "<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>"
    assert_equal "1. Alpha\n2. Bravo\n3. Charlie\n", Html2markdownUniffi.convert(html)
  end
end

class TestConvertWith < Minitest::Test
  def test_default_options_matches_convert
    html = "<h1>Hello</h1>"
    expected = Html2markdownUniffi.convert(html)
    actual = Html2markdownUniffi.convert_with(html, Html2markdownUniffi.default_options)
    assert_equal expected, actual
  end
end

class TestDefaultOptions < Minitest::Test
  def test_stringify_options
    opts = Html2markdownUniffi.default_stringify_options
    assert_equal Html2markdownUniffi::HeadingStyle::ATX, opts.heading_style
    assert_equal "*", opts.bullet
    assert_equal ".", opts.bullet_ordered
    assert_equal "*", opts.emphasis
    assert_equal "*", opts.strong
    assert_equal "`", opts.fence
    assert_equal "*", opts.rule
    assert_equal 3, opts.rule_repetition
    assert_equal false, opts.rule_spaces
    assert_equal false, opts.close_atx
    assert_equal Html2markdownUniffi::ListItemIndent::ONE, opts.list_item_indent
    assert_equal true, opts.increment_list_marker
    assert_equal '"', opts.quote
    assert_equal true, opts.fences
    assert_equal false, opts.resource_link
  end

  def test_conversion_options
    opts = Html2markdownUniffi.default_options
    assert_equal false, opts.newlines
    assert_nil opts.checked
    assert_nil opts.unchecked
    assert_equal ['"'], opts.quotes
  end
end

class TestErrorHandling < Minitest::Test
  def test_invalid_bullet_raises
    err = assert_raises(Html2markdownUniffi::OptionsError::InvalidOption) do
      Html2markdownUniffi.convert_with("<p>hi</p>", build_options(bullet: "x"))
    end
    assert_equal "bullet", err.field
    assert_equal "x", err.value
  end

  def test_empty_bullet_raises
    assert_raises(Html2markdownUniffi::OptionsError::InvalidOption) do
      Html2markdownUniffi.convert_with("<p>hi</p>", build_options(bullet: ""))
    end
  end

  private

  # Build an Options with default values, overriding specific stringify fields.
  # The generated StringifyOptions uses attr_reader (no setters), so we must
  # reconstruct the full object to change any field.
  def build_options(**overrides)
    defaults = Html2markdownUniffi.default_stringify_options
    stringify = Html2markdownUniffi::StringifyOptions.new(
      heading_style: defaults.heading_style,
      bullet: defaults.bullet,
      bullet_ordered: defaults.bullet_ordered,
      emphasis: defaults.emphasis,
      strong: defaults.strong,
      fence: defaults.fence,
      rule: defaults.rule,
      rule_repetition: defaults.rule_repetition,
      rule_spaces: defaults.rule_spaces,
      close_atx: defaults.close_atx,
      list_item_indent: defaults.list_item_indent,
      increment_list_marker: defaults.increment_list_marker,
      quote: defaults.quote,
      fences: defaults.fences,
      resource_link: defaults.resource_link,
      **overrides
    )
    opts = Html2markdownUniffi.default_options
    Html2markdownUniffi::Options.new(
      stringify: stringify,
      newlines: opts.newlines,
      checked: opts.checked,
      unchecked: opts.unchecked,
      quotes: opts.quotes
    )
  end
end

class TestFixtures < Minitest::Test
  FIXTURES_DIR = File.expand_path("../../../../test-fixtures", __dir__).freeze

  FIXTURE_NAMES = %w[
    a blockquote br code em heading
    img ol paragraph strong table ul
  ].freeze

  FIXTURE_NAMES.each do |name|
    define_method("test_fixture_#{name}") do
      dir = File.join(FIXTURES_DIR, name)
      assert File.directory?(dir), "Fixture dir not found: #{dir}"

      html = File.read(File.join(dir, "index.html"))
      expected_md = File.read(File.join(dir, "index.md"))
      config = JSON.parse(File.read(File.join(dir, "index.json")))

      skip "non-fragment fixtures not tested via bindings" unless config["fragment"]

      result = Html2markdownUniffi.convert(html)
      assert_equal expected_md, result, "Fixture '#{name}' mismatch"
    end
  end
end
