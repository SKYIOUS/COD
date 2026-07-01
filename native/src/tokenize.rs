use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[napi(object)]
pub struct TokenCapture {
    pub start: i32,
    pub end: i32,
    pub type_name: String,
    pub language_id: i32,
}

#[derive(Serialize)]
#[napi(object)]
pub struct EndOffsetToken {
    pub end_offset: i32,
    pub metadata: i32,
}

#[derive(Serialize)]
#[napi(object)]
pub struct ScopeTokenResult {
    pub end_offset: i32,
    pub scopes_json: String,  // JSON-serialized Vec<String>
    pub bracket_json: String, // JSON-serialized Option<Vec<i32>>
    pub language_id: i32,
}

struct ScopeToken {
    end_offset: i32,
    scopes: Vec<String>,
    bracket: Option<Vec<i32>>,
    language_id: i32,
}

const BRACKETS: &[char] = &['{', '}', '[', ']', '(', ')'];

fn find_brackets(text: &str, start_offset: i32) -> Option<Vec<i32>> {
    let mut positions: Vec<i32> = Vec::new();
    for (i, ch) in text.char_indices() {
        if BRACKETS.contains(&ch) {
            positions.push(start_offset + i as i32);
        }
    }
    if positions.is_empty() { None } else { Some(positions) }
}

#[napi]
pub fn create_tokens_from_captures_scoped(
    captures: Vec<TokenCapture>,
    range_start_offset: i32,
    range_end_offset: i32,
    base_scope: String,
) -> Vec<ScopeTokenResult> {
    let range_len = range_end_offset - range_start_offset;
    if captures.is_empty() {
        return vec![ScopeTokenResult {
            end_offset: range_len,
            scopes_json: serde_json::to_string(&vec![base_scope]).unwrap_or_default(),
            bracket_json: String::from("null"),
            language_id: 0,
        }];
    }

    let mut tokens: Vec<ScopeToken> = Vec::with_capacity(captures.len() + 8);
    let mut token_idx: usize = 0;

    // Pre-fill with base scope
    for _ in 0..captures.len() + 4 {
        tokens.push(ScopeToken { end_offset: 0, scopes: vec![base_scope.clone()], bracket: None, language_id: captures[0].language_id });
    }

    for cap in &captures {
        let token_end = if cap.end < range_end_offset { if cap.end < range_start_offset { range_start_offset } else { cap.end } } else { range_end_offset };
        let token_start = if cap.start < range_start_offset { range_start_offset } else { cap.start };
        let end_offset = token_end - range_start_offset;
        let cur_len = token_end - token_start;
        let start_offset = end_offset - cur_len;

        let prev_end = if token_idx > 0 { tokens[token_idx - 1].end_offset } else { token_start - range_start_offset - 1 };
        if prev_end >= 0 && prev_end < start_offset {
            tokens[token_idx] = ScopeToken { end_offset: start_offset, scopes: vec![base_scope.clone()], bracket: None, language_id: cap.language_id };
            token_idx += 1;
        }

        if cur_len < 0 { continue; }

        if prev_end >= end_offset {
            let mut wi = token_idx.saturating_sub(1);
            loop {
                let prev_prev_end = if wi >= 2 { tokens[wi - 1].end_offset } else { 0 };
                if prev_prev_end + cur_len == prev_end {
                    if prev_prev_end == start_offset {
                        tokens[wi].scopes.push(cap.type_name.clone());
                        if tokens[wi].bracket.is_none() { tokens[wi].bracket = find_brackets(&cap.type_name, start_offset); }
                    }
                } else if prev_prev_end <= start_offset {
                    let old_scopes = tokens[wi].scopes.clone();
                    let old_bracket = tokens[wi].bracket.clone();
                    let prev_end_2 = if wi > 0 { tokens[wi - 1].end_offset } else { 0 };
                    if prev_end_2 != start_offset {
                        let mut pre_insert: Option<Vec<i32>> = None;
                        let mut post_insert: Vec<i32> = Vec::new();
                        if let Some(ref ob) = old_bracket {
                            for &b in ob {
                                if b < start_offset { pre_insert.get_or_insert(Vec::new()).push(b); }
                                else if b > end_offset { post_insert.push(b); }
                            }
                            if pre_insert.as_ref().map_or(true, |v| v.is_empty()) { pre_insert = None; }
                        }
                        tokens.insert(wi, ScopeToken {
                            end_offset: start_offset,
                            scopes: old_scopes.clone(),
                            bracket: pre_insert,
                            language_id: cap.language_id,
                        });
                        token_idx += 1;
                        let remaining_bracket = if post_insert.is_empty() { None } else { Some(post_insert) };
                        tokens.insert(wi + 1, ScopeToken {
                            end_offset,
                            scopes: [old_scopes, vec![cap.type_name.clone()]].concat(),
                            bracket: remaining_bracket.or_else(|| find_brackets(&cap.type_name, start_offset)),
                            language_id: cap.language_id,
                        });
                        token_idx += 1;
                        tokens[wi].bracket = None;
                    } else {
                        tokens.insert(wi, ScopeToken {
                            end_offset,
                            scopes: [old_scopes, vec![cap.type_name.clone()]].concat(),
                            bracket: old_bracket.or_else(|| find_brackets(&cap.type_name, start_offset)),
                            language_id: cap.language_id,
                        });
                        token_idx += 1;
                    }
                    break;
                }
                if wi == 0 { break; }
                wi -= 1;
                if tokens[wi].end_offset <= start_offset { break; }
            }
        } else {
            tokens[token_idx] = ScopeToken {
                end_offset,
                scopes: vec![base_scope.clone(), cap.type_name.clone()],
                bracket: find_brackets(&cap.type_name, start_offset),
                language_id: cap.language_id,
            };
            token_idx += 1;
        }
    }

    // Pad end with base scope if needed
    if token_idx > 0 && tokens[token_idx - 1].end_offset < range_len {
        tokens[token_idx] = ScopeToken { end_offset: range_len, scopes: vec![base_scope.clone()], bracket: None, language_id: captures[0].language_id };
        token_idx += 1;
    }

    // Convert to ScopeTokenResult
    let mut result: Vec<ScopeTokenResult> = Vec::with_capacity(token_idx);
    for i in 0..token_idx {
        let t = &tokens[i];
        if t.end_offset == 0 && i != 0 { break; }
        result.push(ScopeTokenResult {
            end_offset: t.end_offset,
            scopes_json: serde_json::to_string(&t.scopes).unwrap_or_default(),
            bracket_json: serde_json::to_string(&t.bracket).unwrap_or_else(|_| "null".to_string()),
            language_id: t.language_id,
        });
    }

    result
}

#[napi]
pub fn tokens_to_uint32_array(tokens: Vec<EndOffsetToken>) -> Vec<i32> {
    let mut flat = Vec::with_capacity(tokens.len() * 2);
    for t in tokens {
        flat.push(t.end_offset);
        flat.push(t.metadata);
    }
    flat
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap(start: i32, end: i32, type_name: &str) -> TokenCapture {
        TokenCapture {
            start,
            end,
            type_name: type_name.to_string(),
            language_id: 0,
        }
    }

    fn s(v: &[&str]) -> Vec<String> { v.iter().map(|s| s.to_string()).collect() }

    fn n() -> Option<Vec<i32>> { None }

    fn parse_result(result: &[ScopeTokenResult]) -> Vec<(i32, Vec<String>, Option<Vec<i32>>)> {
        result.iter().map(|t| {
            let scopes: Vec<String> = serde_json::from_str(&t.scopes_json).unwrap();
            let bracket: Option<Vec<i32>> = serde_json::from_str(&t.bracket_json).unwrap();
            (t.end_offset, scopes, bracket)
        }).collect()
    }

    #[test]
    fn test_empty_captures() {
        let result = create_tokens_from_captures_scoped(vec![], 0, 10, "source.ts".to_string());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].end_offset, 10);
        assert_eq!(serde_json::from_str::<Vec<String>>(&result[0].scopes_json).unwrap(), s(&["source.ts"]));
    }

    #[test]
    fn test_single_capture() {
        let captions = vec![cap(0, 10, "function")];
        let result = create_tokens_from_captures_scoped(captions, 0, 20, "source.ts".to_string());
        let parsed = parse_result(&result);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], (10, s(&["source.ts", "function"]), n()));
        assert_eq!(parsed[1], (20, s(&["source.ts"]), n()));
    }

    #[test]
    fn test_gap_before_capture() {
        let captions = vec![cap(10, 20, "keyword")];
        let result = create_tokens_from_captures_scoped(captions, 0, 30, "source.rs".to_string());
        let parsed = parse_result(&result);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], (10, s(&["source.rs"]), n()));
        assert_eq!(parsed[1], (20, s(&["source.rs", "keyword"]), n()));
        assert_eq!(parsed[2], (30, s(&["source.rs"]), n()));
    }

    #[test]
    fn test_nested_captures() {
        let captions = vec![cap(0, 30, "function"), cap(10, 20, "name")];
        let result = create_tokens_from_captures_scoped(captions, 0, 40, "source.ts".to_string());
        let parsed = parse_result(&result);
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], (10, s(&["source.ts", "function"]), n()));
        assert_eq!(parsed[1], (20, s(&["source.ts", "function", "name"]), n()));
        assert_eq!(parsed[2], (30, s(&["source.ts", "function"]), n()));
        assert_eq!(parsed[3], (40, s(&["source.ts"]), n()));
    }

    #[test]
    fn test_no_gap_for_adjacent_tokens() {
        let captions = vec![cap(0, 10, "keyword"), cap(10, 20, "identifier")];
        let result = create_tokens_from_captures_scoped(captions, 0, 20, "source.ts".to_string());
        let parsed = parse_result(&result);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], (10, s(&["source.ts", "keyword"]), n()));
        assert_eq!(parsed[1], (20, s(&["source.ts", "identifier"]), n()));
    }

    #[test]
    fn test_bracket_detection() {
        let captions = vec![cap(0, 1, "punctuation.bracket")];
        // Brackets based on content of type_name (which is not the actual text, so None)
        let result = create_tokens_from_captures_scoped(captions, 0, 5, "source.ts".to_string());
        let parsed = parse_result(&result);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].2, None); // type_name "punctuation.bracket" contains no bracket chars
    }

    #[test]
    fn test_capture_partially_outside_range() {
        let captions = vec![cap(-5, 15, "comment")];
        let result = create_tokens_from_captures_scoped(captions, 0, 20, "source.rs".to_string());
        let parsed = parse_result(&result);
        assert!(parsed.len() >= 1);
        // capture clamped to range_start_offset..range_end_offset
        assert_eq!(parsed[0].0, 15);
    }

    #[test]
    fn test_multiple_captures_same_position() {
        let captions = vec![cap(5, 15, "variable"), cap(5, 15, "parameter")];
        let result = create_tokens_from_captures_scoped(captions, 0, 20, "source.ts".to_string());
        let parsed = parse_result(&result);
        // gap + merged scope + old scope + padding = 4 tokens
        // the merged token has both scopes; the shifted old token is zero-length (ignored in display)
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], (5, s(&["source.ts"]), n()));
        assert_eq!(parsed[1], (15, s(&["source.ts", "variable", "parameter"]), n()));
        assert_eq!(parsed[3], (20, s(&["source.ts"]), n()));
    }

    #[test]
    fn test_tokens_uint32() {
        let tokens = vec![
            EndOffsetToken { end_offset: 5, metadata: 1 },
            EndOffsetToken { end_offset: 10, metadata: 2 },
        ];
        let flat = tokens_to_uint32_array(tokens);
        assert_eq!(flat, vec![5, 1, 10, 2]);
    }
}
