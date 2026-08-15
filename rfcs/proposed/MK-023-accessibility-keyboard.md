# RFC MK-023 — Accessibility and keyboard contract

**Status.** Proposed
**Tracks.** ABDD policy, keyboard shortcut catalogue, focus order, screen-reader landmarks.
**Touches.** Every interactive surface.
**Supersedes.** MK-015 (accessibility, keyboard, command palette).

> ## Status correction, 2026-08-15
>
> **Returned from `done/`.** Recorded as `Implemented (v0.6.0)`; the keyboard
> contract below was never built. Corrected by project-owner decision.
>
> **What is not implemented.** There is no `Named::Tab` handler, no
> `focus_next`/`focus_previous`, and no `widget::operate` anywhere in
> `crates/app/src`. iced does not traverse focus on its own — the application
> must wire it, and this one does not. Keyboard handling is a **global**
> subscription (`app.rs:3418`): `Escape` plus a six-entry accelerator table
> (Undo, Redo ×2, Save, ReloadConfig, ToggleCommandPalette). Nothing else.
>
> **Consequence.** A keyboard-only user cannot pass the application's **first
> screen**. `Message::ChooseAudienceMode` fires only from a button `.on_press`
> (`screens/mode_picker.rs:32`), it is not in the accelerator table, and
> `view()` short-circuits to the mode picker while `audience_mode.is_none()`.
> This is more severe than the shipped `high_contrast_dark` modal-dim defect
> M8 exists to fix, which at least degraded a usable application.
>
> **Line 93 of this RFC — `- [ ] Main workflows are keyboard reachable` — is
> unticked in the shipped document.** The gap was visible in the record and
> read past.
>
> **How it was found.** Attempting to drive the app with `wtype` for M8's
> capture. The tool works; the application does not accept the input. Evidence:
> `.git-exclude/reviewed/2026-08-15-wtype-probe-result-and-keyboard-contract-gap.md`.
>
> **The satisfiable path is this RFC's own wording.** Line 17 requires every
> control reachable *"via Tab **or the command palette**"*. Full Tab traversal
> may not be achievable on iced 0.14 — MK-056 already lists iced-imposed
> accessibility gaps as a non-goal (its decision 6). A fully keyboard-operable
> palette satisfies the contract without it, and that is MK-033's unimplemented
> requirement rather than new design. See MK-033's matching correction.
>
> **This is the third RFC found in `done/` whose central mechanism was never
> built**, after MK-024 (breakpoints) and MK-033 (palette keyboard operation).
> The common factor is that nothing in this programme executed the UI until
> `scripts/ux/` existed; all three were verified against design intent.
>
> The design below is unchanged and remains the intended target.

## Summary

apimokka commits to **Accessibility By Default Design** (ABDD): the app is usable by keyboard, by screen reader, at 200% text scale, in light or dark mode, with no information conveyed by colour alone. This RFC defines the contract every other screen RFC must honour.

## ABDD requirements

| Requirement | Design rule |
|---|---|
| Status clarity | All status indicators use **icon + text label** |
| Keyboard reachability | Every control is reachable via Tab or the command palette |
| Focus visibility | Focus ring is high-contrast in both themes |
| Non-colour signalling | Match/miss/error cannot rely only on colour |
| Screen-reader naming | Icon buttons have accessible labels |
| Font scaling | Layout tolerates 200% text scaling without clipping |
| Locale expansion | JA labels fit without clipping (typically 10–30% wider than EN) |
| Motion sensitivity | Animation is never required for understanding |

## Keyboard shortcut catalogue

| Shortcut | Action | Available in |
|---|---|---|
| `Esc` | Close topmost overlay (dialog → drawer) | Any |
| `Ctrl/Cmd + K` | Toggle command palette | Any workspace view |
| `Ctrl/Cmd + S` | Save workspace | Workspace |
| `Ctrl/Cmd + R` | Reload config | Workspace when reload available |
| `Ctrl/Cmd + Enter` | Run Test Rule | Test Rule dialog focused / rule editor |
| `Tab` / `Shift + Tab` | Move focus forward / backward | Any |
| Arrow keys | Navigate lists, trees, segmented controls | Tree / segmented contexts |
| `Enter` | Activate focused **non-danger** primary action | Any |
| `Space` | Toggle checkbox / button where appropriate | Any |

Notes:
- `Enter` is intentionally restricted to non-danger primary actions to prevent accidental destructive activation. Danger actions require explicit focus on the danger button.
- The command palette is the canonical fallback for any action that lacks a dedicated shortcut. Adding a shortcut is preferred when it serves a frequent workflow; otherwise the palette suffices.

## Focus order — Routes screen (the most-used view)

```
Top bar identity (workspace switcher)
→ Server status / global actions
→ Left rail
→ Rule-set tree
→ Add rule set
→ Rule editor: URL operator
→ Rule editor: URL path
→ Method controls
→ Header rows (each row: name → op → value → row actions)
→ Body rows (each row: path → op → value → row actions)
→ Respond controls (mode tabs → status → delay → body)
→ Test rule
→ Right column (trace strip or rule inspector)
→ Bottom drawer trigger
```

Focus order on Trace, Settings, Scripts and overlays follows the same top-to-bottom, left-to-right reading order with no surprises.

## Screen-reader landmarks

| Region | Landmark label |
|---|---|
| Top bar | Workspace actions |
| Left rail | Main navigation |
| Rule-set tree | Rule sets and files |
| Rule editor | Rule editor |
| Live trace strip | Recent requests |
| Bottom drawer | Validation and save details |
| Dialog | The dialog's title |

Each landmark must be addressable by an iced accessibility role hint or, where iced lacks the affordance, a descriptive `aria`-equivalent annotation provided to snora.

## Discoverability of shortcuts

The command palette header displays the active shortcuts (`Ctrl/Cmd + K toggle`, `Esc close`) as chip annotations. Individual rows in the palette may also show their shortcut where one exists. New users who never read a tutorial will discover the keyboard surface via the palette.

The palette is opened from a visible button in the top bar (with the shortcut visible in its tooltip).

## Empty-state and error voice

Both error and empty-state copy must avoid blame and unhelpful generic language. The microcopy contract (MK-036) details the exact patterns; this RFC requires only that they be applied.

## Accessibility checklist (for every PR)

- [ ] Every icon-only button has an accessible label.
- [ ] Every status has text + icon.
- [ ] Focus is visible in light and dark mode.
- [ ] Main workflows are keyboard reachable.
- [ ] `Esc` closes the topmost overlay.
- [ ] `Ctrl/Cmd + K` opens the command palette.
- [ ] Colour is not the only severity/outcome signal.
- [ ] JA strings do not clip when the locale is switched.
- [ ] Text can scale to 200% without clipping.

## Acceptance criteria

- The shortcut catalogue is implemented in a single keyboard subscription module that maps key events to messages.
- A test (manual or automated) confirms every catalogue entry actually works.
- The palette is reachable from every workspace view.
- A locale-switch smoke test confirms JA strings do not clip any of the listed surfaces.

## Out of scope

- Custom focus-ring styling — handled in MK-022 component rules (the theme picks the ring colour; this RFC only requires it be visible).
- Specific shortcut conflicts on JA / non-US keyboards — flagged as a v2 follow-up.
