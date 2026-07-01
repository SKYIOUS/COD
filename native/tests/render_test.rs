use cod_native;

#[test]
fn test_render_plain_line() {
    let html = cod_native::render_line_html(
        "hello world".to_string(),
        "[]".to_string(),
        "[]".to_string(),
    );
    assert_eq!(html, "<span>hello world</span>");
}

#[test]
fn test_render_with_tokens() {
    let tokens = r#"[{"start":0,"end":5,"className":"keyword"}]"#;
    let html = cod_native::render_line_html(
        "hello world".to_string(),
        tokens.to_string(),
        "[]".to_string(),
    );
    assert!(html.contains("<span class=\"keyword\">hello</span>"));
}

#[test]
fn test_render_html_escapes() {
    let tokens = r#"[{"start":0,"end":5,"className":"tag"}]"#;
    let html =
        cod_native::render_line_html("<div>".to_string(), tokens.to_string(), "[]".to_string());
    assert!(html.contains("&lt;div&gt;"));
}

#[test]
fn test_render_with_decorations() {
    let decos = r#"[{"start":0,"end":5,"className":"diff-inserted","isInline":true}]"#;
    let html = cod_native::render_line_html(
        "hello world".to_string(),
        "[]".to_string(),
        decos.to_string(),
    );
    assert!(html.contains("diff-inserted"));
}

#[test]
fn test_render_lines_batch() {
    let lines = vec!["line one".to_string(), "line two".to_string()];
    let all_tokens = r#"[[{"start":0,"end":4,"className":"keyword"}],[{"start":5,"end":8,"className":"string"}]]"#;
    let results =
        cod_native::render_lines_html(lines, all_tokens.to_string(), "[[],[]]".to_string());
    assert_eq!(results.len(), 2);
    assert!(results[0].contains("keyword"));
    assert!(results[1].contains("string"));
}

#[test]
fn test_render_empty_token_list() {
    let html = cod_native::render_line_html("".to_string(), "[]".to_string(), "[]".to_string());
    assert_eq!(html, "<span></span>");
}

#[test]
fn test_render_minimap_basic() {
    let tokens = r#"[{"start":0,"end":4,"className":"keyword"}]"#;
    let out = cod_native::render_minimap_line("test".to_string(), tokens.to_string(), 1);
    assert!(!out.is_empty());
}

#[test]
fn test_render_lines_native() {
    use cod_native::BatchLineInput;
    let inputs = vec![
        BatchLineInput {
            line_content: "hello".into(),
            parts_json: r#"[{"end_index":5,"type":"","metadata":0,"containsRTL":false}]"#.into(),
            start_visible_column: 0,
            is_overflowing: false,
            overflowing_char_count: 0,
            len_val: 5,
        },
        BatchLineInput {
            line_content: "world".into(),
            parts_json: r#"[{"end_index":5,"type":"","metadata":0,"containsRTL":false}]"#.into(),
            start_visible_column: 0,
            is_overflowing: false,
            overflowing_char_count: 0,
            len_val: 5,
        },
    ];
    let results = cod_native::render_lines_native(inputs, 4, 0, 8, 0, 0, false, false, true, 0);
    assert_eq!(results.len(), 2);
    assert!(
        results[0].html.contains("hello"),
        "result[0] html: {}",
        results[0].html
    );
    assert!(
        results[1].html.contains("world"),
        "result[1] html: {}",
        results[1].html
    );
}
