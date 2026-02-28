import uniffi.html2markdown_uniffi.*
import java.io.File
import kotlin.test.*
import org.junit.jupiter.api.Nested
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class ConvertTest {

    @Nested inner class Convert {

        @Test fun `heading`() {
            assertEquals("# Hello\n", convert("<h1>Hello</h1>"))
        }

        @Test fun `empty string`() {
            assertEquals("", convert(""))
        }

        @Test fun `paragraph`() {
            assertEquals("Hello\n", convert("<p>Hello</p>"))
        }

        @Test fun `emphasis`() {
            assertEquals("*Hello World.*\n", convert("<em>Hello World.</em>"))
        }

        @Test fun `strong`() {
            assertEquals("**Hello World.**\n", convert("<strong>Hello World.</strong>"))
        }

        @Test fun `link`() {
            val html = """<a href="http://example.com" title="example">example</a>"""
            assertEquals("[example](http://example.com \"example\")\n", convert(html))
        }

        @Test fun `image`() {
            val html = """<img src="http://example.com" alt="example">"""
            assertEquals("![example](http://example.com)\n", convert(html))
        }

        @Test fun `code`() {
            assertEquals("`toString()`\n", convert("<code>toString()</code>"))
        }

        @Test fun `blockquote`() {
            val html = "<blockquote><p>This is a blockquote.</p></blockquote>"
            assertEquals("> This is a blockquote.\n", convert(html))
        }

        @Test fun `unordered list`() {
            val html = "<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>"
            assertEquals("* Alpha\n* Bravo\n* Charlie\n", convert(html))
        }

        @Test fun `ordered list`() {
            val html = "<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>"
            assertEquals("1. Alpha\n2. Bravo\n3. Charlie\n", convert(html))
        }
    }

    @Nested inner class ConvertWith {

        @Test fun `default options matches convert`() {
            val html = "<h1>Hello</h1>"
            val expected = convert(html)
            val actual = convertWith(html, defaultOptions())
            assertEquals(expected, actual)
        }
    }

    @Nested inner class DefaultOptions {

        @Test fun `stringify options`() {
            val opts = defaultStringifyOptions()
            assertEquals(HeadingStyle.ATX, opts.headingStyle)
            assertEquals("*", opts.bullet)
            assertEquals(".", opts.bulletOrdered)
            assertEquals("*", opts.emphasis)
            assertEquals("*", opts.strong)
            assertEquals("`", opts.fence)
            assertEquals("*", opts.rule)
            assertEquals(3.toUByte(), opts.ruleRepetition)
            assertFalse(opts.ruleSpaces)
            assertFalse(opts.closeAtx)
            assertEquals(ListItemIndent.ONE, opts.listItemIndent)
            assertTrue(opts.incrementListMarker)
            assertEquals("\"", opts.quote)
            assertTrue(opts.fences)
            assertFalse(opts.resourceLink)
        }

        @Test fun `conversion options`() {
            val opts = defaultOptions()
            assertFalse(opts.newlines)
            assertNull(opts.checked)
            assertNull(opts.unchecked)
            assertEquals(listOf("\""), opts.quotes)
        }
    }

    @Nested inner class ErrorHandling {

        @Test fun `invalid bullet throws OptionsException`() {
            val opts = defaultOptions().copy(
                stringify = defaultStringifyOptions().copy(bullet = "x")
            )
            val ex = assertFailsWith<OptionsException.InvalidOption> {
                convertWith("<p>hi</p>", opts)
            }
            assertEquals("bullet", ex.field)
            assertEquals("x", ex.value)
        }

        @Test fun `empty bullet throws OptionsException`() {
            val opts = defaultOptions().copy(
                stringify = defaultStringifyOptions().copy(bullet = "")
            )
            assertFailsWith<OptionsException.InvalidOption> {
                convertWith("<p>hi</p>", opts)
            }
        }
    }

    @Nested inner class Fixtures {

        private val fixturesDir: File by lazy {
            val path = System.getProperty("fixtures.dir")
                ?: error("System property 'fixtures.dir' not set — run via Gradle")
            File(path).also {
                assertTrue(it.isDirectory, "Fixtures directory not found: $it")
            }
        }

        @ParameterizedTest
        @ValueSource(strings = [
            "a", "blockquote", "br", "code", "em", "heading",
            "img", "ol", "paragraph", "strong", "table", "ul",
        ])
        fun `matches expected markdown`(name: String) {
            val dir = fixturesDir.resolve(name)
            assertTrue(dir.isDirectory, "Fixture directory not found: $dir")

            val html = dir.resolve("index.html").readText()
            val expectedMd = dir.resolve("index.md").readText()

            val result = convert(html)
            assertEquals(expectedMd, result, "Fixture '$name' mismatch")
        }
    }
}
