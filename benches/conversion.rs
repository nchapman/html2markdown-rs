// Benchmarks for html-to-markdown conversion.

use criterion::{criterion_group, criterion_main, Criterion};
use html_to_markdown::convert;

fn bench_simple(c: &mut Criterion) {
    let html = "<h1>Hello</h1><p>This is a <strong>simple</strong> document.</p>";
    c.bench_function("simple_document", |b| {
        b.iter(|| convert(html).unwrap());
    });
}

criterion_group!(benches, bench_simple);
criterion_main!(benches);
