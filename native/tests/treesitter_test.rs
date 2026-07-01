use cod_native;

fn js_query(source: &str, query: &str) -> Vec<cod_native::TreeSitterCapture> {
    let result =
        cod_native::query_tree_sitter(source.to_string(), "javascript".into(), query.into());
    assert!(result.error.is_empty(), "Query failed: {}", result.error);
    result.captures
}

#[test]
fn test_unsupported_language() {
    let result =
        cod_native::query_tree_sitter("x".into(), "coffeescript".into(), "(identifier)".into());
    assert!(!result.error.is_empty());
    assert!(result.error.contains("Unsupported language"));
}

#[test]
fn test_parse_javascript() {
    let result =
        cod_native::parse_with_tree_sitter("function foo() {}".into(), "javascript".into());
    assert!(result.error.is_empty(), "Parse failed: {}", result.error);
    assert!(!result.captures.is_empty(), "Should have captured nodes");
}

#[test]
fn test_parse_python() {
    let result = cod_native::parse_with_tree_sitter("def foo():\n    pass".into(), "python".into());
    assert!(result.error.is_empty(), "Parse failed: {}", result.error);
    assert!(!result.captures.is_empty());
}

#[test]
fn test_parse_rust() {
    let result = cod_native::parse_with_tree_sitter("fn main() {}".into(), "rust".into());
    assert!(result.error.is_empty(), "Parse failed: {}", result.error);
    assert!(!result.captures.is_empty());
}

#[test]
fn test_query_javascript_function() {
    let captures = js_query(
        "function foo() {}",
        "(function_declaration name: (identifier) @name)",
    );
    assert!(!captures.is_empty(), "Should capture function name");
    assert!(captures.iter().any(|c| c.type_name == "name"));
}

#[test]
fn test_query_javascript_string() {
    let captures = js_query("var x = \"hello\";", "(string) @str");
    assert!(!captures.is_empty(), "Should capture string node");
    assert!(captures.iter().any(|c| c.type_name == "str"));
}

#[test]
fn test_query_rust_function() {
    let result = cod_native::query_tree_sitter(
        "fn greet(name: &str) -> String { format!(\"hi {}\", name) }".into(),
        "rust".into(),
        "(function_item name: (identifier) @fn_name)".into(),
    );
    assert!(result.error.is_empty(), "Query failed: {}", result.error);
    assert!(!result.captures.is_empty(), "Should capture function name");
}
