# RFC MK-001 — App shell and snora AppLayout contract

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Top-level GUI architecture.
**Touches.** `crates/apimokka-app/src/shell/`, `crates/apimokka-app/src/main.rs`

## Summary

Defines the workspace shell as a snora `AppLayout` composition:

```
┌──────────────────────────────────────────────────────────┐
│                       top_bar                            │
├──────┬───────────────────────────────────────────────────┤
│ rail │    screen body (tab-dispatched)    │  inspector   │
├──────┴───────────────────────────────────────────────────┤
│              bottom drawer (snora Sheet)                 │
└──────────────────────────────────────────────────────────┘
```

The left rail is embedded directly in the body row (not in snora's `side_bar`
slot) to allow full border and sizing control. The AppLayout `side_bar` slot
receives a zero-width placeholder.

## Selection stability

All selection state is anchored to `NodeId` values (v4 UUIDs from the engine
model). Resolution to concrete `RuleView`/`RuleSetView` happens at render time
via `WorkspaceSnapshot::find_rule` / `find_rule_set`. Selection survives
re-snapshots, reorderings, and intermediate edits.

## Outer routing

`AppView` distinguishes four outer states: `Welcome`, `Dashboard`, `Wizard`,
`Workspace`. Only `Workspace` renders the full shell; the others render
full-window standalone screens.

## Snora overlay wiring

- Bottom drawer → `Sheet::new(content).at(SheetEdge::Bottom).with_size(SheetSize::Ratio(0.30))`
- Command palette → `Dialog::new(content)`, dismissed by `on_close_modals`
- Dotted-path assistant → `Dialog::new(content)` (command palette takes priority when both are nominally open)
