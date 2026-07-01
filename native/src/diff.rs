use serde::Serialize;

#[derive(Serialize)]
#[napi(object)]
pub struct SequenceDiff {
    pub start1: i32,
    pub end1: i32,
    pub start2: i32,
    pub end2: i32,
}

#[derive(Serialize)]
#[napi(object)]
pub struct DiffChange {
    pub original_start: i32,
    pub original_length: i32,
    pub modified_start: i32,
    pub modified_length: i32,
}

#[napi]
pub fn myers_diff(a: Vec<i32>, b: Vec<i32>) -> Vec<SequenceDiff> {
    let n = a.len() as i32;
    let m = b.len() as i32;

    if n == 0 && m == 0 {
        return Vec::new();
    }
    if n == 0 {
        return vec![SequenceDiff {
            start1: 0,
            end1: 0,
            start2: 0,
            end2: m,
        }];
    }
    if m == 0 {
        return vec![SequenceDiff {
            start1: 0,
            end1: n,
            start2: 0,
            end2: 0,
        }];
    }

    let max_d = (n + m) as usize;
    let size = 2 * max_d + 3;
    let off = max_d as i32;

    let mut v: Vec<i32> = vec![-1; size];
    v[(1 + off) as usize] = 0;

    let mut trace: Vec<Vec<i32>> = Vec::new();

    let d32 = 0i32;
    let mut k = -d32;
    while k <= d32 {
        let x = if k == -d32
            || (k != d32 && v[((k - 1) + off) as usize] < v[((k + 1) + off) as usize])
        {
            v[((k + 1) + off) as usize]
        } else {
            v[((k - 1) + off) as usize] + 1
        };
        let mut x = x;
        let mut y = x - k;
        while x < n && y < m && a[x as usize] == b[y as usize] {
            x += 1;
            y += 1;
        }
        v[(k + off) as usize] = x;
        if x >= n && y >= m {
            return Vec::new();
        }
        k += 2;
    }
    trace.push(v.clone());

    for d in 1..=max_d {
        let d32 = d as i32;
        let mut k = -d32;
        while k <= d32 {
            let down = k == -d32
                || (k != d32 && v[((k - 1) + off) as usize] < v[((k + 1) + off) as usize]);
            let x = if down {
                v[((k + 1) + off) as usize]
            } else {
                v[((k - 1) + off) as usize] + 1
            };
            let mut x = x;
            let mut y = x - k;
            while x < n && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[(k + off) as usize] = x;

            if x >= n && y >= m {
                let mut edits: Vec<(i32, i32, bool)> = Vec::new();
                let mut px = n;
                let mut py = m;

                for tdi in (1..=d).rev() {
                    let vd = &trace[tdi - 1];
                    let kk = px - py;
                    let td32 = tdi as i32;
                    let down = kk == -td32
                        || (kk != td32
                            && vd[((kk - 1) + off) as usize] < vd[((kk + 1) + off) as usize]);
                    let prev_k = if down { kk + 1 } else { kk - 1 };
                    let prev_x = vd[(prev_k + off) as usize];
                    let prev_y = prev_x - prev_k;
                    edits.push((prev_x, prev_y, down));
                    px = prev_x;
                    py = prev_y;
                }

                edits.reverse();

                let mut diffs: Vec<SequenceDiff> = Vec::new();
                let mut idx = 0;
                while idx < edits.len() {
                    let (ex, ey, _is_down) = edits[idx];
                    let s1 = ex;
                    let mut e1 = ex;
                    let s2 = ey;
                    let mut e2 = ey;

                    let mut j = idx;
                    while j < edits.len() {
                        let (nx, ny, nd) = edits[j];
                        if nd {
                            if nx == e1 && ny == e2 {
                                e2 = ny + 1;
                            } else {
                                break;
                            }
                        } else {
                            if nx == e1 && ny == e2 {
                                e1 = nx + 1;
                            } else {
                                break;
                            }
                        }
                        j += 1;
                    }

                    if s1 != e1 || s2 != e2 {
                        diffs.push(SequenceDiff {
                            start1: s1,
                            end1: e1,
                            start2: s2,
                            end2: e2,
                        });
                    }

                    idx = j;
                }

                return diffs;
            }

            k += 2;
        }
        trace.push(v.clone());
    }

    Vec::new()
}

#[napi]
pub fn lcs_diff(a: Vec<i32>, b: Vec<i32>) -> Vec<DiffChange> {
    let n = a.len();
    let m = b.len();

    let mut dp = vec![vec![0i32; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = std::cmp::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    let mut changes = Vec::new();
    let mut i = n;
    let mut j = m;
    let mut orig_start = n;
    let mut mod_start = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
            let orig_len = orig_start - i;
            let mod_len = mod_start - j;
            if orig_len > 0 || mod_len > 0 {
                changes.push(DiffChange {
                    original_start: i as i32,
                    original_length: orig_len as i32,
                    modified_start: j as i32,
                    modified_length: mod_len as i32,
                });
            }
            orig_start = i - 1;
            mod_start = j - 1;
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            j -= 1;
        } else if i > 0 {
            i -= 1;
        } else {
            break;
        }
    }

    if orig_start > 0 || mod_start > 0 {
        changes.push(DiffChange {
            original_start: 0,
            original_length: orig_start as i32,
            modified_start: 0,
            modified_length: mod_start as i32,
        });
    }

    changes.reverse();
    changes
}

fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

// ponytail: character-level Myers diff for similarity check. Ports `areLinesSimilar` from computeMovedLines.ts
fn char_myers_common(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len();
    let m = b.len();
    if n == 0 || m == 0 {
        return 0;
    }

    let max_d = n + m;
    let size = 2 * max_d + 3;
    let off = max_d as i32;
    let mut v: Vec<i32> = vec![-1; size];
    v[(1 + off) as usize] = 0;

    let mut traces: Vec<Vec<i32>> = Vec::new();

    // d = 0: only diagonal k=0 matters
    let mut x = 0i32;
    let mut y = 0i32;
    while (x as usize) < n && (y as usize) < m && a[x as usize] == b[y as usize] {
        x += 1;
        y += 1;
    }
    v[(0 + off) as usize] = x;
    if x as usize >= n && y as usize >= m {
        return count_non_space(a, 0, n as i32);
    }
    traces.push(v.clone());

    for d in 1..=max_d {
        let d32 = d as i32;
        let mut k = -d32;
        while k <= d32 {
            let down = k == -d32
                || (k != d32
                    && traces[d - 1][((k - 1) + off) as usize]
                        < traces[d - 1][((k + 1) + off) as usize]);
            let x = if down {
                traces[d - 1][((k + 1) + off) as usize]
            } else {
                traces[d - 1][((k - 1) + off) as usize] + 1
            };
            let mut x = x;
            let mut y = x - k;
            while (x as usize) < n && (y as usize) < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[(k + off) as usize] = x;
            if x as usize == n && y as usize == m {
                traces.push(v.clone());
                let common = count_matching_inverted(&traces, n as i32, m as i32, d, off, a);
                return common;
            }
            k += 2;
        }
        traces.push(v.clone());
    }
    0
}

fn count_non_space(s: &[u8], start: i32, end: i32) -> i32 {
    let mut c = 0;
    for i in start..end {
        if (i as usize) < s.len() && !is_space(s[i as usize]) {
            c += 1;
        }
    }
    c
}

fn count_matching_inverted(
    traces: &[Vec<i32>],
    n: i32,
    m: i32,
    d: usize,
    off: i32,
    a: &[u8],
) -> i32 {
    let mut common = 0i32;
    let mut px = n;
    let mut py = m;
    for tdi in (1..=d).rev() {
        let vd = &traces[tdi - 1];
        let kk = px - py;
        let td32 = tdi as i32;
        let down = kk == -td32
            || (kk != td32 && vd[((kk - 1) + off) as usize] < vd[((kk + 1) + off) as usize]);
        let prev_k = if down { kk + 1 } else { kk - 1 };
        let prev_x = vd[(prev_k + off) as usize];
        let prev_y = prev_x - prev_k;
        // The diagonal (snake) from (prev_x, prev_y) to (px, py) is matching characters
        for i in prev_x..px {
            if !is_space(a[i as usize]) {
                common += 1;
            }
        }
        px = prev_x;
        py = prev_y;
    }
    common
}

#[napi]
pub fn lines_similar(line1: String, line2: String) -> bool {
    let l1 = line1.trim();
    let l2 = line2.trim();
    if l1 == l2 {
        return true;
    }
    if l1.len() > 300 && l2.len() > 300 {
        return false;
    }

    let common = char_myers_common(l1.as_bytes(), l2.as_bytes());

    let longer = if l1.len() > l2.len() { l1 } else { l2 };
    let total_non_space = longer.bytes().filter(|&c| !is_space(c)).count() as i32;

    total_non_space > 10 && common * 10 > total_non_space * 6
}
