use serde_json::Value;

#[napi(object)]
pub struct JsoncParseResult {
    pub ok: bool,
    pub value: Option<String>,
    pub error: Option<String>,
}

#[napi]
pub fn parse_jsonc(content: String) -> JsoncParseResult {
    let stripped = strip_comments(&content);
    match serde_json::from_str::<Value>(&stripped) {
        Ok(val) => {
            match serde_json::to_string(&val) {
                Ok(json_str) => JsoncParseResult {
                    ok: true,
                    value: Some(json_str),
                    error: None,
                },
                Err(e) => JsoncParseResult {
                    ok: false,
                    value: None,
                    error: Some(e.to_string()),
                },
            }
        }
        Err(_) => {
            // Try stripping trailing commas
            let trailing_commas_stripped = strip_trailing_commas(&stripped);
            match serde_json::from_str::<Value>(&trailing_commas_stripped) {
                Ok(val) => {
                    match serde_json::to_string(&val) {
                        Ok(json_str) => JsoncParseResult {
                            ok: true,
                            value: Some(json_str),
                            error: None,
                        },
                        Err(e) => JsoncParseResult {
                            ok: false,
                            value: None,
                            error: Some(e.to_string()),
                        },
                    }
                }
                Err(e) => JsoncParseResult {
                    ok: false,
                    value: None,
                    error: Some(e.to_string()),
                },
            }
        }
    }
}

fn strip_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        // String literals - pass through as-is
        if c == '"' {
            result.push(c);
            i += 1;
            while i < len {
                let sc = chars[i];
                result.push(sc);
                if sc == '\\' {
                    i += 1;
                    if i < len {
                        result.push(chars[i]);
                        i += 1;
                    }
                } else if sc == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Single line comment //
        if c == '/' && i + 1 < len && chars[i + 1] == '/' {
            i += 2;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment /* */
        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2; // skip */
            }
            continue;
        }
        result.push(c);
        i += 1;
    }
    result
}

fn strip_trailing_commas(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if c == ',' {
            // Look ahead for whitespace followed by } or ]
            let mut j = i + 1;
            while j < len && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
                j += 1;
            }
            if j < len && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }
        // Also handle strings for safety
        if c == '"' {
            result.push(c);
            i += 1;
            while i < len {
                let sc = chars[i];
                result.push(sc);
                if sc == '\\' {
                    i += 1;
                    if i < len {
                        result.push(chars[i]);
                        i += 1;
                    }
                } else if sc == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        result.push(c);
        i += 1;
    }
    result
}
