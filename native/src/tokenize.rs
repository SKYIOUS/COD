use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn token_type_to_metadata(type_name: &str, theme_map: &HashMap<String, i32>) -> i32 {
    theme_map.get(type_name).copied().unwrap_or(0)
}

#[napi]
pub fn create_tokens_from_captures(
    captures: Vec<TokenCapture>,
    range_start_offset: i32,
    range_end_offset: i32,
    base_scope: String,
    theme_json: String,
) -> Vec<EndOffsetToken> {
    let theme_map: HashMap<String, i32> = serde_json::from_str(&theme_json).unwrap_or_default();
    let range_len = range_end_offset - range_start_offset;
    if captures.is_empty() {
        return vec![EndOffsetToken { end_offset: range_len, metadata: 0 }];
    }

    let mut tokens: Vec<ScopeToken> = Vec::with_capacity(captures.len() + 8);
    let mut token_idx: usize = 0;

    let push_token = |tokens: &mut Vec<ScopeToken>, end_offset: i32, scopes: Vec<String>, bracket: Option<Vec<i32>>, lid: i32| {
        tokens.push(ScopeToken { end_offset, scopes, bracket, language_id: lid });
    };

    let add_current = |tokens: &mut Vec<ScopeToken>, token_idx: &mut usize, capture: &TokenCapture, start_offset: i32, end_offset: i32, position: Option<usize>| {
        if let Some(pos) = position {
            let old_scopes = tokens[pos].scopes.clone();
            let old_bracket = tokens[pos].bracket.clone();
            let prev_end = if pos > 0 { tokens[pos - 1].end_offset } else { 0 };
            if prev_end != start_offset {
                let mut pre_insert: Option<Vec<i32>> = None;
                let mut post_insert: Vec<i32> = Vec::new();
                if let Some(ref ob) = old_bracket {
                    for &b in ob {
                        if b < start_offset { pre_insert.get_or_insert(Vec::new()).push(b); }
                        else if b > end_offset { post_insert.push(b); }
                    }
                    if pre_insert.as_ref().map_or(true, |v| v.is_empty()) { pre_insert = None; }
                }
                let new_bracket = if pre_insert.is_some() { pre_insert.clone() } else { None };
                tokens.insert(pos, ScopeToken {
                    end_offset: start_offset,
                    scopes: old_scopes.clone(),
                    bracket: new_bracket,
                    language_id: capture.language_id,
                });
                *token_idx += 1;
                let remaining_bracket = if post_insert.is_empty() { None } else { Some(post_insert) };
                tokens.insert(pos + 1, ScopeToken {
                    end_offset,
                    scopes: [old_scopes.clone(), vec![capture.type_name.clone()]].concat(),
                    bracket: remaining_bracket.or_else(|| find_brackets(&capture.type_name, start_offset)),
                    language_id: capture.language_id,
                });
                tokens[pos].bracket = pre_insert;
            } else {
                tokens.insert(pos + 1, ScopeToken {
                    end_offset,
                    scopes: [old_scopes.clone(), vec![capture.type_name.clone()]].concat(),
                    bracket: old_bracket.clone().or_else(|| find_brackets(&capture.type_name, start_offset)),
                    language_id: capture.language_id,
                });
            }
        } else {
            tokens.push(ScopeToken {
                end_offset,
                scopes: vec![base_scope.clone(), capture.type_name.clone()],
                bracket: find_brackets(&capture.type_name, start_offset),
                language_id: capture.language_id,
            });
        }
        *token_idx += 1;
    };

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
                let prev_end = if wi < tokens.len() { tokens[wi].end_offset } else { 0 };
                if prev_prev_end + cur_len == prev_end {
                    if prev_prev_end == start_offset {
                        tokens[wi].scopes.push(cap.type_name.clone());
                        if tokens[wi].bracket.is_none() { tokens[wi].bracket = find_brackets(&cap.type_name, start_offset); }
                    }
                } else if prev_prev_end <= start_offset {
                    add_current(&mut tokens, &mut token_idx, cap, start_offset, end_offset, Some(wi));
                    break;
                }
                if wi == 0 { break; }
                wi -= 1;
                let prev_start = if wi >= 1 { tokens[wi - 1].end_offset } else { 0 };
                let prev_end = tokens[wi].end_offset;
                if prev_end <= start_offset { break; }
            }
        } else {
            add_current(&mut tokens, &mut token_idx, cap, start_offset, end_offset, None);
        }
    }

    // Pad end with base scope if needed
    if token_idx > 0 && tokens[token_idx - 1].end_offset < range_len {
        tokens[token_idx] = ScopeToken { end_offset: range_len, scopes: vec![base_scope.clone()], bracket: None, language_id: captures[0].language_id };
        token_idx += 1;
    }

    // Remove trailing zero-length tokens, resolve metadata
    let mut result: Vec<EndOffsetToken> = Vec::with_capacity(token_idx);
    for i in 0..token_idx {
        let t = &tokens[i];
        if t.end_offset == 0 && i != 0 { break; }
        let scope_name = t.scopes.last().map(|s| s.as_str()).unwrap_or(&base_scope);
        let metadata = token_type_to_metadata(scope_name, &theme_map);
        result.push(EndOffsetToken { end_offset: t.end_offset, metadata });
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
