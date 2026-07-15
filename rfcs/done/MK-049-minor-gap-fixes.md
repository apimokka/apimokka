# RFC MK-049 — Minor gap fixes

**Status.** Implemented (v0.9.18)
**Tracks.** Three minor unresolved gaps identified in the post-MK-048 audit.
**Touches.** `app.rs` (DuplicateRule, run_stub_test, dead code removal).

## 1. DuplicateRule (was a stub)

Creates a copy of the selected rule with a fresh `NodeId`, inserted immediately
after the original in the same rule set. The copy is selected and the rule set
is marked dirty. Recorded on the undo stack so the duplicate can be undone
with ⌘Z.

## 2. run_stub_test() — header and body condition checking

The "Test rule" dialog previously matched only on method and URL path. Rules
that have header or body conditions always showed `Matched` even when the test
request didn't satisfy those conditions.

The test runner now evaluates:
- **All 9 `HeaderOp` variants**: Equal, Contains, StartsWith, EndsWith, Regex,
  Exists, Absent, NotEqual, WildCard
- **All `BodyOp` variants** (except Regex and WildCard which are skipped with
  `Matched` — best-effort): Equal/EqualString, EqualTyped, EqualNumber,
  EqualInteger, GreaterThan, LessThan, GreaterOrEqual, LessOrEqual,
  ArrayLengthEqual, ArrayLengthAtLeast, ArrayContains, Contains, StartsWith,
  EndsWith, Exists, Absent

Test inputs:
- `headers_text`: one `key: value` per line (case-insensitive name lookup)
- `body`: parsed as `serde_json::Value` (already a dependency)
- The dotted-path accessor is implemented to match the engine's own semantics
  (`a.b.c`, `items.0.name`)

## 3. ConfirmAction::DeleteRule — dead code removal

`ConfirmAction::DeleteRule` was added when delete-rule required a confirm dialog.
MK-039 made delete-rule non-modal (direct delete + undo). No dispatch site
exists for `ConfirmAction::DeleteRule` after MK-039 — it became unreachable.

Removed: the enum variant, the match arm in the confirm-dialog handler, and
the label-key match arm. The confirm-dialog infrastructure is otherwise kept
intact for `DeleteRuleSet`, `DiscardChanges`, `SwitchWorkspace`, `RevertFile`.
