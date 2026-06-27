---
title: COD — Rust-Accelerated Code Editor
---

# COD

**COD** is a fork of Visual Studio Code — stripped of Copilot, Microsoft sign-in, and telemetry, and accelerated with native Rust modules for the hottest code paths.

```
Startup:  822 ms  (2x faster than VS Code)
Idle RAM: 30 MB  (3.3x less than VS Code)
Diff:     19x faster with Rust native hashing
Search:   7.3x faster with Rust fuzzy scoring
```

## Quick Start

```bash
git clone https://github.com/SKYIOUS/COD
cd COD
npm install
npm run compile:native
npm run compile
.\scripts\code.bat
```

## Benchmarks

See the full [benchmark comparison](https://github.com/SKYIOUS/COD/blob/main/COMPARISON.md).

## License

Licensed under the [MIT](https://github.com/SKYIOUS/COD/blob/main/LICENSE.txt) license.
