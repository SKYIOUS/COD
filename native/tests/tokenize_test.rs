use cod_native::{
    create_tokens_from_captures_scoped, tokens_to_uint32_array, EndOffsetToken, TokenCapture,
};

fn cap(start: i32, end: i32, type_name: &str) -> TokenCapture {
    TokenCapture {
        start,
        end,
        type_name: type_name.to_string(),
        language_id: 0,
    }
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn parse_result(
    result: &[cod_native::ScopeTokenResult],
) -> Vec<(i32, Vec<String>, Option<Vec<i32>>)> {
    result
        .iter()
        .map(|t| {
            let scopes: Vec<String> = serde_json::from_str(&t.scopes_json).unwrap();
            let bracket: Option<Vec<i32>> = serde_json::from_str(&t.bracket_json).unwrap();
            (t.end_offset, scopes, bracket)
        })
        .collect()
}

#[test]
fn test_empty_captures() {
    let result = create_tokens_from_captures_scoped(vec![], 0, 50, "source".into());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].end_offset, 50);
    assert!(result[0].scopes_json.contains("source"));
}

#[test]
fn test_single_capture_no_gap() {
    let result =
        create_tokens_from_captures_scoped(vec![cap(0, 10, "keyword")], 0, 20, "source".into());
    let parsed = parse_result(&result);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0], (10, s(&["source", "keyword"]), None));
    assert_eq!(parsed[1], (20, s(&["source"]), None));
}

#[test]
fn test_single_capture_with_gap() {
    let result =
        create_tokens_from_captures_scoped(vec![cap(10, 20, "keyword")], 0, 50, "source".into());
    let parsed = parse_result(&result);
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0], (10, s(&["source"]), None));
    assert_eq!(parsed[1], (20, s(&["source", "keyword"]), None));
    assert_eq!(parsed[2], (50, s(&["source"]), None));
}

#[test]
fn test_nested_captures() {
    let result = create_tokens_from_captures_scoped(
        vec![cap(5, 25, "function"), cap(10, 20, "keyword")],
        0,
        30,
        "source".into(),
    );
    let parsed = parse_result(&result);
    assert_eq!(parsed[0], (5, s(&["source"]), None));
    assert!(parsed[1].1.contains(&"function".to_string()));
    assert!(parsed[2].1.contains(&"keyword".to_string()));
    assert_eq!(parsed.last().unwrap().0, 30);
}

#[test]
fn test_no_gap_for_adjacent_tokens() {
    let result = create_tokens_from_captures_scoped(
        vec![cap(0, 10, "keyword"), cap(10, 20, "string")],
        0,
        20,
        "source".into(),
    );
    let parsed = parse_result(&result);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0], (10, s(&["source", "keyword"]), None));
    assert_eq!(parsed[1], (20, s(&["source", "string"]), None));
}

#[test]
fn test_bracket_detection_from_type_name() {
    // bracket detection uses actual bracket chars in type_name
    let result = create_tokens_from_captures_scoped(vec![cap(0, 1, "{")], 0, 10, "source".into());
    let parsed = parse_result(&result);
    assert_eq!(parsed[0].0, 1);
    assert!(parsed[0].2.is_some());
}

#[test]
fn test_bracket_detection_scope_name_has_no_bracket() {
    // scope name like "punctuation.bracket" has no bracket chars -> None
    let result = create_tokens_from_captures_scoped(
        vec![cap(0, 1, "punctuation.bracket")],
        0,
        5,
        "source".into(),
    );
    let parsed = parse_result(&result);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].2, None);
}

#[test]
fn test_capture_partially_outside_range() {
    let result =
        create_tokens_from_captures_scoped(vec![cap(-5, 5, "keyword")], 0, 10, "source".into());
    let parsed = parse_result(&result);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0], (5, s(&["source", "keyword"]), None));
    assert_eq!(parsed[1], (10, s(&["source"]), None));
}

#[test]
fn test_multiple_captures_same_position() {
    let result = create_tokens_from_captures_scoped(
        vec![cap(5, 15, "variable"), cap(5, 15, "parameter")],
        0,
        20,
        "source".into(),
    );
    let parsed = parse_result(&result);
    // gap + merged scope + old shifted zero-length + padding = 4
    assert_eq!(parsed.len(), 4);
    assert_eq!(parsed[0], (5, s(&["source"]), None));
    assert_eq!(
        parsed[1],
        (15, s(&["source", "variable", "parameter"]), None)
    );
    assert_eq!(parsed[3], (20, s(&["source"]), None));
}

#[test]
fn test_tokens_uint32() {
    let tokens = vec![
        EndOffsetToken {
            end_offset: 10,
            metadata: 100,
        },
        EndOffsetToken {
            end_offset: 20,
            metadata: 200,
        },
    ];
    let flat = tokens_to_uint32_array(tokens);
    assert_eq!(flat, vec![10, 100, 20, 200]);
}
