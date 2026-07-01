# Remaining Work & Full App Inspection

**Generated:** July 2026
**Rust native modules:** 12 source files, 22+ exported functions
**CI/CD:** 9 workflows, 21-step build-release pipeline

---

## 1. What's Wired vs Not Wired

### ✅ Fully wired (Rust → JS, called in production path)

| Module | Rust function | JS callers | Status |
|--------|-------------|-----------|--------|
| `color.rs` | `parseCssColor` | `color.ts` | Live |
| `hash.rs` | `numberHash`, `objectHash`, `stringHash` | `hash.ts` | Live |
| `hash.rs` | `stringSha1` | Various | Live |
| `jsonc.rs` | `parseJsonc` | `jsonc.ts` | Live |
| `encoding.rs` | `nativeEncodeHex`, `decodeHex`, `encodeBase64`, `decodeBase64` | `buffer.ts` | Live |
| `fuzzy.rs` | `fuzzyScore`, `scoreFuzzy` | `filters.ts`, `fuzzyScorer.ts` | Live |
| `diff.rs` | `myersDiff`, `linesSimilar` | `myersDiffAlgorithm.ts`, `computeMovedLines.ts` | Live |
| `welcome.rs` | `codLogoHtml`, `codAboutHtml` | Getting started page | Live |
| `search.rs` | `searchFiles`, `searchFilesChunked` | `ripgrepTextSearchEngine.ts` | **Fast path live** |
| `tokenize.rs` | `createTokensFromCapturesScoped` | `treeSitterTokenizationImpl.ts` | **Just wired** |

### 🟡 Partially wired (exports exist, limited callers)

| Module | Rust function | Callers | Gap |
|--------|-------------|---------|-----|
| `tokenize.rs` | `tokensToUint32Array` | None | No caller in JS. The JS `_endOffsetTokensToUint32Array` at line 781 is 9 lines — not worth the napi overhead |
| `treesitter.rs` | `queryTreeSitter` | None | Native bridge exists but `treeSitterSyntaxTokenBackend.ts` still uses WASM. Need to add fallback: try Rust first, WASM on failure |
| `treesitter.rs` | `parseWithTreeSitter` | None | Same — `collectAllNodes` is useful for debugging only |
| `search.rs` | `indexDirectory`, `searchIndex` | None | Ready for use — JS needs to manage index lifecycle (cache, invalidate on file change) |
| `render.rs` | `renderLineHtml`, `renderLinesHtml`, `renderMinimapLine` | `nativeRender.ts` wrappers exist | **No caller in rendering pipeline.** `ViewLine.renderLine` at `viewLine.ts:200` calls JS `renderViewLine` directly |

### ❌ Not wired (Rust exists, no JS integration)

| Module | Function | Why not wired |
|--------|----------|--------------|
| `render.rs` | `renderLineHtml` | Character mapping parity needed — Rust doesn't produce `CharacterMapping` for cursor hit-testing |
| `render.rs` | `render_minimap_line` | `minimap.ts` line rendering is tied into `ViewLine` output — not a simple slot |
| `treesitter.rs` | `queryTreeSitter` | Must replace or augment `TreeSitterLibraryService`'s WASM parser loading |

---

## 2. Critical Bugs Found & Fixed

### `ripgrepTextSearchEngine.ts:64-71` — Double-escape bug (FIXED in this session)
**Before:** `searchPattern` was both regex-escaped AND passed through `escapeRegExpCharacters`, then used with `line.indexOf(searchPattern)` — the indexOf used the escaped version, not the original literal.
**Fix:** Separated `literalPattern` (for indexOf) from `regexForRust = escapeRegExpCharacters(literalPattern)` (for Rust regex). Also replaced synchronous `searchFiles` with chunked `searchFilesChunked` for cancellation support.

---

## 3. Build-Time Issues

| Issue | Severity | Fix |
|-------|----------|-----|
| No Cargo.lock committed correctly | Low | `Cargo.lock` now committed but has tab/space mixing. Run `cargo fmt` to fix |
| tree-sitter versions 0.22 + 0.21 may conflict | **High** | Verify: `tree-sitter = "0.22"` but `tree-sitter-rust = "0.21"`. Need crate audit to ensure ABI compatibility |
| Missing `build.rs` for napi cross-compile | Medium | `build-release.yml` builds native module for host only. Need `--target aarch64-pc-windows-msvc` etc. |
| 12+ grammar crates total binary size | Medium | Each `tree-sitter-*` crate adds ~100KB compressed → ~2MB total for 20 languages. Acceptable for desktop |
| No `native/benches/` directory | Medium | `cargo bench` will fail. Need at least `fn search_large_repo`, `fn tokenize_line_100_captures`, `fn render_50_line_viewport` |
| No `native/tests/` directory | **High** | Zero Rust tests. Every export is untested. Need unit tests for tokenize scope stacking, search glob filtering, render edge cases |

---

## 4. Performance Bottlenecks Remaining

### 🔴 High priority

| Bottleneck | Location | Lines | Current approach | Rust alternative |
|-----------|----------|-------|----------------|-----------------|
| DOM line breaking | `domLineBreaksComputer.ts` | 46 | `getClientRects()` — forces synchronous layout reflow on every frame | `rustybuzz` crate (HarfBuzz) computes glyph advances without DOM. **Could eliminate #1 cause of editor jank** |
| Rendered line HTML | `viewLineRenderer.ts:_renderLine` | ~220 | JS string concat + character mapping per character | Rust `render_line_inner` would replace the character loop (lines 1051-1166). Character mapping stays in JS |
| Tree-sitter WASM parsing | `treeSitterSyntaxTokenBackend.ts:62` | ~5 | Loads WASM binary, runs parser in WebAssembly | Native Rust `tree_sitter::Parser` via `treesitter.rs`. 5-10x faster than WASM |

### 🟡 Medium priority

| Bottleneck | Location | Lines | Rust alternative |
|-----------|----------|-------|-----------------|
| File watching | `src/vs/platform/files/node/watcher/` | ~1000 | `notify` crate — lower latency, no Node event loop dependency |
| Regular expression find-in-file | `src/vs/editor/contrib/find/` | ~500 | `regex` crate on in-memory lines (already fast, low gain) |
| JSONC parsing | `jsonc.rs` | Already live | ✅ Already handled |
| Clipboard / encoding | `encoding.rs` | Already live | ✅ Already handled |

### 🟢 Low priority (not worth Rust)

| Area | Reason |
|------|--------|
| Markdown preview | Async, uses `marked` library, not on critical path |
| Source map processing | Only on error, once per crash |
| Perceptual diff for screenshots | Only in CI, not user-facing |
| Language server protocol | That's what `rust-analyzer`, `typescript-language-server` etc. are for |

---

## 5. Full App Inspection — Module-by-Module

### 5.1 Native module ecosystem (12 Rust files)

```
native/src/
├── lib.rs          # Module root — registers all 12 submodules
├── color.rs        # ✅ Live. CSS color parser (hex, rgb, named, hsl)
├── diff.rs         # ✅ Live. Myers diff + linesSimilar
├── encoding.rs     # ✅ Live. Base64/Hex
├── fuzzy.rs        # ✅ Live. Fuzzy string matching
├── hash.rs         # ✅ Live. String/number/object hash + SHA1
├── jsonc.rs        # ✅ Live. JSONC parser
├── welcome.rs      # ✅ Live. Branding HTML
├── tokenize.rs     # 🟡 Just wired. Scope stacking + bracket detection
├── search.rs       # 🟡 Partially wired. Globs, index, chunked search
├── render.rs       # ❌ Not wired in view pipeline. HTML renderer
└── treesitter.rs   # ❌ Not wired. Native tree-sitter parser + queries
```

### 5.2 CI/CD pipeline (9 workflows)

```
.github/workflows/
├── build-release.yml   # ✅ 21-step release pipeline (changelog, build, lint, test, sign, smoke, benchmark, release, notify)
├── release.yml         # ✅ Windows installer release
├── rust-native-ci.yml  # ✅ Rust build + test on push
├── benchmark.yml       # ✅ Performance benchmarks
├── pages.yml           # ✅ GitHub Pages site
├── ci-loongarch.yml    # ✅ LoongArch CI
├── ci-riscv64.yml      # ✅ RISC-V CI
├── monaco-editor.yml   # ✅ Monaco Editor CI
└── sessions-e2e.yml    # ✅ Sessions E2E tests
```

### 5.3 Test coverage gaps

| What should be tested | Test file | Exists? |
|----------------------|-----------|---------|
| Rust `create_tokens_from_captures_scoped` | `native/tests/tokenize_test.rs` | ❌ |
| Rust `search_files` with globs | `native/tests/search_test.rs` | ❌ |
| Rust `render_line_html` edge cases | `native/tests/render_test.rs` | ❌ |
| Rust `query_tree_sitter` with .scm queries | `native/tests/treesitter_test.rs` | ❌ |
| Rust `index_directory` / `search_index` | `native/tests/index_test.rs` | ❌ |
| JS native tokenize path in tree-sitter | `treeSitterTokenizationImpl.test.ts` | ❌ (file doesn't exist) |
| JS native search path (chunked, cancellation) | `ripgrepTextSearchEngine.test.ts` | 🟡 Extends existing |
| Benchmark: tokenize throughput | `native/benches/tokenize.rs` | ❌ |
| Benchmark: viewport HTML generation | `native/benches/render.rs` | ❌ |
| Test fixtures: sample source files | `native/tests/fixtures/` | ❌ |
| Golden HTML outputs | `native/tests/expected/` | ❌ |

---

## 6. What We Can Do — Prioritized Roadmap

### Do now (next session, 1-2 hours each)

| # | Task | Files | Effort | Impact |
|---|------|-------|--------|--------|
| 1 | Fix Cargo.lock whitespace + crate version audit | `Cargo.lock`, `Cargo.toml` | 30 min | Build reliability |
| 2 | Add Rust unit tests for `tokenize.rs` scope stacking | `native/tests/tokenize_test.rs` | 1 hr | Prevents regressions |
| 3 | Add `native/tests/fixtures/` with sample source files | 5-10 small files (.rs, .ts, .js, .py, .json) | 30 min | Enables all Rust testing |
| 4 | Wire `query_tree_sitter` into `treeSitterSyntaxTokenBackend.ts` as fast path before WASM | `treeSitterSyntaxTokenBackend.ts:62-67` | 1 hr | 5-10x parse speedup for supported languages |
| 5 | Add `nativeRenderLineHtmlSync` caller in minimap | `minimap.ts` line rendering | 1 hr | Minimap rendering off main thread later |

### This week

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 6 | `rustybuzz` line breaking — replace DOM `getClientRects` | 3-4 hrs | Eliminates editor jank on scroll |
| 7 | File index lifecycle in JS — call `indexDirectory` on workspace open, `searchIndex` on query | 2 hrs | Faster repeated searches in large projects |
| 8 | Add `native/benches/` with tokenize + render benchmarks | 2 hrs | Data-driven optimization decisions |

### This sprint

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 9 | Replace inner character loop in `_renderLine` with Rust (lines 1051-1166) | 4-6 hrs | Viewport rendering 2-3x faster |
| 10 | Cross-compile `.node` for arm64/linux/darwin in CI | 2-3 hrs | Ship Rust acceleration to all platforms |
| 11 | Add `native/tests/` directory with full test suite (tokenize, search, render) | 4 hrs | Production confidence |

### This quarter

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 12 | Ship 12+ tree-sitter grammar crates, compile into `.node` binary | 8-10 hrs | Eliminate WASM entirely for 20 languages |
| 13 | Replace `_finishRendering` loop with single Rust batch call | 3-4 hrs | Eliminate N round-trips per frame |
| 14 | Rust file watching via `notify` crate | 4-6 hrs | Faster file tree refresh, instant search reindex |
| 15 | Full viewport rendering pipeline in Rust (HTML + CharacterMapping) | 8-12 hrs | Complete render offload |

---

## 7. Architecture Decisions Log

| Decision | Date | Rationale |
|----------|------|-----------|
| Rust `create_tokens_from_captures_scoped` returns `ScopeTokenResult` with JSON-encoded scopes | Jul 2026 | Avoids napi object per token + dynamic array serialization issues. JS `JSON.parse` is fast enough for <50 tokens/line |
| Chunked search for cancellation instead of `AtomicBool` | Jul 2026 | `Option<&napi::JsBoolean>` is not usable from Rust synchronously. Chunked approach gives JS control at natural breakpoints |
| Keep `CharacterMapping` in JS, Rust only does HTML | Jul 2026 | Character mapping complexity (tab expansion, RTL reordering, ligature offsets) would require full port of 220-line `_renderLine`. Starting with inner character loop is safer |
| Feature-gated rollout via native module availability check | Jul 2026 | `nativeModuleSync` is null when `.node` binary is missing (dev, web, remote). All Rust paths degrade gracefully to JS |
