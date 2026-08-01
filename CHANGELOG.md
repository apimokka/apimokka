## [Unreleased]

### Changed

- Release-decision checks now run through one repository-owned stable/MSRV,
  strict-Clippy, audit, and governance gate. The full workspace is warning-free
  under denied Clippy warnings. The fallback JSON editor remains editable as
  plain text without iced's optional syntax-highlighter dependency chain, and
  the retained Wayland build path now resolves patched `quick-xml` 0.41.0
  (RFC MK-054).
- Configuration editing now uses the RFC MK-053 `WorkspacePort` mapping
  boundary instead of mutating render snapshots directly. Typed atomic edits,
  stable condition identities, canonical/render snapshot correlation,
  semantic undo/redo, runtime request correlation, and historical Global Save
  reporting are covered by the in-memory contract suite. Production
  filesystem, process, watcher, merge, and trace-transport integration remains
  explicitly deferred.
- Test Rule now evaluates supported conditions through the lockfile-resolved
  apimock-routing 5.10.0 matcher primitives. Unsupported configured methods and
  operators produce a distinct “Unable to verify” outcome; malformed input
  produces Error; neither can be reported as Matched or No match. Exact i64,
  engine wildcard, dotted-path, typed JSON, numeric, array, and presence
  semantics have executable conformance coverage. The README and UI disclose
  the supported matrix and link the detailed limitations page. A repository
  guard rejects unreviewed matcher/parser version, source, checksum, or feature
  drift (RFC MK-052).
- Repository governance records now reconcile every RFC lifecycle entry with
  the files on disk and the historical release record.
- The snora 0.25 dependency is no longer vendored. The workspace manifest uses
  the crates.io release, and `Cargo.lock` currently resolves `snora`,
  `snora-core`, `snora-design`, and `snora-widgets` to 0.25.2.

---

## [0.10.0] — 2026-06-20

RFC MK-050 — Migrate to snora 0.25 Snora Design system. A minor-version bump
reflecting the dependency upgrade and the new accessibility-focused theming.

### Added — Snora Design token system

snora 0.25.0 introduces "Snora Design": an iced-free design-token layer
(`snora-design` crate) with WCAG-AA contrast-tested presets and an iced style
bridge. apimokka now adopts it via the `design` feature.

**Four themes** (was: Light / Dark binary toggle):
- Light, Dark — iced's native themes (visual continuity)
- **High Contrast Light, High Contrast Dark** — built from snora's
  contrast-verified high-contrast presets, surfaced as a custom iced palette

Selectable in Settings → Appearance via a four-option segmented picker.

**Accessibility improvements:**
- `theme::muted()` now derives its color from the matching snora preset's
  `text_muted` role instead of a fixed grey — so high-contrast modes get a
  properly-contrasted muted text color rather than the same value as standard.
- Cards and panels gain a visible 1–1.5px border in high-contrast modes
  (`theme::is_high_contrast` + `theme::hc_border`), since shadow-only elevation
  is insufficient for low-vision users (WCAG non-text-contrast).

### Changed

- `ThemeChoice` enum extended from 2 to 4 variants with `tokens()`, `iced()`,
  `is_dark()`, `label_key()`, `all()` helpers.
- `Message::SetTheme(ThemeChoice)` added; the Settings picker dispatches it.
  `ToggleTheme` retained (cycles through all four).
- Root `Cargo.toml`: `snora` dependency upgraded `0.18` → `0.25` with
  `features = ["widgets", "design"]`.

### Vendored

snora 0.25.0 is not yet published to crates.io. The four crates (`snora`,
`snora-core`, `snora-design`, `snora-widgets`) are vendored under `vendor/`
and added as workspace members. When 0.25 publishes, the dependency reverts to
a registry version and `vendor/` is removed.

The existing snora layout API (`AppLayout`, `render`, `Dialog`, `Sheet`,
`SheetEdge`, `SheetSize`, `LayoutDirection`) is unchanged between 0.18 and 0.25
— no call-site breakage.

### Tests
99 total (92 app + 7 model). New MK-050 tests: four theme variants, toggle
cycle, distinct tokens per preset, native-vs-custom iced theme mapping,
high-contrast detection, SetTheme handler, is_dark classification, card/panel
style builds for all themes, settings view builds in high-contrast.

---

## [0.9.19] — 2026-06-13

Audit release: codebase, tests, docs, and RFC alignment verified and corrected.

### Fixed — Bug: DismissNotice cleared the undo stack

`DismissNotice` called `self.undo_stack.retain(|_| false)`, which silently
cleared the entire undo history whenever the user dismissed a feedback banner
(e.g. "Deleted rule"). The comment below it correctly stated that the undo stack
should *not* be cleared — only the banner notice text.

The `retain` call is removed. Dismissing the banner now hides the text only;
⌘Z continues to work after dismissal.

Regression test added: `dismiss_notice_does_not_clear_undo_stack` verifies
the complete sequence: delete rule → undo stack non-empty → dismiss notice →
undo stack still non-empty → ⌘Z works.

### Fixed — Documentation accuracy

**README.md** rewritten:
- `snora 0.8` → `snora 0.18` (correct dependency version)
- Quick Start corrected: the app now starts at the mode picker (MK-046), not
  directly at the workspace with pre-loaded data
- Feature list updated to include audience modes, undo/redo, first-launch flow,
  wizard starters, test rule dialog, bottom drawer, trace completeness
- Phantom RFC numbers (MK-001, MK-003, MK-007, MK-015) replaced with correct
  cross-references

**ROADMAP.md:** test count updated from "64 at v0.9.15" to "88 at v0.9.18".

**rfcs/README.md:** MK-049 added to the implemented table.

**docs/src/architecture.md:** `snora 0.8` → `snora 0.18`.

**docs/src/README.md:** Added a note that `designer-brief.md` and
`ux-redesign.md` are historical documents from the v0.3.x era, preserved for
reference but not current.

### Tests
89 total (82 app + 7 model). +1 regression test for DismissNotice.

---

## [0.9.18] — 2026-06-13

RFC MK-049 — Minor gap fixes. The three minor unresolved gaps identified in
the post-MK-048 audit are now closed.

### Fixed — DuplicateRule (was a stub)

`DuplicateRule` previously incremented `dirty_count` and did nothing else.
It now creates a proper copy of the selected rule:
- Fresh `NodeId` for the copy
- Inserted immediately after the original in the same rule set
- Copy is selected after duplication
- Rule set marked dirty
- Recorded on the undo stack as `UndoCommand::AddRule` — ⌘Z removes the copy

### Fixed — Test rule dialog evaluates header and body conditions

`run_stub_test()` previously matched only on method and URL path. Rules with
header or body conditions always showed `Matched` regardless of the test input.

Now evaluates all condition types:

**Header conditions (all 9 `HeaderOp` variants):** Equal, Contains, StartsWith,
EndsWith, Regex (best-effort), Exists, Absent, NotEqual, WildCard (best-effort).
Test input: `headers_text` parsed as `name: value` lines.

**Body conditions (all `BodyOp` variants):** String-coerced ops (Equal,
EqualString, Contains, StartsWith, EndsWith), type-aware (EqualTyped),
numeric f64 (EqualNumber, GreaterThan, LessThan, GreaterOrEqual, LessOrEqual),
integer (EqualInteger), array (ArrayLengthEqual, ArrayLengthAtLeast,
ArrayContains), presence (Exists, Absent). Regex skipped (best-effort).
Test input: `body` parsed as `serde_json::Value` (already a dep). Invalid JSON
when body conditions exist returns `TestRuleResult::Error`.

Dotted-path accessor implemented: `a.b.c` for nested objects, `items.0.name`
for array indices — matching the engine's own path semantics.

### Fixed — `ConfirmAction::DeleteRule` dead code removed

`ConfirmAction::DeleteRule` was added before MK-039 made delete-rule non-modal.
After MK-039 no code dispatches `ConfirmRequest(ConfirmAction::DeleteRule(...))`,
so the variant, its match arm in `ConfirmProceed`, and its label-key entry were
dead code. All three removed. `ConfirmAction::DeleteRuleSet`, `DiscardChanges`,
`SwitchWorkspace`, and `RevertFile` are unaffected.

### Tests
88 total (81 app + 7 model). New: duplicate creates copy, duplicate is undoable,
header condition match/no-match/absent, body condition match, invalid body,
confirm-delete-rule-set still works, dotted-path nested/array/missing.

---

## [0.9.17] — 2026-06-13

RFC MK-048 — Workspace creation completion. The last two stubs in the
creation flow are now real.

### Added — Real `AddRuleSet`

`AddRuleSet` previously did `dirty_count += 1` — a placeholder from v0.9.1
that was never updated when `AddRule` became real in v0.9.14.

It now creates a proper `RuleSetView` with:
- A generated filename: `rules/rule-set-N.toml` where N = (existing count + 1)
- Empty rules list, `dirty = true`
- The new rule set is immediately selected and opened in the sidebar accordion

On a blank workspace (the Empty starter), the first click on "+ Add rule set"
produces `rules/rule-set-1.toml`; a second click produces `rules/rule-set-2.toml`.

### Added — Wizard starter selector

`WizardStarter` enum: `Empty | Minimal (default) | ShopApi`.

The wizard's "Starter rules" section now shows three radio buttons wired to
`Message::WizardSetStarter`. `WizardCreate` dispatches to the appropriate
mock constructor based on the selection:

| Starter | Content |
|---|---|
| **Minimal** *(default)* | One rule set (`rules/main.toml`) with a single `GET /health → 200 OK` rule |
| **Shop API example** | Full `shop_api_mock()` — two rule sets, weighted/priority strategies, fallback files, middleware scripts |
| **Empty workspace** | No rules, no rule sets — blank slate |

The "Minimal" default is the idiomatic starting point for any new mock service:
a health-check endpoint that confirms the server is alive.

`WizardCreate` also applies the wizard's name/host/port/TLS fields to the Shop
API starter (previously those inputs were ignored when loading the pre-baked mock).

### Added — `mock::minimal_workspace`

```
minimal_workspace(name, host, port, tls) -> WorkspaceSnapshot
  one rule set: rules/main.toml
  one rule:     GET /health → 200 OK + {"status":"ok"}
```

### Tests
77 total (70 app + 7 model). New: AddRuleSet creates real rule set and
increments filename numbers; all three starter variants (Minimal, ShopApi,
Empty); default is Minimal; WizardSetStarter message updates state;
minimal_workspace model constructor.

---

## [0.9.16] — 2026-06-13

Two improvements: ROADMAP housekeeping and MK-047 blank workspace from wizard.

### Changed — ROADMAP.md

Rewritten to reflect the current state of the project. Updated test count
(26 → 69), marked undo/redo as implemented, removed stale notes. The deferred
section now accurately covers only the items that genuinely need persistence or
v2 effort.

### Added — MK-047: Blank workspace from wizard (RFC coming)

Previously `WizardCreate` and `OpenWorkspace` both loaded `shop_api_mock()`,
making "create new workspace" and "open existing workspace" indistinguishable.

`WizardCreate` now calls `mock::blank_workspace(name, host, port, tls)` using
the values the user typed into the wizard:
- Name field → `meta.name` and path
- Host/port → `root_settings.listener_ip` / `listener_port`
- TLS checkbox → `root_settings.tls_enabled`
- Empty name defaults to "my-mock"
- Rule sets, fallback files, and middleware scripts start empty

`OpenWorkspace` (from Dashboard) continues to load the full `shop_api_mock()`
as before — representing opening a pre-existing workspace.

**Centre panel for blank workspace.** When a workspace has no rule sets, the
centre panel now shows "Add a rule set to start mocking requests." with an
"Add rule set" primary button, instead of the "No rule selected / Add rule"
state (which required a rule set to already exist).

**Server state.** After WizardCreate the server state shows Stopped (correct —
a new workspace isn't running yet).

**Welcome notice.** A success notice appears after creating the workspace:
`Workspace "inventory-mock" created. Add a rule set to get started.`

### Tests
69 total (62 app + 7 model). New: wizard name/host/port used in blank
workspace, default name fallback, OpenWorkspace still loads mock, blank
workspace CTA renders, `blank_workspace` constructor.

---

## [0.9.15] — 2026-06-13

RFC MK-046 — First-launch flow. The app now starts at the Welcome screen with
no workspace loaded. The audience mode picker appears full-screen before
anything else, giving the user a calm, uncluttered first decision.

### Changed

**App launch sequence.** Previously the app started directly in `AppView::Workspace`
with mock data pre-loaded, showing the audience mode picker as a dialog on top
of live workspace content — incoherent for a first-time user.

New sequence:
1. No audience mode stored → mode picker fills the screen (nothing behind it).
2. User chooses Guided or Expert → Welcome screen appears.
3. "Open workspace" → Dashboard → click workspace → Workspace view.
4. "Create workspace" → Wizard → fill in → Workspace view.

**Mode picker** is now a properly centred full-screen view (dialog card on a
plain background) rather than an overlay on top of route data.

**`App::new()`** no longer pre-loads the mock snapshot. The snapshot is loaded
only when `OpenWorkspace` or `WizardCreate` is dispatched. This matches the
production contract: the app has no workspace until the user opens or creates one.

### Tests
64 total (58 app + 6 model). New: app starts at Welcome with no snapshot,
mode picker builds before mode is chosen, full Welcome→Dashboard→Workspace
flow, Wizard→Workspace flow, Welcome screen builds after mode is set.

---

# Changelog

All notable changes to apimokka are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [0.9.14] — 2026-06-13

RFC MK-045 — Undo / redo history. Implements a typed command log for
reversible operations, the architecture the real product will use.

### Added

**Typed `UndoCommand` log** (command-log pattern, not snapshot-based):
- `DeleteRule` — undo re-inserts the rule at its original index; redo removes it again.
- `AddRule` — undo removes the newly added rule.
- `MoveRule` — undo moves the rule back to `from_index`; redo re-moves it forward.
- `EditUrlPath` — undo restores `old_value`; redo restores `new_value`.

Stack depth: 25 per direction. Any new edit clears the redo stack.

**Keyboard shortcuts:** ⌘Z / Ctrl+Z for Undo; ⌘⇧Z / Ctrl+Shift+Z / Ctrl+Y for Redo.

**Command palette:** "Undo ⌘Z" and "Redo ⌘⇧Z" at the top of the list.

**Feedback banner:** shows the description of the top-of-undo-stack command
("Deleted rule", "Added rule", "Moved rule", "URL path changed") with the ⌘Z
shortcut label. Dismissing hides the banner without clearing the stack —
⌘Z still works.

**AddRule is now fully implemented** (no longer a stub): adds a real
`RuleView` to the rule set, selects it, and records the add as undoable.

**MoveRuleUp / MoveRuleDown are now fully implemented**: actually swap rules
in the snapshot and record the move as undoable.

### Changed

- `UndoLast` (MK-039 single-entry undo) is now an alias for `Undo`.
- Existing tests updated: `a.undo` → `a.undo_stack.last()`.

### Tests
59 total (53 app + 6 model). New MK-045 tests: delete→undo→redo round-trip,
add-rule undoable, move-rule undoable, url-path undoable, new edit clears redo,
undo/redo keyboard shortcut smoke.

---

## [0.9.13] — 2026-06-13

RFC MK-044 — Bottom drawer completeness. Both panels are now actionable and
informative. Also closes the one remaining stubbed command palette entry.

### Added — Validation panel

**Grouped by rule set.** Each dirty rule set appears as a named heading;
the file name makes it immediately clear which config file has issues.

**Click-to-navigate.** Each validation issue shows the rule summary and a
"Go to rule →" link that calls `JumpToRule` — switching to Routes, selecting
the rule, and closing the drawer in one action.

**Positive confirmation per file.** A rule set with no issues shows
"✓ No issues in <filename>" in muted text, so the absence of issues is
explicit rather than invisible.

**Workspace-level diagnostics first.** Workspace-scope issues (from
`snap.diagnostics`) appear above the per-rule-set groups.

**Proper empty state.** When the workspace is fully valid, a centred
"✓ No validation issues" confirmation replaces the blank panel.

### Added — Save-diff panel

**Rule summaries in dirty rule-set rows.** Each dirty rule-set file shows
"N rules · GET /health, POST /api/orders, …" so the developer knows what
will be saved before committing.

**Fallback file change indicator.** Dirty fallback files show "JSON content
modified" below the file name.

**Clean empty state.** When nothing is dirty, shows "✓ No unsaved changes."

### Added — Command palette

`PaletteCmdAddRule` was the only unwired command (→ `Noop`). It now dispatches
`AddRuleFromPalette`, which closes the palette, switches to Routes, selects the
first rule set, and adds a new rule.

### Changed

`JumpToRule` now also closes `self.drawer`, so navigating from a validation
issue or trace match detail dismisses the drawer automatically.

### Tests
52 total (46 app + 6 model). New: jump-to-rule closes drawer, palette add-rule
navigates, drawer view smoke tests (with issues, clean, all-clean, no-changes).

---

## [0.9.12] — 2026-06-13

RFC MK-043 — Rule-set strategy UI. Surfaces the rule-selection strategy,
per-rule weight/priority fields, and validation feedback — three features the
model and integration reference have carried since v0.6.0 but that were never
exposed in the UI.

### Added

**Strategy section in rule set config panel.** A strategy dropdown
(`FirstMatch`, `UniformRandom`, `WeightedRandom`, `Priority`, `RoundRobin`)
with the engine's own one-line description beside it. The ⓘ hint
(`HintStrategy`) explains what strategy does in context. In Guided mode the
section starts collapsed behind "▸ More rule-set options"; in Expert mode it
is always visible.

**Per-rule weight field** (shown when strategy is WeightedRandom). Appears in
the RESPOND column. Numeric input; empty = default weight 1. In Guided mode
it follows the `rule_when_more` toggle so it is hidden with the other advanced
fields.

**Per-rule priority field** (shown when strategy is Priority). Same placement
and Guided-mode behaviour as weight. Signed integer; negative values allowed
for deprioritised rules.

**Validation issues strip.** When a rule carries validation issues (e.g. the
mock data rule with "WeightedRandom is selected, but this rule has no weight
set"), a warning strip appears above the WHEN/RESPOND editor row, listing
each issue with a ⚠ glyph.

### Tests
46 total (40 app + 6 model). New: strategy change updates snapshot,
weight/priority round-trip, invalid input leaves `None`, Guided reset on mode
switch, toggle flip, rule set config + rule editor smoke tests for all five
strategies and both modes, validation strip build.

---

## [0.9.11] — 2026-06-13

Two-track release: RFC lifecycle governance housekeeping (all implemented RFCs
moved to `rfcs/done/`) and MK-042 trace screen completeness.

### RFC lifecycle migration

Per RFC 000 (lifecycle policy), completed RFCs must live in `rfcs/done/`, not
`rfcs/proposed/`. All 21 previously-implemented RFCs (MK-021 through MK-041)
were migrated in this release:
- Status fields updated to `Implemented (vX.X.X)`.
- MK-031 (Scripts viewer) moved to `rfcs/archive/` — withdrawn/deferred.
- `rfcs/README.md` rebuilt with the correct three-section structure.
- `rfcs/proposed/` is now empty (as expected for a project with no RFCs
  currently under review).

### MK-042 — Trace screen completeness

**Live filter:** The filter text input is wired. Typing filters the event list
by URL path, method, or outcome label (case-insensitive). Empty filter shows
all events.

**Outcome-aware match detail panel:**
- `Matched` — shows rule set file name + rule summary + "Jump to rule" button
  (switches to Routes tab and selects the exact rule).
- `Fallback` — shows file path + status + "Jump to file" button (switches to
  Routes tab and selects the file in the sidebar).
- `Miss` — shows status code + "No rule matched this request." + "Create rule
  for this path" CTA (switches to Routes, adds a rule, pre-fills URL path).
- `Error` — shows error kind + full message in monospace.

**Dropped-count warning:** When the selected event has `dropped_count > 0`, a
warning strip appears in the detail panel ("N events dropped before this one
(queue full)").

**Rule editor trace strip — index-based matching:** `recent_matching_events`
now matches by `rule_set_index` / `rule_index` reported in the trace outcome,
not by URL path substring. URL path heuristic is kept as fallback when indices
don't resolve.

**New messages:** `TraceFilterChanged(String)`, `JumpToRule(NodeId)`,
`JumpToFile(String)`, `AddRuleForPath(String)`.

### Tests
37 total (31 app + 6 model). New: filter empty/path, jump-to-rule/file tab
switch, trace strip index matching, trace view builds for all four outcomes
(including dropped-count warning).

---

## [0.9.10] — 2026-06-13

RFC MK-041 — Layout density: common-first, advanced-behind-More.
MK-040 phase 3. In Guided mode the WHEN panel and Settings screen now surface
only the most-used controls at a glance; the rest expand on demand.

### Added
- **WHEN column layout density** (Guided mode): URL path + method are always
  shown; headers and body condition cards start collapsed behind a
  "▸ More matching criteria" toggle. If rules already have hidden active
  conditions, a count badge ("1 header · 2 body active") is shown next to the
  toggle so nothing silently hides state that affects server behaviour.
- **Settings layout density** (Guided mode): Appearance + Server sections are
  always visible; Logs + Trace sections start collapsed behind a
  "▸ More settings" toggle.
- Both toggles flip with "▾ Fewer …" once expanded, and persist across
  navigation for the session (not reset per rule or page switch).
- Reset behaviour: switching to Guided resets both density toggles to
  collapsed; switching to Expert leaves them (they have no effect in Expert).

### Unchanged (by design, Expert parity)
- Expert mode: all four WHEN cards and all four Settings sections always
  visible. No change from v0.9.0.

### Tests
31 total (25 app + 6 model). New: `guided_when_starts_collapsed_and_resets_on_mode_switch`,
`rule_when_more_persists_across_rule_navigation`, `settings_advanced_toggle_works`,
routes + settings view smoke in Guided collapsed and expanded.

---


## [0.9.9] — 2026-06-13

Audience modes — Guided and Expert (RFC MK-040). A user-chosen presentation
density that adapts scaffolding **without ever renaming the domain**. This is
the structural answer to a real shift: as AI takes over implementation work,
people configuring mocks are increasingly not the people who would hand-write
the config. apimokka serves both from one product, one vocabulary.

### Principle
Guided mode *adds scaffolding*; it never *renames the domain*. The scaffold
bridges the user toward the real concept (so they grow less dependent on it),
rather than substituting a private vocabulary that fails the moment the user
leaves apimokka for an AI chat, the docs, or a colleague.

### Added
- **`AudienceMode` { Guided, Expert }** in `apimokka-model` (pure, no UI dep).
- **First-run mode picker**: a non-dismissible dialog shown on first launch
  (when no mode is stored). Highest dialog priority; no Esc/backdrop close.
- **Settings → Appearance → Guidance**: a Guided/Expert segmented control, so
  the choice is reversible at any time.
- **Mode-aware concept hints**: Guided expands the ⓘ hint inline as a plain
  gloss under the field heading; Expert shows the ⓘ marker only. The hint text
  is identical in both modes.
- **Mode-aware errors**: `FriendlyProblem` gains `technical_detail` (errno /
  raw message). Expert shows it inline; Guided collapses it behind a
  "Show details" / "Hide details" toggle.

### Changed
- `FriendlyProblem::port_in_use` now keeps `EADDRINUSE` in `technical_detail`
  and the plain line stays jargon-free — the same struct serves both modes.

### Tests
26 total (20 app + 6 model): first-run picker shows then choice persists,
Guided/Expert scaffolding, Expert expands details by default, vocabulary
identical between modes, picker + error banner build in both modes,
`AudienceMode` behaviour.

### Out of scope (future phases)
- Inferring mode from behaviour (explicit choice only).
- Per-screen layout density (common-first / advanced-behind-More).
- Real on-disk persistence (mockup keeps the setting in memory).

---

## [0.9.8] — 2026-06-13

Dependency upgrade: `snora` 0.8 → 0.18.1. Zero code changes required —
the upgrade to 0.18.0 was confirmed migration-safe (the only breaking change,
`AppLayout` gaining `#[non_exhaustive]` in 0.11, did not affect apimokka
because builder-pattern construction was always used). 0.18.1 followed
immediately to fix a build bug in 0.18.0.

---

## [0.9.7] — 2026-06-13

Added 13 plain `#[test]` unit and smoke tests (no `iced_test` dependency —
rationale documented in `ROADMAP.md`: the library is too immature). Coverage
areas: mock workspace well-formedness, fallback files, trace IDs, selection
clearing, accordion state, MK-038 lifecycle, and view-build smoke tests per
tab and per centre-panel branch.

---

## [0.9.6] — 2026-06-13

UI and bug-fix pass. Header top bar split into plain `apimokka` label and a
separate `workspace-name ▼` button. Tab text centred within equal-width cells.
Sidebar accordion: rule sets use single-open accordion (only one expanded at a
time); Fallback files and Middleware scripts sections collapsed by default;
Middleware scripts gained the missing "+ Add .rhai" entry. Fixed: clicking a
fallback file or middleware script selected it but the centre panel showed the
rule set config instead — root cause was `SelectFileRoute` and `SelectScript`
not clearing `selection.rule_set`. Appearance controls (theme, locale, command
palette shortcut) moved from top bar to Settings → Appearance.

---

## [0.9.5] — 2026-06-13

Intuitive-workflow improvements for developers who are rusty on HTTP
fundamentals (RFC MK-039). A UI/UX review proposed relabeling the domain
vocabulary for a non-technical audience; since the persona is unchanged
(developers/QA), the relabeling was rejected and the sound *mechanics* were
adopted instead. Domain vocabulary (workspace, rule, trace, JSON, header,
status code) is kept.

### Added
- **ⓘ concept hints** on the body-path, URL-operator, and header-operator
  cards. Opt-in (revealed on hover), so the default view stays uncluttered.
  Teach the exact gotcha in domain language — e.g. body paths are dotted
  (`user.id`), not JSONPath (`$.foo`).
- **FriendlyProblem error model** (`apimokka-model::friendly_error`): pure,
  no-UI struct `{ title, detail, action_label }`. Developer-register content
  that names the real cause and fix (e.g. port conflict keeps `EADDRINUSE`).
- **Feedback banner** in the shell: friendly error > undo > success notice,
  shown between the tab bar and screen.
- **Non-modal undo** for deleting a rule — the rule is removed immediately and
  an "Undo" banner restores it. High-blast-radius actions (delete rule set,
  discard all, revert file) keep the confirm dialog.
- **Disabled-action reasons**: `widgets::action_with_reason` renders a primary
  action that, when blocked, shows a one-line reason beside it instead of a
  dead button. Wired to Test rule (gated when the rule has no match criteria).
- `touch` token module: `MIN` 44 px (WCAG floor), `COMFORTABLE` 52 px.

### Changed
- **Comfort type scale** (WCAG AA): body 14 → 16 px, section 17 → 18, title
  22 → 24, display 32 → 36. Caption stays 12 px (metadata); mono stays 13 px.
- Primary buttons adopt a 52 px minimum height.
- Save now surfaces a success notice.

### Rejected (from the review, with rationale in RFC MK-039)
- Renaming workspace/rule/trace/JSON/header/body/status code — increases
  cognitive load for the actual persona.
- Blocked-technical-terms build test — would forbid the domain's own
  vocabulary.
- "Local helper" euphemism for the server — obscures port diagnostics.
- Parallel copy crate — we keep the exhaustive `apimokka-i18n`.

### Tests
19 total (14 app + 5 model): added non-modal undo round-trip, save notice,
problem→Settings routing, comfort-floor assertion, FriendlyProblem
constructors.

---

## [0.9.4] — 2026-06-05

Fallback file editor rebuilt around a correct data lifecycle (RFC MK-038).

### Fixed
- The JSON content editor now uses iced's multi-line `text_editor` widget
  (monospace, fills available height). The v0.6.2 single-line `text_input`
  could not render or edit multi-line JSON.
- Dirty state is now **derived** (`draft != saved`, trailing-newline
  normalised), never keystroke-counted. The top-bar `Unsaved (N)` chip counts
  dirty rule files + dirty fallback files without inflation.
- Rule auto-save no longer commits fallback file drafts as a side effect:
  `auto_save_rules()` (rule edits) is split from `simulate_save()` (global
  Save, which commits all dirty drafts).
- Discard (save-diff drawer) now actually reverts every dirty draft to its
  saved baseline instead of only zeroing the counter.

### Added
- **Two-buffer lifecycle per file** (saved baseline + draft buffer). Drafts
  persist across file switches; the sidebar shows a dirty dot (●) on every
  file with unsaved edits, mirrored in the editor header.
- **Explicit save** for file content (per-file Save button + global Save),
  with the rationale documented in MK-038: free-text JSON passes through
  invalid transient states while typing, so auto-saving would have the live
  server serving broken JSON mid-edit.
- **Revert** (ghost button, dirty-gated) routed through the standard confirm
  dialog; restores the saved baseline exactly.
- **Live JSON validity badge** (`✓ Valid JSON` / `⚠ Invalid JSON — will be
  served as-is`). Warns, never blocks: serving malformed JSON is a
  legitimate client-error test case.
- State hint in the footer: "Saved — changes take effect on the next
  request." (apimock-rs reads fallback files per request; no reload needed.)
- Dirty fallback files now appear in the save-diff drawer alongside dirty
  rule-set files.
- RFC MK-038 (fallback file editor lifecycle) in `rfcs/proposed/`.

---

## [0.9.3] — 2026-06-05

File-system based routing becomes a first-class editing surface.

### Added
- Selecting a fallback `.json` file in the Routes sidebar opens a dedicated
  editor in the centre panel: header (filename + `Serves: GET /users` pill +
  explanation), content card with path breadcrumb, footer (status code,
  Format JSON, Save).
- Sidebar fallback entries render two lines: `{ }` glyph + filename, with the
  served URL path below. `+ Add file` button (stubbed; creation dialog is a
  future RFC).
- Mock JSON content seeded for health.json / users.json / order-created.json.
- serde_json dependency for pretty-printing.

---

## [0.9.2] — 2026-06-05

Navigation and button-style refinements per design review.

### Changed
- **Tab bar replaces the side rail**: a horizontal strip below the top bar
  (Routes · Trace · Settings). The active tab carries a 3 px accent strip.
  Frees the full rail width (~120 px) for the three-column Routes layout.
- Segmented buttons (method Any/GET/POST/…, respond mode tabs, Test Rule
  method picker) now use `button::text` unconditionally — no default grey
  outline; the active option is highlighted by a tinted selection card.
- URL path card simplified: removed the redundant checkbox echoing the card
  title.

### Removed
- **Scripts tab** (Rhai middleware viewer) — deferred with rationale in
  `ROADMAP.md`: a read-only stub without syntax highlighting and error
  surfacing is a misleading affordance for the persona users. Future RFC
  (MK-040+).

---

## [0.9.1] — 2026-05-23

Complete restart. The v0.1–v0.5 implementation is archived in
`rfcs/archive/` (MK-001..MK-020, all marked Superseded). The new
implementation follows the MK-021..MK-037 RFC series, which was derived
from the v1.0.0 UI/UX and GUI workflow design document.

### What was replaced

All screen code was discarded and rewritten from scratch:
- `theme.rs` — MK-022 design tokens (`space.S1–S6`, `size.CAPTION–DISPLAY`,
  `radius.SM/MD/LG/XL/PILL`, `pad.*`, 10 container style helpers)
- `apimokka-i18n` — 248 keys rebuilt per MK-036 microcopy (EN + JA)
- `message.rs` — full MK-021/MK-035 message set
- `selection.rs` — `WorkspaceTab` (4 destinations), `DrawerMode`,
  `RouteSelection`
- `app.rs` — MK-035 state machines (server, save, trace) cleanly separated
- All 12 screens rewritten

### What is new in v0.6.0

**Design system (MK-022)**
- 6-step spacing scale: `space.S1`(4) → `space.S6`(24)
- 6-step type scale: `size.CAPTION`(12) → `size.DISPLAY`(32)
- 4 radius steps: `radius.SM`(6) → `radius.XL`(18)
- Cards: soft shadow + `radius.LG` (no hard borders)
- Selected-card: primary tint adapts to light/dark (10%/18% opacity)

**Routes workbench (MK-028)**
- Three-column layout: 280px sidebar / flexible editor / 290px right column
- Rule editor: two-column WHEN → RESPOND with prominent arrow divider
- Live trace strip with two-line event rows and replay ⟲ button
- Rule inspector (duplicate / move / delete)
- Left sidebar: rule sets with dirty markers, rules with validation glyphs,
  fallback files with URL hints, middleware scripts
- `Any/GET/POST/PUT/PATCH/DELETE` method segmented buttons
- All condition cards (URL path / headers / body) with full field sets

**Trace screen (MK-029)**
- Filter bar (path filter, pause, clear)
- Event rows: outcome glyph + method + path + time + duration
- Match detail panel: request, outcome, actions

**Welcome (MK-025)**
- Hero at `size.DISPLAY` (32px)
- Three-step pipeline diagram (middleware → rule sets → fallback files)

**Dashboard (MK-025)**
- Search-filtered recent workspace list
- Workspace cards with name / path / last-opened

**Wizard (MK-026)**
- Single-page with three collapsible sections (Server, Starter, Trace)
- Create button opens Routes with mock workspace

**All overlays (MK-033, MK-034)**
- Command palette: 17 commands, shortcut hints (⌘K, Esc), live filter
- Test Rule dialog: method picker, path/headers/body inputs, result banner
- Dotted-path assistant: JSON leaf extractor, Use button per leaf
- Confirm dialog: title + body + Cancel (ghost) + Proceed (danger)
- Workspace menu: current workspace + recents + Open/Create buttons

**Settings (MK-030), Scripts (MK-031), Bottom drawer (MK-032)**
- Settings: sectioned cards with reload/restart impact labels
- Scripts: list + read-only monospace viewer
- Validation drawer + save-diff drawer

**MK-035 state machines**
- `ServerState`: Stopped → Starting → Running → ReloadPending / RestartRequired → Error
- `SaveState`: auto-save on rule edits; restart-class changes require explicit save
- `ThemeChoice`: Light/Dark toggle (☾/☀ in top bar)
- Keyboard subscription: Esc (close overlay stack), ⌘K (toggle palette)

**MK-023 keyboard**
- `Esc` closes topmost overlay in priority order
- `Ctrl/Cmd+K` toggles command palette
- Overlay priority: confirm > workspace-menu > palette > test-rule > path-assistant > drawer

---
