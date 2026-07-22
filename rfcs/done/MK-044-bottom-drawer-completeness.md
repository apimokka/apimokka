# RFC MK-044 — Bottom drawer completeness

**Status.** Implemented (v0.9.13)
**Tracks.** Bottom drawer (MK-032): validation panel and save-diff panel.
Also: the one stubbed command palette entry (`PaletteCmdAddRule → Noop`).
**Touches.** `screens/bottom_drawer.rs`, `message.rs`, `app.rs`,
`screens/command_palette.rs`, i18n.
**Follows.** MK-032 (original drawer spec), MK-042 (trace jump-to-rule pattern).
**Amended by.** MK-053 section 5.1 (durable diagnostic ordering and
presentation).

## Context

The bottom drawer has two panels. Both were implemented minimally in v0.6.0.

**Validation panel gaps:**
- Flat list — no grouping by rule set, so the user cannot tell which file an
  issue belongs to at a glance.
- No click-to-navigate — knowing an issue exists is useful; being able to jump
  directly to the offending rule is actionable.
- Wrong empty state — shows the drawer title string instead of a positive
  confirmation.
- Workspace-level diagnostics (`snap.diagnostics`) were rendered alongside
  rule issues without visual distinction.

**Save-diff panel gaps:**
- Lists dirty file paths only — no indication of what changed inside each file.
- For rule-set files: doesn't show which rules (or how many) are affected.
- For fallback files: doesn't distinguish JSON-content edits from no change.

**Command palette:**
- `PaletteCmdAddRule → Noop` was the only unwired command (adding a rule
  requires knowing the target rule set).

## Validation panel (new design)

```
Validation
──────────────────────────────────────
ℹ Workspace            (workspace-level diagnostics first)
  No include filter is set. All supported files are visible.

⚠ rules/error-scenarios.toml         (grouped by rule set)
  POST /api/orders                    (rule summary)
  WeightedRandom is selected, but this rule has no weight set.
                                 [ Go to rule → ]

✓ No issues in rules/main.toml        (clean rule set)
──────────────────────────────────────
✓ No validation issues                (empty-state when nothing found)
```

Clicking "Go to rule →" dispatches `JumpToRule(id)`. `JumpToRule` is extended
to also close the drawer (`self.drawer = None`).

## Save-diff panel (new design)

```
Save diff
──────────────────────────────────────
2 files with unsaved changes

  ● rules/main.toml
    3 rules · GET /health, POST /api/orders, GET /api/users

  ● responses/users.json
    JSON content modified
──────────────────────────────────────
[ Discard ]                 [ Save all ]
```

Rule-set rows expand to show the rule summaries for all rules in the file
(since the mockup doesn't track individual rule-level change history, showing
all rules in a dirty file is accurate and useful). Fallback file rows show
"JSON content modified".

## Command palette: AddRule

`PaletteCmdAddRule` now dispatches `AddRuleFromPalette`. The handler switches
to Routes, selects (and opens) the first rule set, and adds a new rule to it.
This is the palette's best available approximation — rule set selection may
be refined when per-rule-set palette support is added later.

## New messages

```rust
AddRuleFromPalette,
```

`JumpToRule` extended (no new message) to close `self.drawer`.

## Acceptance criteria

- Validation panel groups issues by rule set with the file name as a heading.
- Workspace-level diagnostics appear above rule-set groups.
- Clicking "Go to rule" navigates to the rule AND closes the drawer.
- A rule set with no issues shows "✓ No issues in <file>."
- When no issues exist anywhere, shows "✓ No validation issues."
- Save-diff panel shows rule summaries for dirty rule-set files.
- Save-diff panel shows "JSON content modified" for dirty fallback files.
- `PaletteCmdAddRule` adds a rule and navigates to it instead of Noop.
- Zero errors, zero warnings, existing + new tests pass.

## Later validation-panel supersession (2026-07-21)

MK-053 section 5.1 supersedes only this RFC's validation-panel grouping and
clean-file-row presentation. Durable diagnostics now remain in one fixed
cross-source order and are not regrouped by target; clean rule sets do not add
rows when other diagnostics exist. Rule-level rows retain their owning file
path in their scope, and target rows retain a localized text action plus the
navigation arrow. The all-clear state, target navigation, save-diff behavior,
and command-palette behavior above remain authoritative.

The original acceptance criteria record what shipped in v0.9.13. This note
records the current superseding behavior without rewriting that historical
release claim or changing MK-044's Implemented lifecycle state.
