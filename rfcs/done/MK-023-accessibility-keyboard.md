# RFC MK-023 — Accessibility and keyboard contract

**Status.** Implemented (v0.6.0)
**Tracks.** ABDD policy, keyboard shortcut catalogue, focus order, screen-reader landmarks.
**Touches.** Every interactive surface.
**Supersedes.** MK-015 (accessibility, keyboard, command palette).

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
