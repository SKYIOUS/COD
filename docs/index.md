---
title: COD — Rust-Accelerated Code Editor
description: COD is a fork of VS Code — stripped of Copilot, Microsoft sign-in, and telemetry, accelerated with native Rust modules.
---

# COD

**COD** is a fork of Visual Studio Code — stripped of Copilot, Microsoft sign-in, and telemetry, and accelerated with native Rust modules for the hottest code paths.

---

## Performance

| Metric | COD | VS Code |
|---|---|---|
| Startup to window | **822 ms** | 1,644 ms |
| Idle RAM | **30 MB** | 98 MB |
| Install size | **268 MB** | 385 MB |
| Fuzzy search | **15.2x** vs TS | — |
| String hashing | **4.2x** vs TS | — |
| Lines similarity | **5.7–10x** vs TS | — |
| Diff computation | **2.0x** vs TS | — |

## Quick Start

```bash
git clone https://github.com/SKYIOUS/COD
cd COD
npm install
npm run compile:native
npm run compile
.\scripts\code.bat
```

## What's Removed

- **GitHub Copilot** — all UI, commands, chat, status bar, agent
- **Microsoft sign-in** — no auth prompts, no account hub
- **Telemetry** — no crash reporting, no usage data
- **Copilot chat** — no chat tips, no onboarding tour
- **Debug extension prompts** — no nag dialogs
- **Welcome page onboarding** — stripped to essentials

## Rust Native Modules

COD ships a Rust crate (`native/`) using napi-rs that accelerates 4 hot paths:

| Module | Functions | Speedup |
|---|---|---|
| **fuzzy** | `fuzzyScore`, `scoreFuzzy`, `prepareQuery` | up to **15x** |
| **diff** | `myersDiff`, `lcsDiff`, `linesSimilar` | up to **10x** |
| **hash** | `stringSha1`, `stringHash`, `numberHash`, `objectHash` | up to **19x** |

All native calls have sync + async JS fallbacks — Rust is optional, zero breakage risk.

## Benchmarks

Full comparison against VS Code and Zed: [COMPARISON.md](https://github.com/SKYIOUS/COD/blob/main/COMPARISON.md)

### Run your own

```bash
# Rust vs TypeScript microbenchmarks
node benchmark-rust-ts.cjs

# Editor-level benchmarks (startup, memory, diff)
powershell -File benchmark-editors.ps1
```

## Build from Source

```bash
npm run compile          # Full production build
npm run compile:native   # Rust only
npm run compile:ts       # TypeScript only (faster iteration)
```

Output: `.build/electron/COD.exe`

## Features

COD retains everything that makes VS Code great:
- IntelliSense, debugging, Git integration
- Extension ecosystem (thousands available)
- Remote development (WSL, Containers, SSH)
- Terminal, tasks, snippets, themes
- Multi-root workspaces, Settings Sync

The only difference: **no Microsoft telemetry, no Copilot, and faster internals.**

## License

Licensed under the [MIT](https://github.com/SKYIOUS/COD/blob/main/LICENSE.txt) license.

Original [VS Code](https://github.com/microsoft/vscode) by Microsoft Corporation.
