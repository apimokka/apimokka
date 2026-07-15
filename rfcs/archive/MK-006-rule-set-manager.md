# RFC MK-006 — Rule-set manager and rule list

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Rule-set tree, rule list operations.
**Touches.** `screens/routes.rs`, `shell/right_inspector.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| User can understand and navigate rule sets without opening TOML | ✅ | Tree in left sidebar with rule summaries |
| Rule order and rule validity visible | ✅ | Rules render in order; validation glyph (⚠) per rule |
| All rule list operations map to structured commands | ✅ | `Message::{AddRule, DeleteRule, MoveRuleUp/Down, DuplicateRule}` |
| Keyboard users can add, select, move, and delete rules | ❌ | No keyboard subscription wired (see MK-015) |

## Deferred

Keyboard handlers for `Insert` (add rule), `Delete` (delete rule),
`Alt+Up/Down` (move rule). These are blocked on the broader keyboard
infrastructure tracked in MK-015.
