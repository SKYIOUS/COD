# COD — Rust-Accelerated Code Editor

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE.txt)
[![Benchmarks](COMPARISON.md)](COMPARISON.md)

**COD** is a fork of Visual Studio Code — stripped of Copilot, Microsoft sign-in, and telemetry, and accelerated with native Rust modules for the hottest code paths.

```
Startup:  822 ms  (2x faster than VS Code)
Idle RAM: 30 MB  (3.3x less than VS Code)
Diff:     19x faster with Rust native SHA-1
Search:   7.3x faster with Rust fuzzy scoring
```

---

## Why COD?

| VS Code has | COD has |
|---|---|
| Microsoft sign-in, telemetry | Zero — all removed |
| Copilot baked into UI | Removed entirely |
| Copilot chat, agent chat | Removed entirely |
| ~98 MB idle RAM | ~30 MB idle RAM |
| ~1.6 s startup | ~0.8 s startup |
| Pure TypeScript bottlenecks | Rust native acceleration |

See the [full benchmark comparison](COMPARISON.md) for COD vs VS Code vs Zed.

---

## Rust Native Modules

COD's Rust crate (`native/`) accelerates 4 hot paths with 11 exported functions:

| Module | Functions | Speedup vs TS |
|---|---|---|
| **fuzzy** | `fuzzyScore`, `scoreFuzzy`, `prepareQuery` | up to 15x |
| **diff** | `myersDiff`, `lcsDiff`, `linesSimilar` | up to 10x |
| **hash** | `stringSha1`, `stringHash`, `numberHash`, `objectHash` | up to 19x |

All native calls have sync + async JS fallbacks. Zero risk — Rust is optional.

---

## Quick Start

```bash
# Prerequisites: Node.js 24+, Rust nightly, Python 3
git clone https://github.com/SKYIOUS/COD
cd cod

# Install dependencies
npm install

# Build the Rust native module
npm run compile:native

# Build TypeScript
npm run compile

# Launch COD
.\scripts\code.bat
```

---

## Build from Source

```bash
# Full production build (creates .build/electron/COD.exe)
npm run compile

# Rust only (needs recompile after native/ changes)
npm run compile:native

# TypeScript only (faster iteration)
npm run compile:ts
```

---

## Benchmarks

Run your own benchmarks:

```bash
# Rust vs TypeScript microbenchmarks (all 11 functions)
node benchmark-rust-ts.cjs

# Editor-level benchmarks (startup, memory, diff)
powershell -File benchmark-editors.ps1
```

See the full results in [COMPARISON.md](COMPARISON.md).

---

## What's Removed

- GitHub Copilot (all UI, commands, chat, status bar)
- Microsoft sign-in / auth
- Telemetry and crash reporting
- Copilot chat, agent chat, chat tips
- Debug extension download prompts
- Welcome page onboarding tour

---

## License

Licensed under the [MIT](LICENSE.txt) license.

Website: [skyious.github.io/cod](https://skyious.github.io/COD)
Original [VS Code](https://github.com/microsoft/vscode) by Microsoft Corporation.
