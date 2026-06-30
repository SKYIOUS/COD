use serde::Serialize;

#[derive(Clone, Serialize)]
#[napi(object)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: i32,
    pub line_content: String,
    pub match_start: i32,
    pub match_end: i32,
}

#[napi]
pub fn search_files(root: String, pattern: String, max_results: i32) -> Vec<SearchMatch> {
    let max = if max_results <= 0 { 100 } else { max_results as usize };
    let mut results: Vec<SearchMatch> = Vec::with_capacity(max.min(1000));

    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return results,
    };

    let walker = ignore::WalkBuilder::new(&root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .build();

    for entry in walker.flatten() {
        if results.len() >= max { break; }
        let path = entry.path();
        if !path.is_file() { continue; }

        // ponytail: skip binary extensions, add configurable list if needed
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "pyc" | "class" | "wasm" | "zip" | "gz" | "png" | "jpg" | "ico" | "woff2") {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let rel = path.strip_prefix(&root).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            for (i, line) in content.lines().enumerate() {
                if results.len() >= max { break; }
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

#[napi]
pub fn search_files_chunked(root: String, pattern: String, max_results: i32, chunk_size: i32) -> Vec<Vec<SearchMatch>> {
    let all = search_files(root, pattern, max_results);
    let cs = if chunk_size <= 0 { 50 } else { chunk_size as usize };
    all.chunks(cs).map(|c| c.to_vec()).collect()
}
