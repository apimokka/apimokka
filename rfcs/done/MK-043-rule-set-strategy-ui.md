# RFC MK-043 — Rule-set strategy UI

**Status.** Implemented (v0.12.0)
**Tracks.** Surfacing the rule-selection strategy, per-rule weight and
priority fields, and validation feedback for strategy mismatches.
**Touches.** `screens/routes.rs` (rule_set_config, rule_editor),
`message.rs`, `app.rs`, i18n.
**Follows.** MK-028 (routes workbench), MK-041 (layout density).

## Context

The apimock-rs engine lets each rule set define how to pick among multiple
matching rules (`Strategy`). Five variants exist: `FirstMatch`,
`UniformRandom`, `WeightedRandom`, `Priority`, `RoundRobin`. Two variants
require per-rule numeric fields (`weight` for WeightedRandom, `priority` for
Priority). The model already carries all of this. The UI has never exposed it.

As a result, the current mockup cannot represent a large class of real
configurations — any workspace that uses weighted or priority routing looks
the same as one using first-match. The strategy panel in the rule set config
is blank, and the weight/priority fields in the rule editor are invisible.

## Design

### Rule set config panel — strategy section

A dedicated section below the rule list:

```
Strategy
  [FirstMatch  ▼]   The first matching rule in list order wins.
```

```
Strategy
  [WeightedRandom  ▼]   Matching rules are selected randomly using
                        per-rule weights.
```

The description text (`Strategy::help()`) is shown inline next to the
dropdown as a one-line summary. The ⓘ hint (`Key::HintStrategy`) is on the
section heading in Expert mode; expanded inline in Guided mode.

Mode-aware: in Guided mode, the strategy section starts collapsed behind
"More rule-set options" (a new `rule_set_config_more: bool` toggle, default
false, reset when mode switches to Guided).

### Rule editor — per-rule weight / priority field

When the active strategy is `WeightedRandom`:
- A "Weight" number input appears below the RESPOND column.
- Empty = 1 (default weight).
- Range: unsigned integer.

When the active strategy is `Priority`:
- A "Priority" number input appears instead.
- Empty = 0 (default priority, lowest).
- Range: any integer (negative allowed for deprioritised rules).

In Guided mode (layout density): these fields are hidden when
`rule_when_more = false`, since they are advanced. They appear automatically
when the user expands "More matching criteria".

### Validation surfacing in the rule editor

When the rule's `validation.issues` is non-empty, a warning strip appears
at the top of the rule editor (above WHEN/RESPOND), listing the issue
messages. The mock data already produces "WeightedRandom is selected, but
this rule has no weight set." — this is the primary demonstration case.

## State additions

```rust
// App
pub rule_set_config_more: bool,  // Guided: strategy section expanded
```

Reset to `false` when `ChooseAudienceMode(Guided)`.

## Messages

```rust
RuleSetSetStrategy(Strategy),          // strategy dropdown changed
RuleWeightChanged(String),             // weight input changed
RulePriorityChanged(String),           // priority input changed
ToggleRuleSetConfigMore,               // Guided mode expand/collapse
```

## Acceptance criteria

- Rule set config shows a strategy dropdown with all five variants.
- Selecting the strategy updates `root_settings.strategy` (global, for
  the mockup).
- `Strategy::help()` appears as a description beside the dropdown.
- When strategy is WeightedRandom, each rule editor shows a weight input;
  when Priority, a priority input; other strategies show neither.
- Validation issues on a rule are shown as a warning strip above WHEN/RESPOND.
- In Guided mode, the strategy section starts collapsed and the
  weight/priority fields follow `rule_when_more`.
- In Expert mode, everything is always visible.
- Zero errors, zero warnings, existing + new tests pass.
