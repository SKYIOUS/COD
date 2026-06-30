use serde::Serialize;

#[derive(Serialize)]
#[napi(object)]
pub struct TokenCapture {
    pub start: i32,
    pub end: i32,
    pub type_name: String,
    pub language_id: i32,
}

#[derive(Serialize)]
#[napi(object)]
pub struct EncodedToken {
    pub start_index: i32,
    pub metadata: i32,
}

#[derive(Serialize)]
#[napi(object)]
pub struct TokenizeResult {
    pub tokens: Vec<EncodedToken>,
    pub language_id: i32,
}

#[napi]
pub fn encode_tree_sitter_captures(captures: Vec<TokenCapture>, theme_json: String) -> Vec<EncodedToken> {
    let theme_map: std::collections::HashMap<String, i32> = serde_json::from_str(&theme_json).unwrap_or_default();
    let mut encoded: Vec<EncodedToken> = Vec::with_capacity(captures.len());

    for cap in &captures {
        let metadata = theme_map.get(&cap.type_name).copied().unwrap_or(0);
        // ponytail: naive dedup — if same start as previous, keep the longer/higher-priority one
        if let Some(last) = encoded.last_mut() {
            if last.start_index == cap.start {
                last.metadata = metadata;
                continue;
            }
        }
        encoded.push(EncodedToken { start_index: cap.start, metadata });
    }

    encoded
}

#[napi]
pub fn tokens_to_uint32_array(tokens: Vec<EncodedToken>) -> Vec<i32> {
    let mut flat = Vec::with_capacity(tokens.len() * 2);
    for t in tokens {
        flat.push(t.start_index);
        flat.push(t.metadata);
    }
    flat
}
