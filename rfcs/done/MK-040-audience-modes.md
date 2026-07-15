# RFC MK-040 — Audience modes: Guided and Expert

**Status.** Implemented (v0.9.9)
**Tracks.** A user-chosen presentation density (Guided / Expert) that adapts
scaffolding without ever renaming the domain. First-run mode picker; persisted
user setting; mode-aware content layer.
**Touches.** `App` state, `message.rs`, first-run flow, Settings, shell
feedback banner, condition cards, `FriendlyProblem` rendering, i18n.
**Refines.** MK-039 (intuitive workflow), MK-022 (design system), MK-030
(settings). Supersedes nothing.

## Why

As AI takes over implementation work that skilled engineers used to do, the
person configuring a mock server is increasingly *not* the person who could
hand-write the TOML. "Technical business, non-technical user" becomes common.
apimokka should serve both the expert and the newcomer from one product.

The naïve response — relabel the domain into plain words (RFC MK-039 rejected
this) — backfires, because the user does not stay inside apimokka. They paste
errors into AI chats, read apimock-rs docs, and ask colleagues; all of those
speak the real vocabulary. Teaching a private vocabulary ("reply text" for
JSON) disconnects the user from the ecosystem they must operate in.

## Principle

**Guided mode adds scaffolding; it never renames the domain.**

The scaffold's job is to *bridge the user toward the real concept*, not to
shield them from it. A good Guided hint makes the user **less** dependent on
Guided mode over time. Every scaffolding decision is held to that test.

Rejected alternatives:
- *Two parallel vocabularies* — manufactures the ecosystem disconnect above.
- *Single adaptive view (no explicit mode)* — removes user control; an expert
  stuck in an inferred newcomer view has no recourse. An explicit, remembered
  choice respects the user's own judgment about where they are.

## The two modes

| Aspect | Expert | Guided |
|---|---|---|
| Vocabulary | identical | identical |
| ⓘ concept hints | available on hover | the same hints, expanded inline by default |
| Field sublabels | term only (`Body`) | term + plain gloss (`Body — the JSON sent with the request`) |
| Errors | title + technical detail inline | title + plain line; technical detail behind "Show details" |
| Layout | full density | common 80% first; advanced behind "More" (future) |
| First-run | — | the default offered to self-identified newcomers |

Both modes show the same words, the same screens, the same data. Guided only
changes *how much explanatory scaffolding is visible by default*.

## First-run mode picker

On first launch with no stored preference, present a one-question chooser:

```
How would you like apimokka to guide you?

  ( ) Guided     Show extra explanations as you work.
                 Best if HTTP mocking is newer to you.

  ( ) Expert     Compact view, no extra explanations.
                 Best if you already know your way around.

  You can change this any time in Settings.
                                              [ Continue ]
```

The choice writes to a persisted user setting (`AudienceMode`). Settings →
Appearance gains a matching control so it is reversible at any time.

## Data model

```rust
// apimokka-model (pure): a user-facing preference, no UI dependency.
pub enum AudienceMode { Guided, Expert }
```

`App` gains `audience_mode: Option<AudienceMode>` — `None` means "not yet
chosen" and triggers the first-run picker; `Some(_)` is the active mode.

### Mode-aware content layer

`FriendlyProblem` gains an optional `technical_detail`:

```rust
pub struct FriendlyProblem {
    pub title: String,
    pub detail: String,           // plain, always shown
    pub technical_detail: Option<String>,  // errno/stack; Expert inline, Guided behind "Show details"
    pub action_label: Option<String>,
}
```

Concept hints gain a render mode: in Guided they render expanded inline; in
Expert they render as the ⓘ marker only. The hint *text is identical*.

## Acceptance criteria

- First launch with no stored mode shows the picker; choosing writes the
  setting; subsequent launches skip the picker.
- Settings exposes the mode and switching it takes effect immediately.
- Guided expands hints inline and shows plain glosses; Expert shows ⓘ only.
- Vocabulary is byte-for-byte identical between modes (a test asserts no
  Guided-only relabeling of domain terms).
- `FriendlyProblem.technical_detail` is shown inline in Expert and collapsed
  in Guided.
- Zero errors, zero warnings; new + existing tests pass.

## Phasing

1. **This RFC + model + state + toggle + first-run picker** (v0.9.0).
2. Mode-aware errors (`technical_detail`) and inline-expanded hints.
3. Layout density (common-first, advanced-behind-More) — later, per screen.

Scaffolding must always be dismissible and must teach the real concept, never
replace it. No tutorials/coach-marks (violates "less is more").

## Out of scope

- Inferring mode from behaviour (explicit choice only).
- Per-screen layout density (phase 3).
- Real persistence to disk (mockup keeps the setting in memory; production
  writes it to a preferences file).
