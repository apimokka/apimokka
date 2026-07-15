# apimokka — designer brief

This document hands off the apimokka GUI mockup to a visual designer.
The current implementation works as a functional spike but the visual
result is dense and inconsistent. Use this brief to produce a clean
visual design (Figma, Sketch, etc.); the existing code can then be
re-skinned against it.

---

## 1. Product summary

**apimokka** is a desktop GUI wrapper for **apimock-rs**, a Rust HTTP
mock server used by backend / frontend / QA engineers to simulate APIs
during development. Without apimokka, users edit TOML files by hand
and watch logs in a terminal.

The GUI's purpose: make rule editing visual, make traffic observable
in real time, and remove the "edit TOML → restart → check logs" loop.

**One sentence:** A native desktop app where you author HTTP mock rules
in a visual editor and watch live request traffic match (or miss) those
rules.

---

## 2. Who uses it

- **Backend developers** stubbing third-party APIs they depend on
  (payment gateways, social logins, partner endpoints)
- **Frontend developers** mocking a backend that doesn't exist yet
- **QA engineers** forcing edge-case responses (500s, timeouts,
  malformed payloads) to verify their app handles them
- **API designers** prototyping endpoints before implementation

Common context: the user runs apimokka on their dev laptop alongside
their actual application. They alt-tab between the two constantly.

---

## 3. Core user workflows

In rough frequency order — the design should make #1 and #2 effortless;
the others can be slightly buried.

### Workflow 1 — Tweak an existing rule and verify (most common)

1. User has the app open; the rule they want to edit is already
   selected from a previous session
2. They change something — e.g. switch a response from 200 to 500,
   or change a URL match operator
3. They trigger a request from their app (or curl in a terminal)
4. They want to immediately see whether the request matched the rule,
   and which response was returned
5. If it didn't match, they need to know *why* — what condition failed

**Design implication:** the rule editor and the live trace need to be
visible at the same time, not in different tabs.

### Workflow 2 — Add a new endpoint

1. User decides they need to mock a new path (e.g. `/api/checkout`)
2. They pick a rule set to put it in (or create one)
3. They author the rule: URL path, method, optional header/body match
   conditions, and the response (status, body, optional delay)
4. They test it works (workflow 1)

**Design implication:** the rule editor must scaffold a sensible
starting state — empty fields with placeholders, not blank panels.

### Workflow 3 — Debug a miss

1. User sees a "miss" event in the trace
2. They click it to see what the request was
3. They suspect a particular rule should have matched and need to
   compare the request against the rule's conditions
4. They want to "replay" the request as a test input against a
   specific rule to verify

**Design implication:** trace events need a Replay action, and the
match-detail view needs to surface *which* rule was the closest match
and *why* it didn't match.

### Workflow 4 — Switch workspaces

A workspace is one folder containing one set of rule files. Users
typically have several (one per project). Switching workspaces should
be one click from anywhere.

### Workflow 5 — First-time setup

New user, no workspaces yet. Wizard creates a workspace with sensible
defaults (host, port, starter content). Should be skippable / collapse
to one click for the impatient.

### Workflow 6 — Settings (rare)

Port, TLS, log level, trace queue size, server strategy mode. Most
users touch these once during setup and never again.

---

## 4. Domain model (vocabulary the design must surface clearly)

| Term | Meaning |
|---|---|
| **Workspace** | One folder containing all configuration for a single mock-API project. Contains rule sets, fallback files, and middleware scripts. |
| **Rule set** | One TOML file containing 1..N rules. Files appear as siblings in the workspace. |
| **Rule** | A `WHEN → RESPOND` pair. `WHEN` = match conditions on the HTTP request. `RESPOND` = the response to return. |
| **Match conditions** | URL path (with operator: equals, starts-with, contains, …), HTTP method, header conditions, body conditions (with JSON dotted-path) |
| **Respond** | Either inline text (status + body string) OR serve a file from disk. Plus optional response delay. |
| **Fallback file** | A file in the workspace that's served when no rule matches a path. The file's name maps to a URL. |
| **Middleware script** | A Rhai script that can rewrite requests/responses (out of scope for visual editing; read-only display) |
| **Strategy** | How the engine picks among multiple matching rules in one rule set: `single` (first match), `weighted_random`, `priority` |
| **Trace event** | A record of one HTTP request the server received, with outcome (matched/fallback/miss/error), request preview, response preview, and duration |
| **Outcome** | `matched` (a rule won), `fallback` (no rule matched, fallback file served), `miss` (no rule, no fallback), `error` (script or runtime error) |
| **Server state** | `stopped`, `starting`, `running`, `reload-pending`, `restart-required`, `error` |
| **Save state** | Workspace can have unsaved edits. Some settings changes require server reload; some require restart. |

---

## 5. Information architecture

### Outer states (one of)

- **Welcome** — first-launch / no workspace open
- **Dashboard** — workspace picker (lists recent workspaces with
  pin/last-opened)
- **Wizard** — new-workspace creation form
- **Workspace** — the main app once a workspace is loaded

### Workspace shell (persistent chrome around the body)

- **Top bar**: workspace identity, status indicators, action buttons,
  view controls, theme/locale toggles
- **Left rail**: 4 destinations — Routes, Trace, Scripts, Settings
- **Body**: per-tab screen content
- **Bottom drawer (optional)**: validation issues / save diff
- **Right column on Routes (optional)**: live trace strip OR rule inspector

### Tab contents

| Tab | Content |
|---|---|
| **Routes** | Left sidebar (rule-set tree + fallback files + middleware scripts) + centre (rule editor for selected rule) + right (trace strip or inspector) |
| **Trace** | Full-screen event stream with filter + match-detail panel on selection |
| **Scripts** | Read-only viewer for middleware Rhai scripts |
| **Settings** | Sectioned form: General, Server, TLS, Logs, Trace, Strategy |

### Dialogs / overlays (one at a time, Esc dismisses)

- **Confirm dialog** — destructive-action confirmation
- **Command palette** — keyboard-first action launcher
- **Test rule** — dry-run match against a synthetic request
- **Dotted-path assistant** — JSON tree picker for body conditions
- **Workspace menu** — dropdown from the top bar (header menu slot)

---

## 6. Screen-by-screen content (designer reference)

### Welcome (S-00)

**Purpose:** First impression. New user with no workspace.

**Content:**
- Hero: app name, one-line tagline ("Visual HTTP mock authoring")
- Two primary actions: Open workspace, Create new workspace
- (When recent workspaces exist) Recents grid: name, path, last-opened
- (Educational) Three-layer diagram explaining the request-handling
  order: middleware scripts → rule sets → fallback files

### Dashboard (S-01)

**Purpose:** Workspace picker.

**Content:**
- Search bar
- List of workspaces: name, path, last-opened, pin toggle, open button
- "Create new workspace" CTA

### Wizard (S-02)

**Purpose:** Create a new workspace.

**Content (single page):**
- Required: workspace name, parent folder
- Server (collapsed): IP, port, TLS toggle
- Starter content (collapsed): empty / basic REST + error samples + delay sample
- Trace (collapsed): enable + queue size
- Action bar: Cancel, Create

### Routes (S-05 to S-08) — the most-used screen

**Three columns:**

**Left sidebar** (~260 px):
- Header: "Rule sets" (caption)
- Per rule set: filename, dirty marker if unsaved, expandable rules below
- Each rule shown as a one-line summary: e.g. `GET /api/orders → 200`
  with validation/match status glyph on the right
- Section divider, then: Fallback files (name + URL hint), Middleware
  scripts (path)
- Bottom: "Add rule set" button

**Centre panel** (fills remaining):
- When no rule selected: empty-state hint
- When rule selected: two-column rule editor
  - WHEN column: URL path card (with operator), method buttons,
    headers card (rows: name / op / value), body card (rows: dotted
    path / op / value)
  - Arrow divider
  - RESPOND column: mode tabs (Inline text / Serve file), body
    editor, status + delay row, "Test rule" action button

**Right column** (~260 px, toggleable):
- Either: Live trace strip (last 15 events, two-line rows: method+path
  on top, time+duration below in muted text; each row has a Replay
  icon button), with pause/resume/clear at bottom
- Or: Rule inspector (validation issues, strategy-specific fields like
  weight or priority, quick actions: duplicate / move up / move down /
  delete)

### Trace (S-11) + Match detail (S-12)

**Full-screen event list.**

- Filter bar: method, outcome, path-contains
- Event rows: outcome glyph, method, path, time, duration, rule
  matched (if any)
- Selecting an event opens the match-detail panel on the right with:
  request method/path/headers/body, response status/headers/body,
  matched rule (if any) with link to its rule-set file, "Replay as
  test input" button
- Footer: pause/resume, clear

### Scripts (S-14)

**Purpose:** Inspect middleware Rhai scripts. Read-only.

- Left list of script paths
- Centre: code viewer (syntax-highlighted Rhai, monospace, line numbers)
- No edit affordances in v1

### Settings (S-13)

**Sectioned form.** Each section is a card; only restart-required
changes need explicit apply.

- General: workspace name, base path
- Server: host, port, TLS (key + cert), reload watcher toggle
- Logs: file path, level
- Trace: enable, transport (UDS / Unix socket), queue size
- Strategy: single / weighted_random / priority

### Bottom drawer

**Optional sheet, ~30% of screen height, sliding up from the bottom.**

Two modes:
- **Validation**: grouped issues by file, severity glyph, click-to-jump
- **Save diff**: list of files about to be written, with diff-style
  preview, "Save all" / "Discard" buttons

### Top bar (persistent)

Left → right:
- Workspace identity (clickable, opens workspace dropdown):
  app name · workspace name ▼
- Server status chip (glyph + label)
- Saved/Unsaved chip
- (spacer, flex)
- Action buttons: Save, Reload, Restart, Start/Stop server
- View controls: trace-strip toggle (∿), theme toggle (☾/☀),
  command palette (⌘K)
- Locale dropdown (EN / JA)

### Workspace menu (header dropdown)

Triggered by top-bar identity click. Snora `header_menu` slot.
- Current workspace label + path
- Recent workspaces list
- "Open workspace…" / "New workspace…"

### Command palette

Dialog, ~520 px wide.
- Search input
- Filtered command list with shortcut hints (e.g. "⌘K toggle",
  "Esc close" in header)
- ~15 commands: save, server start/stop/reload/restart, navigate to
  tab, toggle dark mode, toggle trace strip, open validation, open/
  create workspace

### Confirm dialog

Dialog, ~420 px wide.
- Title: "Confirm"
- Description sentence
- Cancel (ghost) + Proceed (danger) buttons

### Test rule

Dialog, ~520 px wide.
- Method picker (segmented buttons)
- URL path, headers, body inputs (request to dry-run)
- "Run test" — result banner: `✓ Matched` / `◯ No match` /
  `! Error`
- Pre-fills from current rule or from a trace event ("Replay")

### Dotted-path assistant

Dialog. Helps build body-condition paths.
- Paste / load a sample JSON
- JSON tree view; clicking a node inserts its dotted path into the
  current body-condition field

---

## 7. Hard constraints (do not break)

### Accessibility — ABDD

**Every status indicator must carry a glyph AND a text label.**
Colour is supplementary, never the only signal.

Current glyph vocabulary (preserve or replace consistently):

| Concept | Glyph |
|---|---|
| Server running | ● |
| Server stopped | ■ |
| Reload pending | ↻ |
| Restart required | ⏻ |
| Error | ! |
| Saved | ✓ |
| Dirty / unsaved | ● |
| Match | ✓ |
| Fallback | ↩ |
| Miss | ◯ |
| Validation error | ✕ |
| Validation warning | ⚠ |
| Info | ℹ |

### Internationalisation

- EN + JA shipped today; JA strings are typically 10–30 % longer than
  EN in visual width
- All user-visible strings come from an i18n table; designers should
  size widgets for the longer JA strings, not EN

### Keyboard

- Esc closes the topmost overlay
- ⌘K (or Ctrl+K) toggles the command palette
- All primary workflows must be reachable without a pointer

### Themes

- Light and Dark mode both ship in v1
- Status colours (severity, outcome) must work on both backgrounds

### Technology limits — iced 0.14 + snora 0.8

The renderer is **iced 0.14**. A designer should know what it can and
cannot do natively:

**Can do well:**
- Cards, panels, rounded corners, drop shadows
- Custom fonts (load TTF/OTF), arbitrary text sizes
- Text inputs, buttons, checkboxes, radios, pick-lists, sliders
- Scrollable regions, padding, alignment, flex-like rows/columns
- Inline SVG (limited), drawn shapes via canvas
- Light + Dark theme with extended palette (primary/secondary/danger/
  success/warning families)
- Container shadows, border radius, alpha-blended backgrounds

**Cannot easily do:**
- True freeform vector graphics inline alongside widgets
- CSS-level transitions / smooth animations (iced has them but they're
  a separate API and the team hasn't wired them)
- Backdrop blur, complex gradients
- Web-style hover popovers (snora's `header_menu` is the dropdown
  pattern available)

**Snora 0.8** provides the outer shell pattern (`AppLayout`): header,
sidebar, body, bottom sheet, dialog, header_menu. The dialog and sheet
layers are managed by snora; the designer should think of them as
"native to the layout" rather than overlays we paint ourselves.

---

## 8. Honest assessment of the current implementation

What works (probably worth preserving):
- The four-tab top-level IA (Routes / Trace / Scripts / Settings)
- Routes-tab three-column layout with optional live-trace right strip
- Two-column rule editor (WHEN | → | RESPOND)
- Workspace switcher in the top-bar dropdown
- Auto-save for rule edits (only restart-class settings need explicit
  action)
- Esc / ⌘K keyboard contract

What's not working visually:
- Density too high: every form, every list, every chip feels cramped
- Type hierarchy too flat: six size tokens but in practice everything
  reads at the same weight because iced renders body text uniformly
- No real iconography — Unicode glyphs are a placeholder
- Spacing tokens were added but applied unevenly across the 24 source
  files; many remaining magic numbers
- The status-chip vocabulary works for accessibility but looks busy
  when 4+ chips share a row in the top bar
- No illustration / personality on empty states; the Welcome screen
  reads as functional rather than welcoming
- Card shadows are subtle but the surrounding panels and dividers
  still fragment content
- The current colour palette comes straight from iced's default
  light/dark themes; no brand colour, no accent treatment

---

## 9. What we'd like from the designer

1. A complete Figma (or equivalent) file with:
   - Light + Dark mode variants for every screen
   - All dialog overlays
   - States: empty, loaded, error, hover-where-relevant
   - Component library: button variants, card variants, chip variants,
     input variants
2. A defined type scale (font family, weights, sizes) targeting
   iced 0.14's text rendering
3. A defined colour palette with light/dark mappings — including
   semantic mappings (severity, outcome, server state)
4. Iconography set (glyph replacements) — SVG, sized for inline use
5. Spacing and radius scale (we can carry these as design tokens)
6. Empty-state illustrations or treatment for: Welcome (no recents),
   Routes (no rule selected), Trace (no events yet)

---

## 10. Reference files

| File in the source tree | Purpose |
|---|---|
| `docs/src/ux-redesign.md` | Workflow-centred IA rationale (v0.3) |
| `rfcs/done/MK-018-*.md` | UX redesign decisions |
| `rfcs/done/MK-019-*.md` | Design tokens introduced in v0.4 |
| `rfcs/done/MK-020-*.md` | Dark theme + auto-save + workspace switcher in v0.5 |
| `rfcs/done/MK-015-*.md` | Accessibility / keyboard / ABDD policy |
| `crates/apimokka-i18n/src/keys.rs` | Every user-visible string key |
| `crates/apimokka-model/src/mock.rs` | Sample data the screens render |

The full source — all 45 Rust files, 23 RFCs, EN+JA i18n tables — is
in the v0.5 archive. A designer can run `cargo run` to see the current
state, but should treat it as wireframe-level reference, not a visual
target.
