# COD Editor Benchmark

[![CI](https://github.com/nandagowda/COD/actions/workflows/pages.yml/badge.svg)](https://github.com/nandagowda/COD/actions)

Startup time and memory comparison between **COD**, **VS Code**, and **Zed**.

## Methodology

- **pywinauto mode:** launches editor, times until main window appears (win32gui EnumWindows), measures private/working-set memory across the full process tree (3 samples, median). 5 runs + 1 warmup per editor.
- **hyperfine mode:** wrapper script launches editor, detects window, kills process. Measures wall-clock of the full cycle. 15 runs + 2 warmup per editor.
- Editors launched with `--disable-extensions --skip-release-notes` (COD / VS Code) against a minimal workspace (single `readme.md`).
- Fresh `--user-data-dir` per run; processes killed and cleaned between runs.

## Results

### pywinauto — window-to-window startup + memory

| Editor | Window (ms) | Private (MB) | WorkingSet (MB) |
|--------|------------|-------------|----------------|
| COD    | 751 +- 103 | 442.4 +- 106.3 | 665.6 +- 177.3 |
| VS Code | 909 +- 136 | 450.3 +- 134.1 | 698.4 +- 190.0 |
| Zed    | 621 +- 71  | 194.6 +- 2.1   | 165.3 +- 2.7   |

**COD vs VS Code:** startup **1.21x faster**, memory **1.02x less**  
**COD vs Zed:**     startup **0.83x slower**, memory **0.44x** (Zed uses less than half the memory)

### hyperfine — CLI wrapper cycle (launch → detect → kill)

| Editor | Mean (ms) | Min | Max |
|--------|----------|-----|-----|
| COD    | 1060     | 824 | 1470 |
| VS Code | 1044    | 901 | 1345 |
| Zed    | 966      | 874 | 1092 |

> hyperfine values include Python interpreter startup and process cleanup overhead; use pywinauto for true window-to-window times.

## Run It Yourself

```bash
# pywinauto mode (5 runs + 1 warmup)
python benchmark_editors.py

# hyperfine mode (10 runs + 2 warmup)
python benchmark_editors.py --hyperfine

# hyperfine with custom runs
python benchmark_editors.py --hyperfine --hf-runs 15 --hf-warmup 3

# specific editors only
python benchmark_editors.py -e COD,Zed

# dry-run (print hyperfine commands without executing)
python benchmark_editors.py --hyperfine --dry-run
```
