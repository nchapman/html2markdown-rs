using System.Reflection;
using Xunit;
using uniffi.html2markdown_uniffi;

namespace Html2MarkdownTests;

public class ConvertTest
{
    [Fact]
    public void ConvertsHeading()
    {
        Assert.Equal("# Hello\n", Html2markdownUniffiMethods.Convert("<h1>Hello</h1>"));
    }

    [Fact]
    public void ReturnsEmptyForEmptyInput()
    {
        Assert.Equal("", Html2markdownUniffiMethods.Convert(""));
    }

    [Fact]
    public void ConvertsParagraph()
    {
        Assert.Equal("Hello\n", Html2markdownUniffiMethods.Convert("<p>Hello</p>"));
    }

    [Fact]
    public void ConvertsEmphasis()
    {
        Assert.Equal("*Hello World.*\n", Html2markdownUniffiMethods.Convert("<em>Hello World.</em>"));
    }

    [Fact]
    public void ConvertsStrong()
    {
        Assert.Equal("**Hello World.**\n", Html2markdownUniffiMethods.Convert("<strong>Hello World.</strong>"));
    }

    [Fact]
    public void ConvertsLink()
    {
        var html = "<a href=\"http://example.com\" title=\"example\">example</a>";
        Assert.Equal("[example](http://example.com \"example\")\n", Html2markdownUniffiMethods.Convert(html));
    }

    [Fact]
    public void ConvertsImage()
    {
        var html = "<img src=\"http://example.com\" alt=\"example\">";
        Assert.Equal("![example](http://example.com)\n", Html2markdownUniffiMethods.Convert(html));
    }

    [Fact]
    public void ConvertsInlineCode()
    {
        Assert.Equal("`toString()`\n", Html2markdownUniffiMethods.Convert("<code>toString()</code>"));
    }

    [Fact]
    public void ConvertsBlockquote()
    {
        var html = "<blockquote><p>This is a blockquote.</p></blockquote>";
        Assert.Equal("> This is a blockquote.\n", Html2markdownUniffiMethods.Convert(html));
    }

    [Fact]
    public void ConvertsUnorderedList()
    {
        var html = "<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>";
        Assert.Equal("* Alpha\n* Bravo\n* Charlie\n", Html2markdownUniffiMethods.Convert(html));
    }

    [Fact]
    public void ConvertsOrderedList()
    {
        var html = "<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>";
        Assert.Equal("1. Alpha\n2. Bravo\n3. Charlie\n", Html2markdownUniffiMethods.Convert(html));
    }

    [Fact]
    public void DefaultOptionsMatchesConvert()
    {
        var html = "<h1>Hello</h1>";
        var expected = Html2markdownUniffiMethods.Convert(html);
        var actual = Html2markdownUniffiMethods.ConvertWith(html, Html2markdownUniffiMethods.DefaultOptions());
        Assert.Equal(expected, actual);
    }
}

public class DefaultOptionsTest
{
    [Fact]
    public void StringifyOptionsHaveExpectedDefaults()
    {
        var opts = Html2markdownUniffiMethods.DefaultStringifyOptions();
        Assert.Equal(HeadingStyle.Atx, opts.headingStyle);
        Assert.Equal("*", opts.bullet);
        Assert.Equal(".", opts.bulletOrdered);
        Assert.Equal("*", opts.emphasis);
        Assert.Equal("*", opts.strong);
        Assert.Equal("`", opts.fence);
        Assert.Equal("*", opts.rule);
        Assert.Equal((byte)3, opts.ruleRepetition);
        Assert.False(opts.ruleSpaces);
        Assert.False(opts.closeAtx);
        Assert.Equal(ListItemIndent.One, opts.listItemIndent);
        Assert.True(opts.incrementListMarker);
        Assert.Equal("\"", opts.quote);
        Assert.True(opts.fences);
        Assert.False(opts.resourceLink);
    }

    [Fact]
    public void ConversionOptionsHaveExpectedDefaults()
    {
        var opts = Html2markdownUniffiMethods.DefaultOptions();
        Assert.False(opts.newlines);
        Assert.Null(opts.@checked);
        Assert.Null(opts.@unchecked);
        Assert.Equal(new[] { "\"" }, opts.quotes);
    }
}

public class ErrorHandlingTest
{
    [Fact]
    public void InvalidBulletThrowsOptionsException()
    {
        var stringify = Html2markdownUniffiMethods.DefaultStringifyOptions() with { bullet = "x" };
        var opts = Html2markdownUniffiMethods.DefaultOptions() with { stringify = stringify };
        var ex = Assert.Throws<OptionsException.InvalidOption>(() =>
            Html2markdownUniffiMethods.ConvertWith("<p>hi</p>", opts));
        Assert.Equal("bullet", ex.field);
        Assert.Equal("x", ex.value);
    }

    [Fact]
    public void EmptyBulletThrowsOptionsException()
    {
        var stringify = Html2markdownUniffiMethods.DefaultStringifyOptions() with { bullet = "" };
        var opts = Html2markdownUniffiMethods.DefaultOptions() with { stringify = stringify };
        Assert.Throws<OptionsException.InvalidOption>(() =>
            Html2markdownUniffiMethods.ConvertWith("<p>hi</p>", opts));
    }
}

public class FixturesTest
{
    private static readonly string FixturesDir =
        Assembly.GetExecutingAssembly()
            .GetCustomAttributes<AssemblyMetadataAttribute>()
            .First(a => a.Key == "FixturesDir").Value!;

    private static readonly string[] FixtureNames =
    [
        "a", "blockquote", "br", "code", "em", "heading",
        "img", "ol", "paragraph", "strong", "table", "ul"
    ];

    public static TheoryData<string> FixtureData
    {
        get
        {
            var data = new TheoryData<string>();
            foreach (var name in FixtureNames)
                data.Add(name);
            return data;
        }
    }

    [Theory]
    [MemberData(nameof(FixtureData))]
    public void MatchesExpectedMarkdown(string name)
    {
        var dir = Path.Combine(FixturesDir, name);
        Assert.True(Directory.Exists(dir), $"Fixture directory not found: {dir}");

        var html = File.ReadAllText(Path.Combine(dir, "index.html"));
        var expectedMd = File.ReadAllText(Path.Combine(dir, "index.md"));

        var result = Html2markdownUniffiMethods.Convert(html);
        Assert.Equal(expectedMd, result);
    }
}
