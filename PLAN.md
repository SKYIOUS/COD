# Rust Acceleration Plan

## Priority: Syntax Highlighting → File Search → Rendering

---

## 1. Syntax Highlighting (tokenization)

**Goal:** Replace JS TextMate/Tree-sitter token pipeline with Rust for supported languages.

**What exists:**
- `native/src/tokenize.rs` — `create_tokens_from_captures` (scope stacking, bracket detection, gap handling)
- `native/src/treesitter.rs` — `parse_with_tree_sitter` (8 languages: Rust, TS, JS, Python, Go, Java, JSON)

**Integration steps:**

| Step | File | What |
|------|------|------|
| 1.1 | `src/vs/editor/common/languages/supports/treeSitterTokenizationImpl.ts` | Replace `_createTokensFromCaptures` call with `nativeCreateTokensFromCapturesSync` for supported languages |
| 1.2 | `native/src/treesitter.rs` | Add `.scm` query file support (currently emits all nodes — target specific captures only) |
| 1.3 | `native/src/treesitter.rs` | Add incremental parsing (keep `Tree` between edits via callback) |
| 1.4 | New: `native/src/grammars/` | Ship pre-compiled `tree-sitter-*.wasm` or link more `tree-sitter-*` crates for full language coverage (target: top-20 languages) |
| 1.5 | `src/vs/editor/common/services/languageService.ts` | Route "language supports tree-sitter" → Rust path, else fall back to TextMate |

**Skip:** TextMate grammar parsing in Rust (`.tmLanguage` XML/plist is huge surface). Keep TextMate for languages without tree-sitter support.

---

## 2. File Indexing / Search

**Goal:** Replace ripgrep child-process with in-process Rust search for non-PCRE patterns.

**What exists:**
- `native/src/search.rs` — `search_files` (ignore + regex), glob filtering via `globset`, binary ext skip list
- Cancellation via `AtomicBool` flag

**Integration steps:**

| Step | File | What |
|------|------|------|
| 2.1 | `src/vs/workbench/services/search/node/ripgrepTextSearchEngine.ts` | Fast-path: short non-regex patterns → Rust search directly |
| 2.2 | `native/src/search.rs` | Add file index (not a full index — just a cached file list with mtime). Walk once, re-run pattern on cached paths. |
| 2.3 | `native/src/search.rs` | Add preview context lines (±N lines around match) |
| 2.4 | `native/src/search.rs` | Memory-safe large file handling: read in chunks, not `read_to_string` |
| 2.5 | `src/vs/workbench/services/search/node/ripgrepTextSearchEngine.ts` | Add cancellation: pass shared `AtomicBool` from JS `CancellationToken` |

**Skip:** Full-text index (Lucene-style). VS Code's model is file-system-scan-on-query, not pre-index.

---

## 3. Rendering Pipeline

**Goal:** Generate viewport HTML in Rust instead of JS string concat.

**What exists:**
- `native/src/render.rs` — `render_line_html`, `render_lines_html`, `render_minimap_line`

**Integration steps:**

| Step | File | What |
|------|------|------|
| 3.1 | `src/vs/editor/browser/viewParts/viewLines/viewLine.ts` | Replace `renderLine` inner HTML generation with `nativeRenderLineHtmlSync` (token + decoration spans) |
| 3.2 | `native/src/render.rs` | Add line number gutter, folding controls, indentation guides to HTML output |
| 3.3 | `native/src/render.rs` | Add text measurement via `rustybuzz` (HarfBuzz) for line breaking + cursor positioning |
| 3.4 | `src/vs/editor/browser/view/viewLayer.ts` | Batch viewport lines → `nativeRenderLinesHtmlSync` for entire visible range |
| 3.5 | `native/src/render.rs` | Add font fallback, tab size config, RTL/bidi support |
| 3.6 | `src/vs/editor/browser/viewParts/minimap/minimap.ts` | Replace minimap char rendering with `nativeRenderMinimapLineSync` |

**Skip:** Full GPU rendering (WebGL/Canvas). The current `ViewLinesGpu` experiment covers that path.

---

## Dependency Graph

```
Syntax Highlighting (1.x)  ──┐
                              ├──> Rendering (3.x) — needs computed token data
File Search (2.x)            ──┘  (independent, no blocking deps)
```

- Syntax highlighting and search are independent → parallelize
- Rendering depends on token data format stable first (step 1.1 done → 3.1 starts)
