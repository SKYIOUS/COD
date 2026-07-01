use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[napi(object)]
pub struct TreeSitterCapture {
    pub start: i32,
    pub end: i32,
    pub type_name: String,
}

#[derive(Serialize)]
#[napi(object)]
pub struct TreeSitterQueryResult {
    pub captures: Vec<TreeSitterCapture>,
    pub error: String,
}

fn lang_from_name(name: &str) -> Option<tree_sitter::Language> {
    match name {
        "rust" | "rs" => Some(tree_sitter_rust::language()),
        "typescript" | "ts" => Some(tree_sitter_typescript::language_typescript()),
        "typescriptreact" | "tsx" => Some(tree_sitter_typescript::language_tsx()),
        "javascript" | "js" => Some(tree_sitter_javascript::language()),
        "python" | "py" => Some(tree_sitter_python::language()),
        "go" => Some(tree_sitter_go::language()),
        "java" => Some(tree_sitter_java::language()),
        "json" => Some(tree_sitter_json::language()),
        "c" | "h" => Some(tree_sitter_c::language()),
        "cpp" | "c++" | "hpp" => Some(tree_sitter_cpp::language()),
        "csharp" | "c#" => Some(tree_sitter_c_sharp::language()),
        "css" => Some(tree_sitter_css::language()),
        "bash" | "shell" | "sh" => Some(tree_sitter_bash::language()),
        "ruby" | "rb" => Some(tree_sitter_ruby::language()),
        "php" => Some(tree_sitter_php::language_php()),
        _ => None,
    }
}

#[napi]
pub fn query_tree_sitter(
    source: String,
    language: String,
    query_string: String,
) -> TreeSitterQueryResult {
    let lang = match lang_from_name(&language) {
        Some(l) => l,
        None => {
            return TreeSitterQueryResult {
                captures: Vec::new(),
                error: format!("Unsupported language: {}", language),
            }
        }
    };

    let query = match tree_sitter::Query::new(&lang, &query_string) {
        Ok(q) => q,
        Err(e) => {
            return TreeSitterQueryResult {
                captures: Vec::new(),
                error: format!("Query error: {}", e),
            }
        }
    };

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang).is_err() {
        return TreeSitterQueryResult {
            captures: Vec::new(),
            error: "Failed to set language in parser".to_string(),
        };
    }

    let tree = match parser.parse(&source, None) {
        Some(t) => t,
        None => {
            return TreeSitterQueryResult {
                captures: Vec::new(),
                error: "Failed to parse source".to_string(),
            }
        }
    };

    let root = tree.root_node();
    let mut captures: Vec<TreeSitterCapture> = Vec::new();
    let mut query_cursor = tree_sitter::QueryCursor::new();

    let matches = query_cursor.matches(&query, root, source.as_bytes());
    for match_ in matches {
        for capture in match_.captures {
            let name = query.capture_names()[capture.index as usize].to_string();
            captures.push(TreeSitterCapture {
                start: capture.node.start_byte() as i32,
                end: capture.node.end_byte() as i32,
                type_name: name,
            });
        }
    }

    TreeSitterQueryResult {
        captures,
        error: String::new(),
    }
}

#[napi]
pub fn parse_with_tree_sitter(source: String, language: String) -> TreeSitterQueryResult {
    let lang = match lang_from_name(&language) {
        Some(l) => l,
        None => {
            return TreeSitterQueryResult {
                captures: Vec::new(),
                error: format!("Unsupported language: {}", language),
            }
        }
    };

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang).is_err() {
        return TreeSitterQueryResult {
            captures: Vec::new(),
            error: "Failed to set language in parser".to_string(),
        };
    }

    let tree = match parser.parse(&source, None) {
        Some(t) => t,
        None => {
            return TreeSitterQueryResult {
                captures: Vec::new(),
                error: "Failed to parse source".to_string(),
            }
        }
    };

    let root = tree.root_node();
    let mut captures: Vec<TreeSitterCapture> = Vec::new();
    collect_all_nodes(&root, &mut captures);

    TreeSitterQueryResult {
        captures,
        error: String::new(),
    }
}

fn collect_all_nodes(node: &tree_sitter::Node, captures: &mut Vec<TreeSitterCapture>) {
    captures.push(TreeSitterCapture {
        start: node.start_byte() as i32,
        end: node.end_byte() as i32,
        type_name: node.kind().to_string(),
    });

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            collect_all_nodes(&cursor.node(), captures);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_language() {
        let result = query_tree_sitter(
            "x".to_string(),
            "nonexistent".to_string(),
            "(identifier)".to_string(),
        );
        assert!(result.error.contains("Unsupported language"));
        assert!(result.captures.is_empty());
    }

    #[test]
    fn test_parse_rust() {
        let result = parse_with_tree_sitter("fn main() {}".to_string(), "rust".to_string());
        assert!(result.error.is_empty());
        assert!(!result.captures.is_empty());
        let kinds: Vec<&str> = result
            .captures
            .iter()
            .map(|c| c.type_name.as_str())
            .collect();
        assert!(kinds.contains(&"function_item"));
    }

    #[test]
    fn test_query_rust() {
        let source = "fn foo() {}\nfn bar() {}".to_string();
        let query = "(function_item name: (identifier) @funcname)".to_string();
        let result = query_tree_sitter(source, "rust".to_string(), query);
        assert!(result.error.is_empty());
        assert_eq!(result.captures.len(), 2);
        assert!(result.captures.iter().all(|c| c.type_name == "funcname"));
    }

    #[test]
    fn test_parse_typescript() {
        let result =
            parse_with_tree_sitter("const x: number = 5;".to_string(), "typescript".to_string());
        assert!(result.error.is_empty());
        assert!(!result.captures.is_empty());
        let kinds: Vec<&str> = result
            .captures
            .iter()
            .map(|c| c.type_name.as_str())
            .collect();
        assert!(kinds.contains(&"lexical_declaration"));
    }

    #[test]
    fn test_query_javascript() {
        let source = "function hello() { return 42; }".to_string();
        let query = "(function_declaration name: (identifier) @fnname)".to_string();
        let result = query_tree_sitter(source, "javascript".to_string(), query);
        assert!(result.error.is_empty());
        assert_eq!(result.captures.len(), 1);
        assert_eq!(result.captures[0].type_name, "fnname");
    }

    #[test]
    fn test_parse_python() {
        let result = parse_with_tree_sitter("def hello(): pass".to_string(), "python".to_string());
        assert!(result.error.is_empty());
        assert!(!result.captures.is_empty());
        let kinds: Vec<&str> = result
            .captures
            .iter()
            .map(|c| c.type_name.as_str())
            .collect();
        assert!(kinds.contains(&"function_definition"));
    }
}
