# RFC MK-037 — iced/snora implementation handoff

**Status.** Implemented (v0.6.0)
**Tracks.** Mapping the design onto iced 0.14 + snora 0.8 primitives; component boundaries; data-to-view mapping; what iced cannot do.
**Touches.** The entire implementation.
**Supersedes.** MK-017 (packaging — implementation portion).

## Summary

The redesign expressed in MK-021..MK-036 must be implementable in iced 0.14 with snora 0.8 as the shell. This RFC translates the abstract design into concrete primitive choices, component boundaries, and capability constraints.

It exists so designers don't propose patterns iced can't render natively (backdrop blur, CSS hover-only popovers, complex animations) and so implementers have a starting decomposition.

## Snora primitive mapping

| Design concept | snora primitive |
|---|---|
| Top bar | `Header` |
| Left rail | `Sidebar` |
| Body | `Body` (per-tab content) |
| Bottom drawer | `Sheet` at `SheetEdge::Bottom` |
| Dialogs (Test rule, Dotted-path, Confirm) | `Dialog` |
| Workspace menu dropdown | `header_menu` slot of `AppLayout` |
| Outside-click dismiss | `on_close_menus`, `on_close_modals` callbacks |

Snora 0.8's `AppLayout` is the root composition. The Workspace shell builds one `AppLayout` per render and adds the body, sheet, dialog, and header-menu pieces conditionally.

## What iced 0.14 can do well

- Cards, panels, rounded corners, drop shadows (`container::Style`)
- Custom fonts (load TTF/OTF), arbitrary text sizes
- Text inputs, buttons, checkboxes, radios, pick-lists, sliders
- Scrollable regions, padding, alignment, row/column composition
- Inline SVG (limited path complexity), drawn shapes via canvas
- Light + Dark theme with extended palette (`primary`/`secondary`/`danger`/`success`/`warning` families)
- Container shadows with offset/colour/blur, border radius, alpha-blended backgrounds

## What iced 0.14 cannot easily do

- True freeform vector graphics alongside widgets
- CSS-level transitions / smooth animations (there's an animation API but it's separate and the team will not wire it in v1)
- Backdrop blur
- Complex gradients beyond a single linear/radial fill
- Web-style hover popovers (snora's `header_menu` is the dropdown pattern available)
- Nested scroll traps (don't even try)

Designers should not propose features that depend on these capabilities. If a design requires one, it's a v2 candidate or needs an alternative.

## Recommended component boundaries

```
AppShell
  TopBar
    WorkspaceIdentityMenu          (button → header_menu)
    StatusChipGroup
    GlobalActionGroup              (Save / Reload / Restart / Start/Stop)
    ViewControls                   (trace toggle / theme toggle / palette)
    LocalePicker
  LeftRail
  WorkspaceBody
    RoutesScreen
      RouteTreePanel               (left sidebar: rule sets / fallback / scripts)
      RuleEditorPanel              (centre: WHEN | → | RESPOND)
      RightColumn
        LiveTracePanel  |  RuleInspectorPanel
    TraceScreen
      TraceFilterBar
      TraceEventList
      MatchDetailPanel
    ScriptsScreen
      ScriptList
      ScriptViewer
    SettingsScreen
      SettingsSectionCard (xN)
      SettingsFooterBar
  BottomDrawer                     (snora Sheet, two modes)
    ValidationDrawer
    SaveDiffDrawer
  DialogHost                       (priority-ordered)
    ConfirmDialog
    WorkspaceMenu (header_menu, not a true dialog but lives in this layer)
    CommandPalette
    TestRuleDialog
    DottedPathAssistant
```

Each component is a Rust module under `crates/apimokka-app/src/`. Components communicate by dispatching `Message` variants; no shared mutable state outside `App`.

## Data-to-view mapping

| Domain data | Primary UI surface |
|---|---|
| Workspace | Top bar identity; Dashboard recent row |
| Rule set | Routes sidebar group |
| Rule | Sidebar row + rule editor centre |
| Match conditions | WHEN cards |
| Respond | RESPOND card |
| Fallback file | Sidebar fallback section + Trace fallback outcome |
| Middleware script | Sidebar scripts section + Scripts screen |
| Trace event | Routes trace strip + Trace screen row |
| Validation issue | Sidebar rule indicator + validation drawer |
| Server state | Top bar chip |
| Save state | Top bar chip + save diff drawer |

The mapping is intentionally simple. One piece of domain data shows up in at most three places, and the user can navigate between them.

## Style helper layout

The implementation exposes (in `crate::theme`):

- `space::{XS, SM, MD, LG, XL, XXL}` — spacing constants (`f32`)
- `size::{CAPTION, BODY_SM, BODY, SECTION, HEADING, HERO}` — type sizes (`f32`)
- `radius::{SM, MD, LG, PILL}` — border radii (`f32`)
- `pad::{BUTTON, BUTTON_PRIMARY, CARD, CHIP}` — `[v, h]` arrays
- `card_style`, `card_selected_style`, `chip_style`, `panel_style`, `header_style`, `hairline_style`, `banner_style` — `Theme → container::Style` functions
- `muted_text(theme) -> Color` — secondary text colour
- `severity_tint(theme, severity) -> Color` — severity → colour

Helper functions (`primary_btn`, `secondary_btn`, `ghost_btn`, `danger_btn`, `icon_btn`, `field`, `severity_badge`, `divider`, `empty_state`) live in `crate::widgets`.

The visual designer hands off the colour palette + token values; the implementer maps those into the constants above.

## Anti-patterns to avoid

- Hardcoded magic numbers in screen code (use tokens)
- `iced::Theme::Light` hardcoded in helper functions (use `App::theme()`)
- Nested cards more than one level deep
- Hover-only critical interactions (mobile/keyboard users can't reach them)
- Long inline `container().style(|t| { ... })` closures (factor into `theme.rs`)
- Cross-screen `Message` reuse for unrelated actions (introduce distinct variants)

## Workspace structure

```
crates/
  apimokka-model/      Pure data + state machines. No iced types.
  apimokka-i18n/       Translation tables; one Key enum, EN + JA tables.
  apimokka-app/        iced binary; the screens and widgets live here.
```

The model crate has no dependency on iced; it can be unit-tested cleanly. The i18n crate has no dependency on iced either.

## Testing surface

| Test | Where |
|---|---|
| State machine transitions | `apimokka-model` unit tests |
| i18n key coverage (every Key is translated in both EN and JA) | `apimokka-i18n` compile-time match |
| Screen rendering at reference window sizes | Manual smoke test |
| Keyboard shortcuts catalogue | Integration test (single test per shortcut) |
| Locale-switch smoke test | Manual |

iced does not have a built-in headless renderer, so visual tests are out of scope; rendering correctness is a manual review item.

## Acceptance criteria

- Every snora primitive used appears in this RFC's mapping table.
- Component boundaries match the tree above; no module sprawls beyond its assigned responsibility.
- No magic numbers for size/spacing/radius in screen code.
- The workspace crate split holds (model has no iced; app holds the iced).
- All RFCs from MK-021..MK-036 reference this RFC for their iced/snora implementation footprint.

## Out of scope

- Build / packaging (a separate RFC)
- CI configuration
- Release process
