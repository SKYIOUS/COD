use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ponytail: benchmark the core scope-stacking algorithm
// relies on the napi function working without Node runtime (same as unit tests)

fn bench_tokenize_simple(c: &mut Criterion) {
    let captures = vec![
        cod_native::TokenCapture {
            start: 10,
            end: 20,
            type_name: "keyword".into(),
            language_id: 0,
        },
        cod_native::TokenCapture {
            start: 30,
            end: 40,
            type_name: "string".into(),
            language_id: 0,
        },
    ];
    c.bench_function("tokenize_simple", |b| {
        b.iter(|| {
            cod_native::create_tokens_from_captures_scoped(
                black_box(captures.clone()),
                black_box(0),
                black_box(50),
                black_box("source".into()),
            )
        })
    });
}

fn bench_tokenize_nested(c: &mut Criterion) {
    let captures = vec![
        cod_native::TokenCapture {
            start: 5,
            end: 45,
            type_name: "function".into(),
            language_id: 0,
        },
        cod_native::TokenCapture {
            start: 10,
            end: 40,
            type_name: "keyword".into(),
            language_id: 0,
        },
        cod_native::TokenCapture {
            start: 20,
            end: 30,
            type_name: "string".into(),
            language_id: 0,
        },
    ];
    c.bench_function("tokenize_nested", |b| {
        b.iter(|| {
            cod_native::create_tokens_from_captures_scoped(
                black_box(captures.clone()),
                black_box(0),
                black_box(50),
                black_box("source".into()),
            )
        })
    });
}

fn bench_tokenize_many_captures(c: &mut Criterion) {
    let mut captures = Vec::with_capacity(100);
    for i in 0..100 {
        captures.push(cod_native::TokenCapture {
            start: i * 5,
            end: i * 5 + 3,
            type_name: if i % 2 == 0 {
                "keyword".into()
            } else {
                "string".into()
            },
            language_id: 0,
        });
    }
    c.bench_function("tokenize_100_captures", |b| {
        b.iter(|| {
            cod_native::create_tokens_from_captures_scoped(
                black_box(captures.clone()),
                black_box(0),
                black_box(500),
                black_box("source".into()),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_tokenize_simple,
    bench_tokenize_nested,
    bench_tokenize_many_captures
);
criterion_main!(benches);
