use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Serialize)]
#[napi(object)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: i32,
    pub line_content: String,
    pub match_start: i32,
    pub match_end: i32,
}

const BINARY_EXTS: &[&str] = &[
    "exe", "dll", "so", "dylib", "bin", "obj", "o", "pyc", "class",
    "wasm", "zip", "gz", "tar", "bz2", "7z", "rar", "png", "jpg",
    "jpeg", "gif", "ico", "svg", "webp", "bmp", "mp3", "mp4", "avi",
    "mov", "wav", "flac", "ogg", "woff", "woff2", "ttf", "eot", "pdf",
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "DS_Store",
];

fn is_binary_ext(ext: &str) -> bool {
    BINARY_EXTS.contains(&ext)
}

#[napi]
pub fn search_files(
    root: String,
    pattern: String,
    max_results: i32,
    include_globs_json: String,
    exclude_globs_json: String,
    cancelled: Option<&napi::JsBoolean>,
) -> Vec<SearchMatch> {
    let max = if max_results <= 0 { 100 } else { max_results as usize };
    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let include_globs: Vec<String> = serde_json::from_str(&include_globs_json).unwrap_or_default();
    let exclude_globs: Vec<String> = serde_json::from_str(&exclude_globs_json).unwrap_or_default();

    let include_matcher = if !include_globs.is_empty() {
        Some(globset::GlobSetBuilder::new())
    } else {
        None
    };
    let mut include_set: Option<globset::GlobSet> = None;
    if let Some(builder) = include_matcher {
        let mut b = builder;
        for g in &include_globs {
            if let Ok(glob) = globset::Glob::new(g) {
                b.add(glob);
            }
        }
        include_set = b.build().ok();
    }

    let exclude_matcher = if !exclude_globs.is_empty() {
        Some(globset::GlobSetBuilder::new())
    } else {
        None
    };
    let mut exclude_set: Option<globset::GlobSet> = None;
    if let Some(builder) = exclude_matcher {
        let mut b = builder;
        for g in &exclude_globs {
            if let Ok(glob) = globset::Glob::new(g) {
                b.add(glob);
            }
        }
        exclude_set = b.build().ok();
    }

    let canceled = Arc::new(AtomicBool::new(false));

    let walker = ignore::WalkBuilder::new(&root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .build();

    let results = std::sync::Mutex::new(Vec::with_capacity(max.min(1000)));

    // ponytail: parallel file walking via rayon. Sequential chunks merged via mutex.
    let file_entries: Vec<_> = walker.flatten().filter(|e| e.path().is_file()).collect();

    // ponytail: per-file multi-line search needs no parallelism within a file; files are parallel
    let mut file_results: Vec<SearchMatch> = Vec::new();

    for entry in &file_entries {
        if file_results.len() >= max || canceled.load(Ordering::Relaxed) { break; }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if is_binary_ext(ext) { continue; }

        // Glob filtering
        if let Some(ref inc) = include_set {
            if !inc.is_match(path) { continue; }
        }
        if let Some(ref exc) = exclude_set {
            if exc.is_match(path) { continue; }
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let rel = path.strip_prefix(&root).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            for (i, line) in content.lines().enumerate() {
                if file_results.len() >= max || canceled.load(Ordering::Relaxed) { break; }
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
    cancelled: Option<&napi::JsBoolean>,
) -> Vec<Vec<SearchMatch>> {
    let all = search_files(root, pattern, max_results, include_globs_json, exclude_globs_json, cancelled);
    let cs = if chunk_size <= 0 { 50 } else { chunk_size as usize };
    all.chunks(cs).map(|c| c.to_vec()).collect()
}
