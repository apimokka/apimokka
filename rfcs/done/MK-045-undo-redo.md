# RFC MK-045 — Undo / redo history

**Status.** Implemented (v0.14.0)
**Tracks.** Command-log undo/redo for rule-level operations.
**Touches.** `app.rs`, `message.rs`, `shell/view.rs` (feedback banner),
`screens/command_palette.rs`, keyboard subscription, i18n.
**Follows.** MK-039 (non-modal undo for rule deletion), MK-033 (command
palette). Supersedes the single-entry undo from MK-039.

## Context

MK-039 added non-modal undo for rule deletion — the rule is removed and a
banner offers an Undo button. This is the right pattern but limited to one
operation (delete rule) and holds only one entry.

The ROADMAP records undo/redo as "Needs command-log in App". This RFC
implements a typed command log — not snapshot-based — so the real product
can adopt the same design without the memory and clone cost of copying the
full workspace snapshot on every keystroke.

## Scope

**Undoable operations:**
- Delete rule (migrated from MK-039 single-entry undo)
- Add rule (new)
- Move rule up / down (new)
- URL path edit (new — captures value before and after)

**Not in scope:** header/body condition field edits, method changes,
strategy/weight/priority changes. These are fine-grained enough that per-key
undo would be surprising; the natural boundary is the operation level.

## Data model

```rust
// In apimokka-app (no model-crate dependency needed — these are UI concerns)
pub enum UndoCommand {
    DeleteRule {
        rule_set: RuleSetId,
        index: usize,
        rule: apimokka_model::snapshot::RuleView,
    },
    AddRule {
        rule_set: RuleSetId,
        rule_id: apimokka_model::NodeId,
    },
    MoveRule {
        rule_set: RuleSetId,
        rule_id: apimokka_model::NodeId,
        from_index: usize,    // original position
    },
    EditUrlPath {
        rule_id: apimokka_model::NodeId,
        old_value: String,    // value before the edit
    },
}

impl UndoCommand {
    /// Human-readable description for the undo/redo banner.
    pub fn description(&self) -> &'static str { ... }
}
```

## Stacks

```rust
// In App
pub undo_stack: Vec<UndoCommand>,   // capped at UNDO_STACK_DEPTH = 25
pub redo_stack: Vec<UndoCommand>,
```

Invariants:
- Any new undoable action pushes to `undo_stack` and clears `redo_stack`.
- `Message::Undo` pops the top of `undo_stack`, applies the inverse
  operation, pushes the *forward* command to `redo_stack`.
- `Message::Redo` pops `redo_stack`, re-applies the command, pushes to
  `undo_stack`.
- If `undo_stack` reaches UNDO_STACK_DEPTH, drop the oldest entry.

## Keyboard

Extend the keyboard subscription:
- `⌘Z` / `Ctrl+Z` → `Message::Undo`
- `⌘Shift+Z` / `Ctrl+Shift+Z` → `Message::Redo`
- `Ctrl+Y` (Windows convention) → `Message::Redo`

## Palette

Add two entries at the top of the command list:
- "Undo" with shortcut `⌘Z`
- "Redo" with shortcut `⌘⇧Z`

Both are conditionally active (disabled when the respective stack is empty,
which is the palette's standard behaviour for unavailable commands).

## Feedback banner

The existing priority order (error > undo > notice) is updated: the undo
entry is now driven by `undo_stack.last()` rather than `app.undo`. The
banner shows the description of the undoable command:

```
Deleted "POST /api/checkout"     [Undo ⌘Z]  [✕]
```

When a redo is available but no undo, a parallel redo banner is shown:

```
Redo available                   [Redo ⌘⇧Z] [✕]
```

## Migration from MK-039

`pub undo: Option<UndoEntry>` is removed. `UndoEntry` is replaced by
`UndoCommand::DeleteRule`. Tests that reference `app.undo` are updated to
check `app.undo_stack.last()` with a pattern match.

## Acceptance criteria

- Deleting a rule pushes `DeleteRule` to the undo stack; ⌘Z restores it.
- Adding a rule pushes `AddRule`; ⌘Z removes it.
- Moving a rule up/down pushes `MoveRule`; ⌘Z reverses the move.
- Editing the URL path field pushes `EditUrlPath`; ⌘Z restores the old path.
- Any new undoable action clears the redo stack.
- ⌘⇧Z / Ctrl+Y re-applies an undone action and makes it un-undoable again.
- Undo/redo appear in the command palette.
- Feedback banner shows the top undoable command description.
- Stack is capped at 25; oldest entries drop silently.
- Zero errors, zero warnings, existing + new tests pass.
