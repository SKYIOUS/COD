use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_render_plain_line(c: &mut Criterion) {
    let line = "fn hello(a: i32, b: i32) -> i32 { return a + b; }".repeat(10);
    c.bench_function("render_plain_400ch", |b| {
        b.iter(|| {
            cod_native::render_line_html(
                black_box(line.clone()),
                black_box("[]".into()),
                black_box("[]".into()),
            )
        })
    });
}

fn bench_render_with_tokens(c: &mut Criterion) {
    let line = "    pub fn hello_world(a: i32, b: i32) -> i32 { return a + b; }";
    let tokens = r#"[
        {"start":4,"end":7,"className":"keyword"},
        {"start":8,"end":10,"className":"keyword"},
        {"start":11,"end":22,"className":"entity.name.function"},
        {"start":30,"end":31,"className":"keyword"},
        {"start":36,"end":37,"className":"keyword"},
        {"start":40,"end":46,"className":"keyword"}
    ]"#;
    c.bench_function("render_60ch_6tokens", |b| {
        b.iter(|| {
            cod_native::render_line_html(
                black_box(line.into()),
                black_box(tokens.into()),
                black_box("[]".into()),
            )
        })
    });
}

fn bench_render_with_decorations(c: &mut Criterion) {
    let line = "    check_something(42);";
    let tokens = r#"[{"start":4,"end":18,"className":"entity.name.function"}]"#;
    let decorations = r#"[{"start":0,"end":4,"className":"diffAdded","isInline":false}]"#;
    c.bench_function("render_25ch_tokens+deco", |b| {
        b.iter(|| {
            cod_native::render_line_html(
                black_box(line.into()),
                black_box(tokens.into()),
                black_box(decorations.into()),
            )
        })
    });
}

fn bench_render_native_simple(c: &mut Criterion) {
    let line = "fn hello(a: i32, b: i32) -> i32 { return a + b; }";
    let parts = r#"[{"endIndex":65,"type":"","metadata":0,"containsRTL":false}]"#;
    c.bench_function("render_native_60ch", |b| {
        b.iter(|| {
            cod_native::render_line_native(
                black_box(line.into()),
                black_box(parts.into()),
                black_box(4),
                black_box(0),
                black_box(0),
                black_box(10),
                black_box(183),
                black_box(0),
                black_box(false),
                black_box(true),
                black_box(true),
                black_box(0),
                black_box(false),
                black_box(0),
                black_box(65),
            )
        })
    });
}

fn bench_render_native_tokens(c: &mut Criterion) {
    let line = "    pub fn hello_world(a: i32, b: i32) -> i32 { return a + b; }";
    let parts = r#"[
        {"endIndex":4,"type":"","metadata":0,"containsRTL":false},
        {"endIndex":7,"type":"mtkki","metadata":0,"containsRTL":false},
        {"endIndex":10,"type":"mtkk2","metadata":0,"containsRTL":false},
        {"endIndex":22,"type":"mtknm","metadata":0,"containsRTL":false},
        {"endIndex":31,"type":"","metadata":0,"containsRTL":false},
        {"endIndex":32,"type":"mtkki","metadata":0,"containsRTL":false},
        {"endIndex":37,"type":"","metadata":0,"containsRTL":false},
        {"endIndex":38,"type":"mtkki","metadata":0,"containsRTL":false},
        {"endIndex":46,"type":"","metadata":0,"containsRTL":false},
        {"endIndex":60,"type":"mtkki","metadata":0,"containsRTL":false},
        {"endIndex":61,"type":"","metadata":0,"containsRTL":false}
    ]"#;
    c.bench_function("render_native_60ch_10parts", |b| {
        b.iter(|| {
            cod_native::render_line_native(
                black_box(line.into()),
                black_box(parts.into()),
                black_box(4),
                black_box(4),
                black_box(0),
                black_box(10),
                black_box(183),
                black_box(0),
                black_box(false),
                black_box(true),
                black_box(true),
                black_box(0),
                black_box(false),
                black_box(0),
                black_box(60),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_render_plain_line,
    bench_render_with_tokens,
    bench_render_with_decorations,
    bench_render_native_simple,
    bench_render_native_tokens
);
criterion_main!(benches);
