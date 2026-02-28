import XCTest
import Html2Markdown

final class ConvertTests: XCTestCase {

    override func setUp() {
        super.setUp()
        continueAfterFailure = true
    }

    // MARK: - convert()

    func testConvertHeading() {
        XCTAssertEqual(convert(html: "<h1>Hello</h1>"), "# Hello\n")
    }

    func testConvertEmptyString() {
        XCTAssertEqual(convert(html: ""), "")
    }

    func testConvertParagraph() {
        XCTAssertEqual(convert(html: "<p>Hello</p>"), "Hello\n")
    }

    func testConvertEmphasis() {
        XCTAssertEqual(convert(html: "<em>Hello World.</em>"), "*Hello World.*\n")
    }

    func testConvertStrong() {
        XCTAssertEqual(convert(html: "<strong>Hello World.</strong>"), "**Hello World.**\n")
    }

    func testConvertLink() {
        let html = "<a href=\"http://example.com\" title=\"example\">example</a>"
        XCTAssertEqual(convert(html: html), "[example](http://example.com \"example\")\n")
    }

    func testConvertImage() {
        let html = "<img src=\"http://example.com\" alt=\"example\">"
        XCTAssertEqual(convert(html: html), "![example](http://example.com)\n")
    }

    func testConvertCode() {
        XCTAssertEqual(convert(html: "<code>toString()</code>"), "`toString()`\n")
    }

    func testConvertBlockquote() {
        let html = "<blockquote><p>This is a blockquote.</p></blockquote>"
        XCTAssertEqual(convert(html: html), "> This is a blockquote.\n")
    }

    func testConvertUnorderedList() {
        let html = "<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>"
        XCTAssertEqual(convert(html: html), "* Alpha\n* Bravo\n* Charlie\n")
    }

    func testConvertOrderedList() {
        let html = "<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>"
        XCTAssertEqual(convert(html: html), "1. Alpha\n2. Bravo\n3. Charlie\n")
    }

    // MARK: - convert_with()

    func testConvertWithDefaultOptions() throws {
        let html = "<h1>Hello</h1>"
        let result = try convertWith(html: html, options: defaultOptions())
        XCTAssertEqual(result, convert(html: html))
    }

    // MARK: - default options

    func testDefaultStringifyOptions() {
        let opts = defaultStringifyOptions()
        XCTAssertEqual(opts.headingStyle, HeadingStyle.atx)
        XCTAssertEqual(opts.bullet, "*")
        XCTAssertEqual(opts.bulletOrdered, ".")
        XCTAssertEqual(opts.emphasis, "*")
        XCTAssertEqual(opts.strong, "*")
        XCTAssertEqual(opts.fence, "`")
        XCTAssertEqual(opts.rule, "*")
        XCTAssertEqual(opts.ruleRepetition, 3)
        XCTAssertFalse(opts.ruleSpaces)
        XCTAssertFalse(opts.closeAtx)
        XCTAssertTrue(opts.incrementListMarker)
        XCTAssertEqual(opts.quote, "\"")
        XCTAssertTrue(opts.fences)
        XCTAssertFalse(opts.resourceLink)
    }

    func testDefaultOptions() {
        let opts = defaultOptions()
        XCTAssertFalse(opts.newlines)
        XCTAssertNil(opts.checked)
        XCTAssertNil(opts.unchecked)
        XCTAssertEqual(opts.quotes, ["\""])
    }

    // MARK: - error handling

    func testInvalidBulletThrows() {
        var opts = defaultOptions()
        opts.stringify.bullet = "x"
        XCTAssertThrowsError(try convertWith(html: "<p>hi</p>", options: opts)) { error in
            guard case OptionsError.InvalidOption(let field, _, let value) = error else {
                XCTFail("Expected OptionsError.InvalidOption, got \(error)")
                return
            }
            XCTAssertEqual(field, "bullet")
            XCTAssertEqual(value, "x")
        }
    }

    func testEmptyBulletThrows() {
        var opts = defaultOptions()
        opts.stringify.bullet = ""
        XCTAssertThrowsError(try convertWith(html: "<p>hi</p>", options: opts)) { error in
            guard case OptionsError.InvalidOption = error else {
                XCTFail("Expected OptionsError.InvalidOption, got \(error)")
                return
            }
        }
    }

    // MARK: - fixtures

    func testFixtureA() {
        assertFixture("a")
    }

    func testFixtureBlockquote() {
        assertFixture("blockquote")
    }

    func testFixtureBr() {
        assertFixture("br")
    }

    func testFixtureCode() {
        assertFixture("code")
    }

    func testFixtureEm() {
        assertFixture("em")
    }

    func testFixtureHeading() {
        assertFixture("heading")
    }

    func testFixtureImg() {
        assertFixture("img")
    }

    func testFixtureOl() {
        assertFixture("ol")
    }

    func testFixtureParagraph() {
        assertFixture("paragraph")
    }

    func testFixtureStrong() {
        assertFixture("strong")
    }

    func testFixtureTable() {
        assertFixture("table")
    }

    func testFixtureUl() {
        assertFixture("ul")
    }

    // MARK: - helpers

    private func assertFixture(_ name: String, file: StaticString = #filePath, line: UInt = #line) {
        let fixturesDir = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // → Html2MarkdownTests/
            .deletingLastPathComponent() // → Tests/
            .deletingLastPathComponent() // → swift/
            .deletingLastPathComponent() // → bindings/
            .deletingLastPathComponent() // → tests/
            .deletingLastPathComponent() // → html2markdown-rs/
            .appendingPathComponent("test-fixtures")
            .appendingPathComponent(name)
            .standardized

        let htmlURL = fixturesDir.appendingPathComponent("index.html")
        let mdURL = fixturesDir.appendingPathComponent("index.md")

        let html: String
        do {
            html = try String(contentsOf: htmlURL, encoding: .utf8)
        } catch {
            XCTFail("Could not load fixture '\(name)' HTML at \(htmlURL.path): \(error)", file: file, line: line)
            return
        }

        let expectedMd: String
        do {
            expectedMd = try String(contentsOf: mdURL, encoding: .utf8)
        } catch {
            XCTFail("Could not load fixture '\(name)' MD at \(mdURL.path): \(error)", file: file, line: line)
            return
        }

        let result = convert(html: html)
        XCTAssertEqual(result, expectedMd, "Fixture '\(name)' mismatch", file: file, line: line)
    }
}
