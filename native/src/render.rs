use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct TokenSpan {
    pub start: i32,
    pub end: i32,
    pub class_name: String,
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct DecorationSpan {
    pub start: i32,
    pub end: i32,
    pub class_name: String,
    pub is_inline: bool,
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
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
    let decorations: Vec<DecorationSpan> =
        serde_json::from_str(&decorations_json).unwrap_or_default();

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

        let next_token_start = token.map(|t| t.start as usize).unwrap_or(line_len);
        let next_deco_start = deco.map(|d| d.start as usize).unwrap_or(line_len);
        let _seg_start = pos;
        // Skip ahead to next token/deco start
        let first_active = next_token_start.min(next_deco_start).min(line_len);
        if first_active > pos {
            let text = &line[pos..first_active];
            html.push_str(&escape_html(text));
            pos = first_active;
            if pos >= line_len {
                break;
            }
            continue;
        }

        let next_token_end = token
            .map(|t| t.end.max(t.start) as usize)
            .unwrap_or(line_len);
        let next_deco_end = deco
            .map(|d| d.end.max(d.start) as usize)
            .unwrap_or(line_len);
        let seg_end = next_token_end.min(next_deco_end).min(line_len);

        if seg_end <= pos {
            break;
        }

        let text = &line[pos..seg_end];
        let escaped = escape_html(text);

        let mut classes = Vec::new();
        if let Some(t) = token {
            if !t.class_name.is_empty() && (pos as i32) >= t.start && (pos as i32) < t.end {
                classes.push(&t.class_name);
            }
        }
        if let Some(d) = deco {
            if !d.class_name.is_empty() && (pos as i32) >= d.start && (pos as i32) < d.end {
                if d.is_inline {
                    classes.push(&d.class_name);
                }
            }
        }

        if classes.is_empty() {
            html.push_str(&escaped);
        } else {
            html.push_str("<span class=\"");
            for (i, cls) in classes.iter().enumerate() {
                if i > 0 {
                    html.push(' ');
                }
                html.push_str(cls);
            }
            html.push_str("\">");
            html.push_str(&escaped);
            html.push_str("</span>");
        }

        pos = seg_end;

        // Advance index if we've passed the token/deco end
        if let Some(t) = token {
            if (pos as i32) >= t.end {
                ti += 1;
            }
        }
        if let Some(d) = deco {
            if (pos as i32) >= d.end {
                di += 1;
            }
        }
    }

    if html.is_empty() {
        return format!("<span>{}</span>", escape_html(&line));
    }
    html
}

#[napi]
pub fn render_lines_html(
    lines: Vec<String>,
    all_tokens_json: String,
    all_decorations_json: String,
) -> Vec<String> {
    let all_tokens: Vec<Vec<TokenSpan>> =
        serde_json::from_str(&all_tokens_json).unwrap_or_default();
    let all_decos: Vec<Vec<DecorationSpan>> =
        serde_json::from_str(&all_decorations_json).unwrap_or_default();

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
        if pos >= line.len() {
            break;
        }

        let ch = line.as_bytes()[pos];
        let class = tokens
            .get(ti)
            .filter(|t| (pos as i32) >= t.start && (pos as i32) < t.end)
            .map(|t| t.class_name.as_str())
            .unwrap_or("");

        if ch == b' ' || ch == b'\t' {
            out.push('·');
        } else {
            out.push(if class.is_empty() { '■' } else { '●' });
        }

        if let Some(t) = tokens.get(ti) {
            if (pos as i32) >= t.end - 1 {
                ti += 1;
            }
        }
        pos += 1;
    }

    out
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct CharacterMappingEntry {
    pub column: i32,
    pub part_index: i32,
    pub offset_in_part: i32,
    pub horizontal_offset: i32,
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RenderLineNativeResult {
    pub html: String,
    pub character_mapping_json: String,
    pub contains_foreign_elements: i32,
}

fn is_fullwidth(ch: char) -> bool {
    if ch as u32 >= 0x1100 {
        ch as u32 <= 0x115F
            || ch as u32 == 0x2329
            || ch as u32 == 0x232A
            || (ch as u32 >= 0x2E80 && ch as u32 <= 0x3247)
            || (ch as u32 >= 0x3250 && ch as u32 <= 0x4DBF)
            || (ch as u32 >= 0x4E00 && ch as u32 <= 0xA4CF)
            || (ch as u32 >= 0xA960 && ch as u32 <= 0xA97C)
            || (ch as u32 >= 0xAC00 && ch as u32 <= 0xD7A3)
            || (ch as u32 >= 0xF900 && ch as u32 <= 0xFAFF)
            || (ch as u32 >= 0xFE10 && ch as u32 <= 0xFE19)
            || (ch as u32 >= 0xFE30 && ch as u32 <= 0xFE6F)
            || (ch as u32 >= 0xFF01 && ch as u32 <= 0xFF60)
            || (ch as u32 >= 0xFFE0 && ch as u32 <= 0xFFE6)
            || (ch as u32 >= 0x1B000 && ch as u32 <= 0x1B0FF)
            || (ch as u32 >= 0x1B100 && ch as u32 <= 0x1B12F)
            || (ch as u32 >= 0x1F004)
    } else {
        false
    }
}

// ponytail: renders a single line part's characters into HTML + character mapping
// replaces the inner character loop in viewLineRenderer.ts _renderLine
fn render_characters(
    line_content: &str,
    start_index: usize,
    end_index: usize,
    tab_size: usize,
    visible_column: &mut usize,
    faux_indent_length: usize,
    render_space_char_code: u32,
    render_control_characters: bool,
    can_use_halfwidth_rightwards_arrow: bool,
    part_renders_whitespace: bool,
    char_offset_in_part: &mut usize,
    char_horizontal_offset: &mut usize,
    character_mapping: &mut Vec<CharacterMappingEntry>,
    part_index: i32,
    part_displacement: &mut usize,
    sb: &mut String,
) {
    let chars: Vec<char> = line_content[start_index..end_index].chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        let char_index = start_index + i;
        character_mapping.push(CharacterMappingEntry {
            column: (char_index + 1) as i32,
            part_index,
            offset_in_part: *char_offset_in_part as i32,
            horizontal_offset: *char_horizontal_offset as i32,
        });
        *part_displacement = 0;

        let mut produced_characters = 1usize;
        let mut char_width = 1usize;

        if ch == '\t' {
            if part_renders_whitespace {
                produced_characters = tab_size - (*visible_column % tab_size);
                if produced_characters == 0 {
                    produced_characters = tab_size;
                }
                char_width = produced_characters;
                if !can_use_halfwidth_rightwards_arrow || char_width > 1 {
                    sb.push('\u{2192}'); // RIGHTWARDS ARROW
                } else {
                    sb.push('\u{FFEB}'); // HALFWIDTH RIGHTWARDS ARROW
                }
                for _ in 1..char_width {
                    sb.push('\u{A0}'); // &nbsp;
                }
            } else {
                produced_characters = tab_size - (*visible_column % tab_size);
                char_width = produced_characters;
                for _ in 0..produced_characters {
                    sb.push('\u{A0}'); // &nbsp;
                }
            }
        } else if ch == ' ' {
            if part_renders_whitespace {
                produced_characters = 2;
                char_width = 1;
                sb.push(char::from_u32(render_space_char_code).unwrap_or('\u{00B7}')); // &middot;
                sb.push('\u{200C}'); // ZERO WIDTH NON-JOINER
            } else {
                sb.push('\u{A0}'); // &nbsp;
            }
        } else if ch == '<' {
            sb.push_str("&lt;");
        } else if ch == '>' {
            sb.push_str("&gt;");
        } else if ch == '&' {
            sb.push_str("&amp;");
        } else if ch == '\0' {
            if render_control_characters {
                sb.push('\u{2400}'); // Control pictures: NUL
            } else {
                sb.push_str("&#00;");
            }
        } else if ch == '\u{FEFF}' || ch == '\u{2028}' || ch == '\u{2029}' || ch == '\u{0085}' {
            sb.push('\u{FFFD}');
        } else {
            if !part_renders_whitespace && is_fullwidth(ch) {
                char_width = 2;
            }
            if render_control_characters && (ch as u32) < 32 {
                sb.push(char::from_u32(0x2400 + ch as u32).unwrap());
            } else if render_control_characters && ch as u32 == 127 {
                sb.push('\u{2421}'); // DEL control picture
            } else if render_control_characters && (ch as u32) < 160 && ch != ' ' {
                sb.push_str("[U+");
                sb.push_str(&format!("{:04X}", ch as u32));
                sb.push(']');
                produced_characters = 8;
                char_width = 8;
            } else {
                sb.push(ch);
            }
        }

        *char_offset_in_part += produced_characters;
        *char_horizontal_offset += char_width;
        if char_index >= faux_indent_length {
            *visible_column += char_width;
        }
    }
}

#[napi]
pub fn render_line_native(
    line_content: String,
    parts_json: String,
    tab_size: i32,
    faux_indent_length: i32,
    start_visible_column: i32,
    space_width: i32,
    render_space_char_code: i32,
    render_whitespace: i32,
    render_control_characters: bool,
    can_use_halfwidth_rightwards_arrow: bool,
    font_is_monospace: bool,
    contains_foreign_elements: i32,
    is_overflowing: bool,
    overflowing_char_count: i32,
    len_val: i32,
) -> RenderLineNativeResult {
    native_render_line(
        line_content,
        parts_json,
        tab_size,
        faux_indent_length,
        start_visible_column,
        space_width,
        render_space_char_code,
        render_whitespace,
        render_control_characters,
        can_use_halfwidth_rightwards_arrow,
        font_is_monospace,
        contains_foreign_elements,
        is_overflowing,
        overflowing_char_count,
        len_val,
    )
}

#[derive(Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct BatchLineInput {
    pub line_content: String,
    pub parts_json: String,
    pub start_visible_column: i32,
    pub is_overflowing: bool,
    pub overflowing_char_count: i32,
    pub len_val: i32,
}

#[napi]
pub fn render_lines_native(
    inputs: Vec<BatchLineInput>,
    tab_size: i32,
    faux_indent_length: i32,
    space_width: i32,
    render_space_char_code: i32,
    render_whitespace: i32,
    render_control_characters: bool,
    can_use_halfwidth_rightwards_arrow: bool,
    font_is_monospace: bool,
    contains_foreign_elements: i32,
) -> Vec<RenderLineNativeResult> {
    inputs
        .into_iter()
        .map(|i| {
            native_render_line(
                i.line_content,
                i.parts_json,
                tab_size,
                faux_indent_length,
                i.start_visible_column,
                space_width,
                render_space_char_code,
                render_whitespace,
                render_control_characters,
                can_use_halfwidth_rightwards_arrow,
                font_is_monospace,
                contains_foreign_elements,
                i.is_overflowing,
                i.overflowing_char_count,
                i.len_val,
            )
        })
        .collect()
}

fn native_render_line(
    line_content: String,
    parts_json: String,
    tab_size: i32,
    faux_indent_length: i32,
    start_visible_column: i32,
    space_width: i32,
    render_space_char_code: i32,
    render_whitespace: i32,
    render_control_characters: bool,
    can_use_halfwidth_rightwards_arrow: bool,
    font_is_monospace: bool,
    contains_foreign_elements: i32,
    is_overflowing: bool,
    overflowing_char_count: i32,
    len_val: i32,
) -> RenderLineNativeResult {
    #[derive(Deserialize)]
    struct PartInput {
        end_index: i32,
        #[serde(rename = "type")]
        part_type: String,
        metadata: i32,
        #[serde(rename = "containsRTL")]
        contains_rtl: bool,
    }

    let parts: Vec<PartInput> = serde_json::from_str(&parts_json).unwrap_or_default();
    let mut html = String::with_capacity(line_content.len() * 2 + 64);
    let mut character_mapping: Vec<CharacterMappingEntry> = Vec::new();
    let len = len_val as usize;
    let line_chars: Vec<char> = line_content.chars().collect();

    let mut char_index = 0usize;
    let mut visible_column = start_visible_column as usize;
    let mut char_offset_in_part = 0usize;
    let mut char_horizontal_offset = 0usize;
    let mut part_displacement = 0usize;
    let mut last_character_mapping_defined = false;

    html.push_str("<span>");

    for (part_index, part) in parts.iter().enumerate() {
        let part_end_index = part.end_index as usize;
        let part_type = &part.part_type;
        let part_contains_rtl = part.contains_rtl;
        let part_renders_whitespace = render_whitespace != 0 && part_type == "mtkw";
        let part_renders_whitespace_with_width = part_renders_whitespace
            && !font_is_monospace
            && (part_type == "mtkw" || contains_foreign_elements == 0);
        let part_is_empty_and_has_pseudo_after = char_index == part_end_index && false; // simplified

        char_offset_in_part = 0;

        html.push_str("<span ");
        if part_contains_rtl {
            html.push_str("style=\"unicode-bidi:isolate\" ");
        }
        html.push_str("class=\"");
        if part_renders_whitespace_with_width {
            html.push_str("mtkz");
        } else {
            html.push_str(part_type);
        }
        html.push('"');

        if part_renders_whitespace {
            let mut part_width = 0usize;
            {
                let mut _ci = char_index;
                let mut _vc = visible_column;
                for _ in char_index..part_end_index {
                    if _ci < line_chars.len() {
                        let c = line_chars[_ci];
                        let cw = if c == '\t' {
                            tab_size as usize - (_vc % tab_size as usize)
                        } else {
                            1
                        };
                        part_width += cw;
                        if _ci >= faux_indent_length as usize {
                            _vc += cw;
                        }
                    }
                    _ci += 1;
                }
            }

            if part_renders_whitespace_with_width {
                html.push_str(&format!(
                    " style=\"width:{}px\"",
                    space_width * part_width as i32
                ));
            }
            html.push('>');

            render_characters(
                &line_content,
                char_index,
                part_end_index,
                tab_size as usize,
                &mut visible_column,
                faux_indent_length as usize,
                render_space_char_code as u32,
                render_control_characters,
                can_use_halfwidth_rightwards_arrow,
                true,
                &mut char_offset_in_part,
                &mut char_horizontal_offset,
                &mut character_mapping,
                part_index as i32,
                &mut part_displacement,
                &mut html,
            );
        } else {
            html.push('>');

            render_characters(
                &line_content,
                char_index,
                part_end_index,
                tab_size as usize,
                &mut visible_column,
                faux_indent_length as usize,
                render_space_char_code as u32,
                render_control_characters,
                can_use_halfwidth_rightwards_arrow,
                false,
                &mut char_offset_in_part,
                &mut char_horizontal_offset,
                &mut character_mapping,
                part_index as i32,
                &mut part_displacement,
                &mut html,
            );
        }

        if part_is_empty_and_has_pseudo_after {
            part_displacement += 1;
        } else {
            part_displacement = 0;
        }

        if char_index >= len && !last_character_mapping_defined && false
        /* part.isPseudoAfter() */
        {
            last_character_mapping_defined = true;
            character_mapping.push(CharacterMappingEntry {
                column: (char_index + 1) as i32,
                part_index: part_index as i32,
                offset_in_part: char_offset_in_part as i32,
                horizontal_offset: char_horizontal_offset as i32,
            });
        }

        html.push_str("</span>");
        char_index = part_end_index;
    }

    if !last_character_mapping_defined {
        let last_part_index = if parts.is_empty() { 0 } else { parts.len() - 1 };
        character_mapping.push(CharacterMappingEntry {
            column: (len + 1) as i32,
            part_index: last_part_index as i32,
            offset_in_part: char_offset_in_part as i32,
            horizontal_offset: char_horizontal_offset as i32,
        });
    }

    if is_overflowing {
        html.push_str(&format!(
            "<span class=\"mtkoverflow\">Show more ({})</span>",
            render_overflowing_char_count(overflowing_char_count as usize)
        ));
    }

    html.push_str("</span>");

    let cm_json = serde_json::to_string(&character_mapping).unwrap_or_else(|_| "[]".to_string());

    RenderLineNativeResult {
        html,
        character_mapping_json: cm_json,
        contains_foreign_elements,
    }
}

fn render_overflowing_char_count(n: usize) -> String {
    if n < 1024 {
        format!("{} chars", n)
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{:.1} MB", n as f64 / 1024.0 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_plain_line() {
        let html = render_line_html(
            "hello world".to_string(),
            "[]".to_string(),
            "[]".to_string(),
        );
        assert_eq!(html, "<span>hello world</span>");
    }

    #[test]
    fn test_render_with_tokens() {
        let tokens = r#"[{"start":0,"end":5,"className":"keyword"}]"#;
        let html = render_line_html(
            "hello world".to_string(),
            tokens.to_string(),
            "[]".to_string(),
        );
        assert!(html.contains("<span class=\"keyword\">hello</span>"));
    }

    #[test]
    fn test_render_html_escapes() {
        let tokens = r#"[{"start":0,"end":5,"className":"tag"}]"#;
        let html = render_line_html("<div>".to_string(), tokens.to_string(), "[]".to_string());
        assert!(html.contains("&lt;div&gt;"));
    }

    #[test]
    fn test_render_with_decorations() {
        let decos = r#"[{"start":0,"end":5,"className":"diff-inserted","isInline":true}]"#;
        let html = render_line_html(
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
        let results = render_lines_html(lines, all_tokens.to_string(), "[[],[]]".to_string());
        assert_eq!(results.len(), 2);
        assert!(
            results[0].contains("keyword"),
            "result[0] '{}' lacks keyword",
            results[0]
        );
        assert!(
            results[1].contains("string"),
            "result[1] '{}' lacks string",
            results[1]
        );
    }

    #[test]
    fn test_render_empty_token_list() {
        let html = render_line_html("".to_string(), "[]".to_string(), "[]".to_string());
        assert_eq!(html, "<span></span>");
    }

    #[test]
    fn test_render_minimap_basic() {
        let tokens = r#"[{"start":0,"end":4,"className":"keyword"}]"#;
        let out = render_minimap_line("test".to_string(), tokens.to_string(), 1);
        assert!(!out.is_empty());
    }
}
