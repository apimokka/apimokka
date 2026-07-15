# apimokka

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org)

**A desktop GUI mockup for [apimock-rs](https://github.com/nabbisen/apimock-rs) — the Rhai-scriptable HTTP mock server.**

---

## Overview

apimokka is a UI/UX prototype demonstrating the full screen flow of a desktop
GUI for apimock-rs. It covers workspace management, visual rule editing,
live-request tracing, server settings, and audience-mode switching — built on
[iced 0.14](https://github.com/iced-rs/iced) and
[snora 0.25](https://crates.io/crates/snora) with the Snora Design system.

This is a **mockup only** — no file I/O, no live server connection. All workspace
data is mock data seeded at startup. The goal is to validate screen flows and UX
patterns before writing production integration code.

---

## Why / When

Use this mockup to:
- Evaluate the proposed screen architecture before committing to production code
- Walk through the Guided / Expert audience-mode UX with stakeholders
- Review the visual rule builder, trace screen, and workspace wizard with real users
- Identify missing i18n keys or accessibility gaps early
- Exercise the undo/redo command log and first-launch flow

---

## Quick Start

### Prerequisites

- Rust 1.91 or later
- A Linux desktop with Wayland or X11 (iced 0.14 requirement)

### Build and run

```sh
git clone <this-repo>
cd apimokka
cargo run
```

The app opens to the **mode picker** (first-launch flow). Choose Guided or Expert,
then navigate to the Welcome screen. Click **Open workspace** → **shop-api-mock**
to reach the full Routes workbench.

---

## Features / Design Notes

- **First-launch flow** — Mode picker → Welcome → Dashboard or Wizard → Workspace. On the Welcome screen, "Create workspace" opens a three-starter wizard (Minimal / Shop API example / Empty); the Minimal default generates a single `GET /health → 200 OK` rule.
- **Audience modes** — Guided mode surfaces inline hints and collapses advanced controls (headers, body conditions, strategy) behind expandable "More" rows; Expert mode shows everything directly. Switching modes is reversible in Settings.
- **Snapshot-apply loop simulation** — mirrors the apimock-rs `Workspace::apply(EditCommand)` contract without real file I/O (see [MK-035](./rfcs/done/MK-035-state-models.md))
- **Undo / redo** — typed command log (⌘Z / ⌘⇧Z); covers delete, add, move, and URL-path edits
- **MK-038 fallback file lifecycle** — two-buffer (saved baseline + draft), explicit Save, confirmed Revert, live JSON validity badge
- **Snora Design tokens** — built on snora 0.25's design system: WCAG-AA contrast-tested color presets with four themes (Light, Dark, **High Contrast Light, High Contrast Dark**). High-contrast modes add visible card/panel borders for low-vision users. Selectable in Settings → Appearance.
- **Non-colour status matrix** — every status indicator carries both a glyph and a text label
- **i18n from day one** — English and Japanese translations compiled into the binary; locale switch in Settings
- **snora AppLayout shell** — header / left sidebar / screen body / bottom drawer; command palette (all 17 commands wired)
- **Visual rule builder** — URL path + operator, method segment controls, header and body condition rows with full operator coverage, respond editor (inline text / file path / status / delay)
- **Live Trace panel** — filterable event list, outcome-aware match detail (Matched → jump to rule; Fallback → jump to file; Miss → create rule CTA; Error → kind + message), dropped-event warning
- **Bottom drawer** — Validation panel grouped by rule set with jump-to-rule navigation; Save-diff panel with rule summaries per dirty file
- **Test rule dialog** — evaluates method, URL path, all header conditions (9 ops), all body conditions (19 ops) including dotted-path JSON traversal

---

## More Detail

- [Full documentation](./docs/src/README.md) *(mdbook source)*
- [RFC index](./rfcs/README.md)
- [Changelog](./CHANGELOG.md)

---

## License

Apache-2.0. See [LICENSE](./LICENSE) and [NOTICE](./NOTICE).

Author: **nabbisen**
