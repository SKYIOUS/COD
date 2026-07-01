use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[napi(object)]
pub struct TokenSpan {
    pub start: i32,
    pub end: i32,
    pub class_name: String,
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
pub struct DecorationSpan {
    pub start: i32,
    pub end: i32,
    pub class_name: String,
    pub is_inline: bool,
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
pub struct ViewportLineData {
    pub line_content: String,
    pub tokens_json: String,
    pub decorations_json: String,
    pub tab_size: i32,
    pub faux_indent_length: i32,
    pub is_overflowing: bool,
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\t' => out.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            _ => out.push(ch),
        }
    }
    out
}

#[napi]
pub fn render_line_html(line: String, tokens_json: String, decorations_json: String) -> String {
    let tokens: Vec<TokenSpan> = serde_json::from_str(&tokens_json).unwrap_or_default();
    let decorations: Vec<DecorationSpan> = serde_json::from_str(&decorations_json).unwrap_or_default();

    let line_len = line.len();
    if tokens.is_empty() && decorations.is_empty() {
        return format!("<span>{}</span>", escape_html(&line));
    }

    let mut deco_map: Vec<&DecorationSpan> = Vec::new();
    for d in &decorations {
        if d.start as usize <= line_len {
            deco_map.push(d);
        }
    }

    let mut html = String::with_capacity(line.len() * 2 + 64);
    let mut pos = 0usize;
    let mut ti = 0usize;
    let mut di = 0usize;

    while pos < line_len || ti < tokens.len() || di < deco_map.len() {
        let token = tokens.get(ti);
        let deco = deco_map.get(di).copied();

        let next_token_end = token.map(|t| t.end.max(t.start) as usize).unwrap_or(line_len);
        let next_deco_end = deco.map(|d| d.end.max(d.start) as usize).unwrap_or(line_len);

        let seg_end = next_token_end.min(next_deco_end).min(line_len);

        if seg_end <= pos { break; }

        let text = &line[pos..seg_end];
        let escaped = escape_html(text);

        let mut classes = Vec::new();
        if let Some(t) = token {
            if !t.class_name.is_empty() && (pos as i32) >= t.start && (pos as i32) < t.end {
                classes.push(&t.class_name);
            } else {
                ti += 1;
            }
        }
        if let Some(d) = deco {
            if !d.class_name.is_empty() && (pos as i32) >= d.start && (pos as i32) < d.end {
                if d.is_inline {
                    classes.push(&d.class_name);
                }
            } else {
                di += 1;
            }
        }

        if classes.is_empty() {
            html.push_str(&escaped);
        } else {
            html.push_str("<span class=\"");
            for (i, cls) in classes.iter().enumerate() {
                if i > 0 { html.push(' '); }
                html.push_str(cls);
            }
            html.push_str("\">");
            html.push_str(&escaped);
            html.push_str("</span>");
        }

        pos = seg_end;
    }

    if html.is_empty() {
        return format!("<span>{}</span>", escape_html(&line));
    }
    html
}

#[napi]
pub fn render_lines_html(lines: Vec<String>, all_tokens_json: String, all_decorations_json: String) -> Vec<String> {
    let all_tokens: Vec<Vec<TokenSpan>> = serde_json::from_str(&all_tokens_json).unwrap_or_default();
    let all_decos: Vec<Vec<DecorationSpan>> = serde_json::from_str(&all_decorations_json).unwrap_or_default();

    let n = lines.len();
    let mut results = Vec::with_capacity(n);

    for i in 0..n {
        let tokens_json = if i < all_tokens.len() {
            serde_json::to_string(&all_tokens[i]).unwrap_or_else(|_| "[]".to_string())
        } else {
            "[]".to_string()
        };
        let decos_json = if i < all_decos.len() {
            serde_json::to_string(&all_decos[i]).unwrap_or_else(|_| "[]".to_string())
        } else {
            "[]".to_string()
        };
        results.push(render_line_html(lines[i].clone(), tokens_json, decos_json));
    }

    results
}

#[napi]
pub fn render_minimap_line(line: String, tokens_json: String, ch_width: i32) -> String {
    let tokens: Vec<TokenSpan> = serde_json::from_str(&tokens_json).unwrap_or_default();
    let cw = if ch_width <= 0 { 1 } else { ch_width as usize };
    let max_chars = 120 / cw;

    let mut out = String::with_capacity(max_chars + 16);
    let mut pos = 0usize;
    let mut ti = 0usize;

    for _ in 0..max_chars {
        if pos >= line.len() { break; }

        let ch = line.as_bytes()[pos];
        let class = tokens.get(ti).filter(|t| (pos as i32) >= t.start && (pos as i32) < t.end).map(|t| t.class_name.as_str()).unwrap_or("");

        if ch == b' ' || ch == b'\t' {
            out.push('·');
        } else {
            out.push(if class.is_empty() { '■' } else { '●' });
        }

        if let Some(t) = tokens.get(ti) {
            if (pos as i32) >= t.end - 1 { ti += 1; }
        }
        pos += 1;
    }

    out
}
