use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
#[napi(object)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: i32,
    pub line_content: String,
    pub match_start: i32,
    pub match_end: i32,
}

#[derive(Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct IndexedFile {
    pub path: String,
    pub mtime: i64,
    pub size: i64,
    pub first_line: String,
}

const BINARY_EXTS: &[&str] = &[
    "exe", "dll", "so", "dylib", "bin", "obj", "o", "pyc", "class", "wasm", "zip", "gz", "tar",
    "bz2", "7z", "rar", "png", "jpg", "jpeg", "gif", "ico", "svg", "webp", "bmp", "mp3", "mp4",
    "avi", "mov", "wav", "flac", "ogg", "woff", "woff2", "ttf", "eot", "pdf", "doc", "docx", "xls",
    "xlsx", "ppt", "pptx", "DS_Store",
];

fn is_binary_ext(ext: &str) -> bool {
    BINARY_EXTS.contains(&ext)
}

fn build_glob_set(globs_json: &str) -> Option<globset::GlobSet> {
    let globs: Vec<String> = serde_json::from_str(globs_json).unwrap_or_default();
    if globs.is_empty() {
        return None;
    }
    let mut builder = globset::GlobSetBuilder::new();
    for g in &globs {
        if let Ok(glob) = globset::Glob::new(g) {
            builder.add(glob);
        }
    }
    builder.build().ok()
}

#[napi]
pub fn search_files(
    root: String,
    pattern: String,
    max_results: i32,
    include_globs_json: String,
    exclude_globs_json: String,
) -> Vec<SearchMatch> {
    let max = if max_results <= 0 {
        100
    } else {
        max_results as usize
    };
    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let include_set = build_glob_set(&include_globs_json);
    let exclude_set = build_glob_set(&exclude_globs_json);

    let walker = ignore::WalkBuilder::new(&root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .build();

    let file_entries: Vec<_> = walker.flatten().filter(|e| e.path().is_file()).collect();
    let mut file_results: Vec<SearchMatch> = Vec::new();

    for entry in &file_entries {
        if file_results.len() >= max {
            break;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if is_binary_ext(ext) {
            continue;
        }

        if let Some(ref inc) = include_set {
            if !inc.is_match(path) {
                continue;
            }
        }
        if let Some(ref exc) = exclude_set {
            if exc.is_match(path) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let rel = path
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            for (i, line) in content.lines().enumerate() {
                if file_results.len() >= max {
                    break;
                }
                if let Some(m) = re.find(line) {
                    file_results.push(SearchMatch {
                        path: rel.clone(),
                        line_number: (i + 1) as i32,
                        line_content: line.to_string(),
                        match_start: m.start() as i32,
                        match_end: m.end() as i32,
                    });
                }
            }
        }
    }

    file_results
}

#[napi]
pub fn search_files_chunked(
    root: String,
    pattern: String,
    max_results: i32,
    chunk_size: i32,
    include_globs_json: String,
    exclude_globs_json: String,
    start_offset: i32,
) -> Vec<SearchMatch> {
    let all_files = collect_files(&root, &include_globs_json, &exclude_globs_json);
    let max = if max_results <= 0 {
        100
    } else {
        max_results as usize
    };
    let cs = if chunk_size <= 0 {
        50
    } else {
        chunk_size as usize
    };
    let start = if start_offset < 0 {
        0
    } else {
        start_offset as usize
    };

    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut results: Vec<SearchMatch> = Vec::new();
    let end = (start + cs).min(all_files.len());

    for file_idx in start..end {
        if results.len() >= max {
            break;
        }
        let (rel, path) = &all_files[file_idx];
        if let Ok(content) = std::fs::read_to_string(path) {
            for (i, line) in content.lines().enumerate() {
                if results.len() >= max {
                    break;
                }
                if let Some(m) = re.find(line) {
                    results.push(SearchMatch {
                        path: rel.clone(),
                        line_number: (i + 1) as i32,
                        line_content: line.to_string(),
                        match_start: m.start() as i32,
                        match_end: m.end() as i32,
                    });
                }
            }
        }
    }

    results
}

fn collect_files(
    root: &str,
    include_globs_json: &str,
    exclude_globs_json: &str,
) -> Vec<(String, String)> {
    let include_set = build_glob_set(include_globs_json);
    let exclude_set = build_glob_set(exclude_globs_json);

    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .build();

    walker
        .flatten()
        .filter(|e| e.path().is_file())
        .filter(|e| {
            let ext = e.path().extension().and_then(|e| e.to_str()).unwrap_or("");
            !is_binary_ext(ext)
        })
        .filter(|e| {
            let path = e.path();
            if let Some(ref inc) = include_set {
                inc.is_match(path)
            } else {
                true
            }
        })
        .filter(|e| {
            let path = e.path();
            if let Some(ref exc) = exclude_set {
                !exc.is_match(path)
            } else {
                true
            }
        })
        .map(|e| {
            let rel = e
                .path()
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let abs = e.path().to_string_lossy().to_string();
            (rel, abs)
        })
        .collect()
}

#[napi]
pub fn index_directory(
    root: String,
    include_globs_json: String,
    exclude_globs_json: String,
) -> Vec<IndexedFile> {
    let files = collect_files(&root, &include_globs_json, &exclude_globs_json);
    let mut indexed: Vec<IndexedFile> = Vec::with_capacity(files.len());

    for (rel, abs) in &files {
        if let Ok(meta) = std::fs::metadata(abs) {
            use std::time::SystemTime;
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let first_line = std::fs::read_to_string(abs)
                .ok()
                .and_then(|c| c.lines().next().map(|l| l.to_string()))
                .unwrap_or_default();
            indexed.push(IndexedFile {
                path: rel.clone(),
                mtime,
                size: meta.len() as i64,
                first_line,
            });
        }
    }

    indexed
}

#[napi]
pub fn search_index(pattern: String, index_json: String, max_results: i32) -> Vec<SearchMatch> {
    let max = if max_results <= 0 {
        100
    } else {
        max_results as usize
    };
    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let files: Vec<IndexedFile> = serde_json::from_str(&index_json).unwrap_or_default();

    let mut results: Vec<SearchMatch> = Vec::new();
    for file in &files {
        if results.len() >= max {
            break;
        }
        if let Some(m) = re.find(&file.first_line) {
            results.push(SearchMatch {
                path: file.path.clone(),
                line_number: 1,
                line_content: file.first_line.clone(),
                match_start: m.start() as i32,
                match_end: m.end() as i32,
            });
        }
        if let Some(m) = re.find(&file.path) {
            results.push(SearchMatch {
                path: file.path.clone(),
                line_number: 0,
                line_content: String::new(),
                match_start: m.start() as i32,
                match_end: m.end() as i32,
            });
        }
    }

    results
}
