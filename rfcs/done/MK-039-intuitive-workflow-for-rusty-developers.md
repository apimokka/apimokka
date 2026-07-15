# RFC MK-039 — Intuitive workflow without relabeling

**Status.** Implemented (v0.8.0)
**Tracks.** Accessibility/comfort tokens, friendly error model, async-ready
task shape, disabled-action reasons, inline concept hints, non-modal undo.
**Touches.** `theme.rs` tokens, `apimokka-model` (new `friendly_error.rs`),
`message.rs`, `app.rs`, all screens, `widgets/mod.rs`, i18n.
**Refines.** MK-022 (design system), MK-035 (state models), MK-036 (microcopy).

## Context

A UI/UX review proposed reframing apimokka for a non-technical audience by
renaming the domain vocabulary (workspace→project, rule→saved reply,
JSON→reply text, header→extra detail, 404→"no saved reply matched") and
hiding the HTTP surface behind "More options".

**The persona has not changed.** apimokka targets backend/frontend developers
and QA engineers who run `apimock-rs`. For them, HTTP vocabulary is not
jargon to be hidden — it is the working language. Renaming "status code" to
"result number" forces a translation step and *increases* cognitive load.

However, one real-world shift is acknowledged: with AI assistance, developers
are often less fluent in fundamentals they don't touch daily. The response is
to make the **workflow** more intuitive and the interface more **teaching and
reassuring** — through structure, feedback, and in-context help — while
keeping the domain vocabulary intact.

This RFC adopts the review's sound *mechanics* and rejects its *relabeling*.

## Adopted vs rejected

| Adopted (mechanics) | Rejected (relabeling) |
|---|---|
| Larger type/touch defaults (16 px body, 52 px primary, 44 px min target) | Renaming workspace/rule/trace/JSON/header/body/status code |
| `FriendlyProblem` error model | Blocked-technical-terms build test |
| Async-ready task shape (instant feedback, keep data on failure) | "Local helper" euphemism for the server |
| Disabled-action reasons (no dead control without a visible "why") | Hiding the HTTP surface behind "More options" |
| Non-modal undo for reversible actions | Parallel `apimokka-copy` crate (we keep `apimokka-i18n`) |
| Inline concept hints (behind ⓘ) | — |

## 1. Comfort & accessibility tokens

`size::BODY` 14 → 16; `size::CAPTION` stays 12 (metadata); `size::SECTION`
17 → 18; `size::TITLE` 22 → 24; `size::DISPLAY` 32 → 36. New `touch` module:
`MIN = 44.0`, `COMFORTABLE = 52.0`. Primary buttons adopt a 52 px min height.

Token *names* and the theme-derived color system are unchanged — only values
retune. WCAG AA: 16 px body and 44 px targets meet the comfort floor.

## 2. FriendlyProblem error model

Lives in `apimokka-model::friendly_error` (pure; no iced). Developer-technical
content — names the real cause and the concrete fix.

```rust
pub struct FriendlyProblem {
    pub title: String,
    pub detail: String,
    pub action_label: Option<String>,   // e.g. "Open Settings"
}
```

Examples (developer register):
- Port conflict: "Port 8080 is already in use" / "Another process holds the
  port (EADDRINUSE). Stop it, or change the listener port in Settings." /
  "Open Settings"
- Save failure: "Save failed" / "Could not write to the rule set file. Check
  file permissions or choose another path." / "Retry"

## 3. Async-ready task shape (MK-035 refinement)

Fallible work returns `Result<_, FriendlyProblem>` and is dispatched via
`iced::Task::perform`. The reducer:
1. updates visible state immediately ("Saving…", disable repeat),
2. keeps the previous good data visible,
3. on completion shows "Saved" or the FriendlyProblem.

The mockup has no real I/O, so tasks resolve immediately, but the message
shape (`SaveFinished(Result<…>)`, `HelperStarted(Result<…>)`) is the
production-correct one.

## 4. Disabled-action reasons

No primary action is rendered as a dead button. When an action cannot proceed,
the control is disabled AND a one-line reason appears next to it
(e.g. Save disabled → "Enter a URL path first"). A helper
`widgets::action_with_reason(label, msg_if_ready, reason_if_blocked)` enforces
this.

## 5. Inline concept hints (behind ⓘ)

Technical fields carry a small ⓘ info affordance. Hovering/clicking reveals a
short hint in domain language that teaches the exact gotcha:
- Body path: "Matches the JSON request body by dotted path (`user.id`,
  `items.0.sku`). Not JSONPath — `$.foo` won't work."
- URL path operator: "How the incoming path is compared: Equal, StartsWith,
  Contains, …"
- Strategy: "How a winner is chosen when multiple rules match."

Hints are opt-in (behind ⓘ), so they never clutter the default view — honoring
"less is more" while teaching the rusty developer in place.

## 6. Non-modal undo

Reversible actions (delete rule) perform immediately and surface a transient
"Deleted — Undo" notice instead of a pre-emptive confirm dialog. Irreversible
or high-blast-radius actions (delete rule set, discard all, revert file) keep
the confirm dialog.

## State additions

```rust
// App
pub last_problem: Option<FriendlyProblem>,   // friendly error banner
pub undo: Option<UndoEntry>,                 // transient undo
pub notice: Option<String>,                  // transient success/info
```

`UndoEntry` captures enough to restore a deleted rule (rule payload + parent +
index).

## Acceptance criteria

- Body text is 16 px; primary buttons ≥ 52 px; min target 44 px.
- No primary action is a dead button without a visible reason.
- Every fallible path returns a `FriendlyProblem`; no raw error strings reach
  the UI.
- Deleting a rule shows a non-modal Undo that restores it.
- ⓘ hints exist on body-path, URL-operator, and strategy controls; default
  view is uncluttered.
- Domain vocabulary unchanged (workspace, rule, trace, JSON, header, …).
- Zero errors, zero warnings; existing + new tests pass.

## Out of scope

- Real I/O (mockup keeps in-memory state).
- Screen-reader/ARIA work (separate future RFC).
- Mobile/narrow-screen flow.
