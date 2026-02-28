// Benchmarks for html-to-markdown conversion.
//
// Three groups:
//   full_pipeline  — end-to-end convert() on each fixture
//   transformer    — html_to_mdast() only (parse + tree build, no serialization)
//   serializer     — mdast_to_string() only (pre-built MDAST, no parsing overhead)
//
// Throughput is reported in bytes/sec so results are comparable across fixture sizes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use html2md::parse_html;
use html_to_markdown::{convert, html_to_mdast, mdast_to_string, Options, StringifyOptions};

fn load_fixtures() -> Vec<(&'static str, &'static str)> {
    vec![
        ("article", include_str!("fixtures/article.html")),
        ("table", include_str!("fixtures/table.html")),
        ("lists", include_str!("fixtures/lists.html")),
        ("code", include_str!("fixtures/code.html")),
        ("large", include_str!("fixtures/large.html")),
    ]
}

fn bench_full_pipeline(c: &mut Criterion) {
    let fixtures = load_fixtures();
    let mut group = c.benchmark_group("full_pipeline");

    for (name, html) in &fixtures {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| convert(black_box(html)))
        });
    }

    group.finish();
}

fn bench_transformer(c: &mut Criterion) {
    let fixtures = load_fixtures();
    let options = Options::default();
    let mut group = c.benchmark_group("transformer");

    for (name, html) in &fixtures {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("html_to_mdast", name), html, |b, html| {
            b.iter(|| html_to_mdast(black_box(html), &options))
        });
    }

    group.finish();
}

fn bench_serializer(c: &mut Criterion) {
    let fixtures = load_fixtures();
    let options = Options::default();

    // Pre-build MDAST trees outside the hot loop. Store input byte length for throughput.
    let trees: Vec<(&str, u64, _)> = fixtures
        .iter()
        .map(|(name, html)| (*name, html.len() as u64, html_to_mdast(html, &options)))
        .collect();

    let stringify_options = StringifyOptions::default();
    let mut group = c.benchmark_group("serializer");

    for (name, byte_len, tree) in &trees {
        group.throughput(Throughput::Bytes(*byte_len));
        group.bench_with_input(
            BenchmarkId::new("mdast_to_string", name),
            tree,
            |b, tree| b.iter(|| black_box(mdast_to_string(tree, &stringify_options))),
        );
    }

    group.finish();
}

// NOTE: html2md uses a string-based parser (not an HTML5-compliant DOM).
// Its output quality differs substantially from our converter. This group
// measures raw throughput for context, not architectural equivalence.
fn bench_html2md(c: &mut Criterion) {
    let fixtures = load_fixtures();
    let mut group = c.benchmark_group("html2md");

    for (name, html) in &fixtures {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse_html", name), html, |b, html| {
            b.iter(|| parse_html(black_box(html)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_full_pipeline,
    bench_transformer,
    bench_serializer,
    bench_html2md
);
criterion_main!(benches);
