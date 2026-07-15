# RFC MK-048 — Workspace creation completion

**Status.** Implemented (v0.9.17)
**Tracks.** Making `AddRuleSet` real; wiring the wizard starter-rules selector.
**Touches.** `apimokka-model/src/mock.rs`, `app.rs`, `message.rs`,
`screens/wizard.rs`, i18n.
**Follows.** MK-047 (blank workspace from wizard), MK-026 (workspace wizard).

## Motivation

After MK-047, creating a workspace via the wizard produces a blank workspace.
The user's first action is to add a rule set — but `AddRuleSet` is still a stub
that just increments a counter. `AddRule` became real in v0.9.14; `AddRuleSet`
was not updated at that time. This inconsistency is now the most visible gap.

Separately, the wizard's "Starter rules" section has two checkboxes that are not
connected to any state. The starter choice has no effect on what `WizardCreate`
produces. A developer using the wizard expects their selection to matter.

## Changes

### `AddRuleSet` (real implementation)

Creates a `RuleSetView` with:
- Generated filename: `rules/rule-set-N.toml` where N = (current rule-set count + 1)
- Empty `rules` list
- `file.dirty = true` (unsaved)
- Selects the new rule set and opens it in the accordion
- Records on the undo stack: `UndoCommand::DeleteRule` is re-used via a new
  `UndoCommand::DeleteRuleSet` variant so the add is reversible

### Wizard starter selector

`WizardState` gains:
```rust
pub starter: WizardStarter,
```

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardStarter {
    Empty,
    #[default]
    Minimal,   // default: 1 rule set, 1 GET /health → 200 rule
    ShopApi,   // full shop_api_mock() for exploring features
}
```

The wizard's starter section switches from disconnected checkboxes to three
`radio` buttons. A one-line description is shown under each option.

`WizardCreate` dispatches to `blank_workspace`, `minimal_workspace`, or
`shop_api_mock` based on `wizard.starter`.

### `mock::minimal_workspace`

A new constructor:
```
name + host + port + tls
  → one rule set: rules/main.toml
  → one rule: GET /health → "200 OK" + {"status":"ok"} text
  → no fallback files, no middleware
```

This is the idiomatic "first rule" that a developer adds to any new mock
service: verify the server is alive at `/health`.

## Acceptance criteria

- Clicking "+ Add rule set" in an empty sidebar creates a real rule set named
  `rules/rule-set-1.toml`, selects it, and shows it in the centre panel.
- Clicking it again creates `rules/rule-set-2.toml`, etc.
- Wizard: the three radio buttons change `wizard.starter`; the selection
  persists in the wizard form until the user changes it.
- `WizardCreate` with Minimal → workspace with one rule set + one rule.
- `WizardCreate` with ShopApi → full `shop_api_mock()` workspace.
- `WizardCreate` with Empty → blank workspace (existing behaviour).
- Zero errors, zero warnings, all tests pass.
