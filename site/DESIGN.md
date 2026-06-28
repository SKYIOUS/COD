# COD — UI/UX Design Identity

## Brand Essence

> **Unburdened. Accelerated. Yours.**

COD is VS Code freed from Microsoft's baggage (Copilot, sign-in, telemetry) and supercharged with Rust-native acceleration. The design communicates speed, freedom, and precision — a tool that gets out of your way and stays fast.

---

## Visual Identity

### Color System

| Token | Hex | Usage |
|---|---|---|
| `--bg-deep` | `#07070d` | Deepest background — main canvas |
| `--bg-surface` | `#0b0b18` | Sidebars, panels, secondary surfaces |
| `--bg-card` | `#111128` | Cards, dropdowns, elevated surfaces |
| `--bg-elevated` | `#181840` | Modals, hover states, tooltips |
| `--cyan` | `#00d4ff` | **Primary accent** — interactive elements, highlights |
| `--coral` | `#ff5e5b` | **Warm accent** — deletions, warnings, energy |
| `--purple` | `#a277ff` | **Tertiary** — special states, native module branding |
| `--text-primary` | `#e4e4f0` | Primary text — cool white |
| `--text-secondary` | `#8888bb` | Secondary text — descriptions, labels |
| `--text-muted` | `#555580` | Disabled, placeholders, metadata |

The palette is intentionally **dark and moody** with a dominant cyan accent that pops against the deep indigo-obsidian background. Color is used sparingly to direct attention — cyan for **what matters**, coral for **what's gone**, purple for **what's new** (Rust modules).

### Typography

| Role | Font | Weight | Rationale |
|---|---|---|---|
| Display / Headings | **Bricolage Grotesque** | 700–800 | Geometric, distinctive, modern — avoids generic sans-serif |
| Body / UI | **DM Sans** | 400–600 | Clean, highly readable at small sizes, warm character |
| Code / Data | **JetBrains Mono** | 400–600 | Developer trust — purpose-built for code, ligatures optional |

### Logo Mark

The logo `<COD />` uses angle brackets to nod to HTML/JSX syntax — instantly readable as developer tooling. The cyan color ties it to the accent system. In the editor, the logo appears in:
- Window title bar
- About dialog
- Command palette header

---

## Editor UX Philosophy

### Core Principles

1. **Zero Friction by Default**
   - No onboarding tour — first launch shows a clean welcome page with recent files + quick start
   - No sign-in prompts — everything works offline, no account required
   - No telemetry dialog — telemetry doesn't exist in the codebase
   - No Copilot commands clogging the command palette — all removed

2. **Performance is a Feature You Feel**
   - 822ms startup isn't just a benchmark — it means COD feels instant
   - 30MB idle RAM means COD runs alongside Docker, browsers, terminals
   - Rust acceleration has zero perceptible latency — fuzzy search responds before you finish typing
   - **Visual cue**: a subtle cyan glow in the status bar when a Rust-native path is active

3. **Keyboard-First, Clutter-Free**
   - Defaults optimized for power users: reduced padding, compact layout
   - Activity bar visible but minimal — no badges, no Copilot icon
   - Status bar stripped to essentials — language mode, Git branch, line/col — no "sign in" button, no telemetry gear
   - Command palette is the primary navigation — every action discoverable via `Ctrl+Shift+P`

4. **Unmistakable Identity**
   - Cyan accent color distinguishes COD from VS Code's blue — a subtle but deliberate branding signal
   - Product icon: a stylized "C" in a square, or a bracket pair `[ ]` — no puzzle piece, no Microsoft associations
   - Window title reads "COD" not "Visual Studio Code"
   - Splash screen: clean COD logo + version + "Rust-Accelerated" tagline

### What Stays

COD retains everything that makes VS Code great:
- IntelliSense, debugging, Git integration
- Extension ecosystem
- Remote development (WSL, Containers, SSH)
- Terminal, tasks, snippets, themes
- Multi-root workspaces

### What Changes

| Element | VS Code | COD |
|---|---|---|
| Activity Bar | Copilot icon, accounts icon | No Copilot, no accounts |
| Status Bar | Sign-in, telemetry, Copilot | Clean — lang, branch, position |
| Welcome Page | Onboarding tour, Copilot tips | Recent files, quick start |
| Command Palette | Copilot commands mixed in | De-cluttered |
| Settings | Telemetry, sign-in, Copilot toggles | Removed |
| Title Bar | "Visual Studio Code" | "COD" |
| Splash / About | Microsoft branding | COD branding |
| Default Theme | Dark+ (blue accents) | COD Dark (cyan accents) |

### Theme: COD Dark (Default)

A custom dark theme that ships with the editor:

- **Editor background**: `#0d0d1a`
- **Line numbers**: `#555580`
- **Cursor**: `#00d4ff` (cyan beam)
- **Selection**: `rgba(0,212,255,0.25)`
- **Comments**: `#555580` italic
- **Keywords**: `#ff5e5b` (coral)
- **Strings**: `#a277ff` (purple)
- **Functions**: `#00d4ff` (cyan)
- **Variables**: `#e4e4f0`
- **Constants**: `#f1fa8c`
- **Operators**: `#ff79c6`
- **Brackets**: `#8be9fd`
- **Git added**: `#50fa7b`
- **Git deleted**: `#ff5555`
- **Active line highlight**: `rgba(0,212,255,0.04)`
- **Find match highlight**: `rgba(0,212,255,0.3)`

### UX Micro-Interactions

- **Command Palette**: Opens with a brief scale+opacity animation (100ms) — feels responsive, not jarring
- **Tab Close**: Fade+shrink (80ms) — prevents visual tearing
- **Search Results**: Staggered reveal — results appear as they're scored (Rust fuzzy makes this feel instant)
- **Status Bar**: Rust-native paths flash a cyan dot briefly when activated
- **Notifications**: Slide in from top-right with a subtle cyan border
- **Hover tooltips**: Appear with a 150ms delay — fast enough to feel responsive, slow enough to not flicker

### Rust-Native Visual Cue

A small indicator in the status bar (left side) shows when Rust acceleration is active:

```
λ Rust     ⟶    fuzzy, diff, hash active
```

When no Rust module is loaded (fallback to TS), the indicator dims. This is intentionally subtle — performance shouldn't be a dashboard, but power users appreciate knowing the engine is running.

---

## Design Files

The static marketing site lives in `site/`:
- `site/index.html` — Landing page
- `site/style.css` — Design system + styles
- `site/script.js` — Interaction (scroll reveals, animations)
- `site/DESIGN.md` — This document
