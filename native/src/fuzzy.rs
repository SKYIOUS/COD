use serde::Serialize;

#[derive(Serialize)]
#[napi(object)]
pub struct FuzzyScoreResult {
    pub score: i32,
    pub label: Option<String>,
    pub matches: Vec<i32>,
}

#[derive(Serialize)]
#[napi(object)]
pub struct FuzzyScore {
    pub score: i32,
    pub matches: Vec<i32>,
}

#[derive(Serialize)]
#[napi(object)]
pub struct PreparedQuery {
    pub original: String,
    pub normalized: String,
    pub normalized_lowercase: String,
}

fn char_is_upper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

fn char_is_lower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

fn char_is_separator(c: u8) -> bool {
    c == b'.'
        || c == b'_'
        || c == b'-'
        || c == b'/'
        || c == b'\\'
        || c == b' '
        || c == b'\''
        || c == b'"'
        || c == b':'
        || c == b'~'
}

fn char_is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

#[napi]
pub fn fuzzy_score(pattern: String, word: String) -> Option<FuzzyScoreResult> {
    let pattern = pattern.as_bytes();
    let word = word.as_bytes();
    let p_len = pattern.len();
    let w_len = word.len();

    if p_len == 0 || w_len == 0 || p_len > 128 || w_len > 128 {
        return None;
    }

    let mut pi = 0;
    let mut found = false;
    for &wc in word {
        if pi < p_len
            && (wc == pattern[pi]
                || wc.to_ascii_uppercase() == pattern[pi]
                || wc.to_ascii_lowercase() == pattern[pi])
        {
            pi += 1;
            if pi == p_len {
                found = true;
                break;
            }
        }
    }
    if !found {
        return None;
    }

    let max_size = 128;
    let mut score_table = vec![0i32; max_size * max_size];
    let mut arrow_table = vec![0i32; max_size * max_size];
    let mut min_word_positions = vec![0i32; p_len];
    let mut max_word_positions = vec![0i32; p_len];

    for i in 0..p_len {
        min_word_positions[i] = -1;
        max_word_positions[i] = -1;
    }

    for pi in 0..p_len {
        let pc = pattern[pi];
        for wi in 0..w_len {
            let wc = word[wi];
            if wc == pc || wc.to_ascii_lowercase() == pc || wc.to_ascii_uppercase() == pc {
                if min_word_positions[pi] < 0 || (wi as i32) < min_word_positions[pi] {
                    min_word_positions[pi] = wi as i32;
                }
                if max_word_positions[pi] < 0 || (wi as i32) > max_word_positions[pi] {
                    max_word_positions[pi] = wi as i32;
                }
            }
        }
    }

    for pi in 0..p_len {
        for wi in 0..w_len {
            let idx = pi * max_size + wi;
            score_table[idx] = 0;
            arrow_table[idx] = 0;
        }
    }

    for pi in 0..p_len {
        for wi in 0..w_len {
            let wc = word[wi];
            let pc = pattern[pi];
            if wc != pc && wc.to_ascii_lowercase() != pc && wc.to_ascii_uppercase() != pc {
                continue;
            }

            let score: i32;
            let arrow: i32;
            let mut gap_before: bool = false;
            let mut gap_location: bool = false;

            if pi == 0 && wi == 0 {
                score = 1;
                arrow = 1;
            } else if pi == 0 {
                if wi > 0 && char_is_upper(wc) && char_is_lower(word[wi - 1]) {
                    score = 7;
                } else if wi > 0 && char_is_separator(word[wi - 1]) {
                    score = 5;
                } else {
                    score = -1;
                }
                arrow = 1;
            } else {
                let mut best_score = std::i32::MIN;
                let mut best_arrow = 0;

                for wj in 0..wi {
                    let prev_idx = (pi - 1) * max_size + wj;
                    let prev_score = score_table[prev_idx];
                    if prev_score == 0 && !(pi - 1 == 0 && wj == 0) {
                        continue;
                    }

                    let mut cur_score = prev_score;
                    let gap = (wi - wj) as i32;

                    if gap == 1 {
                        cur_score += 1;
                    } else {
                        if wj > 0
                            && (char_is_separator(word[wj - 1]) || char_is_whitespace(word[wj - 1]))
                        {
                            gap_before = true;
                        }
                        if gap > 1 {
                            gap_location = true;
                        }
                        if gap_before {
                            gap_location = true;
                        }
                        if gap_location {
                            cur_score -= 3;
                        } else {
                            cur_score -= 5;
                        }
                    }

                    let same_case = wc == pc;
                    let first_lower = pi == 0;

                    if first_lower {
                        cur_score += 1;
                    } else if same_case {
                        cur_score += 7;
                    } else {
                        cur_score += 5;
                    }

                    if wi > 0 && char_is_separator(word[wi - 1]) {
                        cur_score += 5;
                    }

                    if cur_score > best_score {
                        best_score = cur_score;
                        best_arrow = wj as i32 + 1;
                    }
                }

                score = best_score;
                arrow = best_arrow;
            }

            if score != 0 || (pi == 0 && wi == 0) {
                score_table[pi * max_size + wi] = score;
                arrow_table[pi * max_size + wi] = arrow;
            }
        }
    }

    let mut best_score = std::i32::MIN;
    let mut best_wi = 0;
    for wi in 0..w_len {
        let score = score_table[(p_len - 1) * max_size + wi];
        if score > best_score {
            best_score = score;
            best_wi = wi;
        }
    }

    if best_score == std::i32::MIN {
        return None;
    }

    let mut matches = Vec::new();
    let mut pi = p_len as i32 - 1;
    let mut wi = best_wi as i32;

    while pi >= 0 && wi >= 0 {
        let arrow = arrow_table[(pi as usize) * max_size + (wi as usize)];
        if arrow == 0 {
            break;
        }
        matches.push(wi);
        wi = arrow - 1;
        pi -= 1;
    }

    matches.reverse();

    Some(FuzzyScoreResult {
        score: best_score,
        label: None,
        matches,
    })
}

fn compute_char_score(
    target_char: u8,
    query_char: u8,
    target_index: usize,
    target_bytes: &[u8],
    seq_len: i32,
) -> i32 {
    let mut score: i32 = 0;

    let same_case = target_char == query_char;
    if same_case {
        score += 1;
    }

    if seq_len > 0 {
        score += std::cmp::min(seq_len, 3) * 6 + std::cmp::max(0, seq_len - 3) * 3;
    }

    if target_index == 0 {
        score += 8;
    }

    if target_index > 0 {
        if char_is_separator(target_bytes[target_index - 1]) {
            score += 5;
        }
    }

    if target_index > 0 {
        if char_is_upper(target_bytes[target_index - 1]) && seq_len == 0 {
            score += 2;
        }
    }

    score
}

#[napi]
pub fn score_fuzzy(
    target: String,
    query: String,
    query_lower: String,
    _allow_non_contiguous: bool,
) -> FuzzyScore {
    let target = target.as_bytes();
    let query = query.as_bytes();
    let query_lower = query_lower.as_bytes();
    let t_len = target.len();
    let q_len = query.len();

    if t_len == 0 || q_len == 0 {
        return FuzzyScore {
            score: 0,
            matches: Vec::new(),
        };
    }

    let size = q_len * t_len;
    let mut scores = vec![0i32; size];
    let mut matches = vec![0i32; size];

    for qi in 0..q_len {
        let qc = query[qi];
        let qlc = query_lower[qi];

        for ti in 0..t_len {
            let tc = target[ti];
            let idx = qi * t_len + ti;

            if tc != qlc && tc.to_ascii_lowercase() != qlc {
                if qi == 0 {
                    scores[idx] = 0;
                    matches[idx] = -1;
                } else {
                    let left_idx = if ti > 0 { qi * t_len + (ti - 1) } else { idx };
                    let diag_idx = if qi > 0 && ti > 0 {
                        (qi - 1) * t_len + (ti - 1)
                    } else {
                        idx
                    };
                    scores[idx] = std::cmp::max(
                        scores.get(left_idx).copied().unwrap_or(0),
                        scores.get(diag_idx).copied().unwrap_or(0),
                    );
                    matches[idx] = -1;
                }
                continue;
            }

            let mut seq_len: i32 = 0;
            if qi > 0 && ti > 0 {
                let prev_match = matches
                    .get((qi - 1) * t_len + (ti - 1))
                    .copied()
                    .unwrap_or(-1);
                if prev_match >= 0 {
                    seq_len = prev_match + 1;
                }
            }

            let char_score = compute_char_score(tc, qc, ti, target, seq_len);

            let mut best_score = char_score;
            let mut best_match = seq_len;

            if qi > 0 && ti > 0 {
                let diag = scores[(qi - 1) * t_len + (ti - 1)] + char_score;
                if diag > best_score {
                    best_score = diag;
                    best_match = seq_len;
                }
            }

            if ti > 0 {
                let left = scores[qi * t_len + (ti - 1)] - 1;
                if left > best_score {
                    best_score = left;
                    best_match = -1;
                }
            }

            if qi > 0 {
                let up = scores[(qi - 1) * t_len + ti] - 1;
                if up > best_score {
                    best_score = up;
                    best_match = -1;
                }
            }

            scores[idx] = best_score;
            matches[idx] = best_match;
        }
    }

    let mut result_matches = Vec::new();
    let total_score = if q_len > 0 && t_len > 0 {
        scores[q_len * t_len - 1]
    } else {
        0
    };
    if q_len > 0 && t_len > 0 && total_score > 0 {
        let mut qi = q_len as i32 - 1;
        let mut ti = t_len as i32 - 1;
        while qi >= 0 && ti >= 0 {
            let idx = (qi as usize) * t_len + (ti as usize);
            let m = matches[idx];
            if m >= 0 {
                result_matches.push(ti);
                qi -= 1;
                ti -= 1;
            } else {
                let left = if ti > 0 {
                    scores[(qi as usize) * t_len + ((ti - 1) as usize)]
                } else {
                    std::i32::MIN
                };
                let up = if qi > 0 {
                    scores[((qi - 1) as usize) * t_len + (ti as usize)]
                } else {
                    std::i32::MIN
                };
                if left >= up {
                    ti -= 1;
                } else {
                    qi -= 1;
                }
            }
        }
    }
    result_matches.reverse();

    FuzzyScore {
        score: total_score,
        matches: result_matches,
    }
}

#[napi]
pub fn prepare_query(original: String) -> PreparedQuery {
    let normalized = original
        .chars()
        .filter(|&c| c != '*' && c != '\u{2026}' && c != '"' && c != '\'')
        .collect::<String>()
        .trim()
        .to_string();

    let normalized_lowercase = normalized.to_lowercase();

    PreparedQuery {
        original,
        normalized,
        normalized_lowercase,
    }
}
