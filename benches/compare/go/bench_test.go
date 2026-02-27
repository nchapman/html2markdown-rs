// Benchmark the JohannesKaufmann/html-to-markdown Go implementation.
//
// Run from this directory:
//
//	go test -bench=. -benchmem -benchtime=5s
//
// The fixture files are loaded from ../../fixtures/ (benches/fixtures/ in the Rust project).
package bench

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/JohannesKaufmann/html-to-markdown/v2/converter"
	"github.com/JohannesKaufmann/html-to-markdown/v2/plugin/base"
	"github.com/JohannesKaufmann/html-to-markdown/v2/plugin/commonmark"
)

var fixtureNames = []string{"article", "table", "lists", "code", "large"}

// loadFixtures reads HTML content from ../../fixtures/<name>.html
func loadFixtures(tb testing.TB) map[string]string {
	tb.Helper()
	fixtures := make(map[string]string, len(fixtureNames))
	for _, name := range fixtureNames {
		path := filepath.Join("..", "..", "fixtures", name+".html")
		data, err := os.ReadFile(path)
		if err != nil {
			tb.Fatalf("load fixture %q: %v", name, err)
		}
		fixtures[name] = string(data)
	}
	return fixtures
}

// newConverter builds a converter equivalent to the default ConvertString setup.
func newConverter() *converter.Converter {
	return converter.NewConverter(
		converter.WithPlugins(
			base.NewBasePlugin(),
			commonmark.NewCommonmarkPlugin(),
		),
	)
}

func BenchmarkConvert(b *testing.B) {
	fixtures := loadFixtures(b)
	conv := newConverter()

	for _, name := range fixtureNames {
		html := fixtures[name]
		b.Run(name, func(b *testing.B) {
			b.SetBytes(int64(len(html)))
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				if _, err := conv.ConvertString(html); err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}
