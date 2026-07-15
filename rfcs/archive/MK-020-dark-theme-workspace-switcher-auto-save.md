# RFC MK-020 — Dark theme, workspace switcher, auto-save, styled buttons

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Remaining UX deferred items + visual completeness.
**Touches.** `theme.rs`, `app.rs`, `shell/top_bar.rs`, `shell/view.rs`,
`screens/workspace_menu.rs` (new), `screens/command_palette.rs`,
`screens/confirm_dialog.rs`, `screens/welcome.rs`, `screens/wizard.rs`,
`widgets/mod.rs`, `message.rs`

## Changes

### 1. Dark theme toggle

- `ThemeChoice { Light, Dark }` enum on `App`. Default: `Light`.
- `App::theme()` dispatches to `ThemeChoice::iced_theme()` —
  `Theme::Light` or `Theme::Dark`.
- Top-bar button (☾/☀) fires `Message::ToggleTheme` to toggle between
  modes. The glyph hints at the *target* state ("click ☾ to go dark").
- `Esc` now also closes the workspace switcher dropdown (added to the
  priority chain in `EscapePressed`).

**Dark-mode-aware style adjustments in `theme.rs`:**
- `card_style`: uses `background.weak.color` in dark mode (slightly
  elevated above the base surface) and `background.base.color` (white)
  in light mode.
- `card_selected_style`: 18% primary alpha in dark mode (vs 10% light)
  so the selection tint is visible against a dark surface.
- `muted_text`: changed from a theme-relative 60%-alpha to a fixed
  mid-grey (`0.52, 0.52, 0.52`). Reads well on both light and dark
  backgrounds without needing a theme reference in every call site.
  This eliminates the 21 hardcoded `&iced::Theme::Light` arguments
  that would have shown wrong colours in dark mode.

### 2. Workspace switcher in top bar

- `workspace_menu_open: bool` field on `App`.
- The workspace identity in the top bar (app name + workspace name +
  chevron ▼/▲) is now a `button` that fires `Message::ToggleWorkspaceMenu`.
- `screens/workspace_menu.rs` — a dropdown rendered via snora's
  `AppLayout::header_menu(element)` slot (appears below the header,
  dismisses on outside click via `on_close_menus`). Shows:
  - Current workspace name + path
  - Recent workspaces list (click to switch)
  - "Open workspace" and "New workspace" action buttons
- `Message::CloseWorkspaceMenu` also wired to `Esc`.

### 3. Auto-save for rule content edits

- `App::with_selected_rule()` now calls `simulate_save()` after every
  rule edit (URL/method/headers/body/respond).
- Effect: the top-bar Save chip always shows "✓ Saved" after a rule
  edit; the user no longer needs a manual Save click for day-to-day
  rule tweaking.
- Top-bar Save chip semantics updated: shows `⏻ Unsaved` only when
  `ServerState::RestartRequired` (i.e. a listener/TLS/log-file setting
  was changed and hasn't been applied). All other states show `✓ Saved`.
- The Save button remains for explicit saves; it becomes disabled when
  nothing is dirty.
- Settings changes that require restart still need explicit "Restart"
  action (unchanged from v0.4).

### 4. Styled buttons — primary, secondary, danger, ghost

New helper functions in `widgets/mod.rs`:

| Helper | iced style | Use |
|---|---|---|
| `primary_btn(label, msg)` | `button::primary` | Hero actions (Open workspace, Create workspace, Wizard Create) |
| `secondary_btn(label, msg)` | `button::secondary` | Alternative actions alongside a primary |
| `danger_btn(label, msg)` | `button::danger` | Destructive confirmation ("Proceed") |
| `ghost_btn(label, msg)` | `button::text` | Cancel / dismiss in dialogs |

Using iced's built-in style functions means they automatically adapt to
`Theme::Dark` using the theme's danger/primary/secondary palette entries.

Applied to:
- **Welcome**: Open (primary), Create (secondary)
- **Wizard sticky bar**: Cancel (ghost), Create (primary)
- **Confirm dialog**: Cancel (ghost), Proceed (danger) — visually
  communicates that the right is destructive without colour alone (the
  button positions and labels carry the semantic; colour is additive)

### 5. Keyboard shortcut hints in command palette

- Command palette header shows `⌘K toggle` and `Esc close` shortcut
  chips using a new `shortcut_hint(key, action)` helper rendered as
  `chip_style` pills.
- Two new palette commands: "Toggle dark mode" and "Switch workspace".
- `ThemeChoice` and `workspace_menu_open` are both accessible via
  the palette, so keyboard-only users can reach every feature.

## Acceptance criteria status

| From MK-015 (keyboard) | Met? |
|---|---|
| Primary workflows reachable without a pointer | ✅ (⌘K for palette, Esc for all overlays + workspace menu) |
| Keyboard shortcut hints visible | ✅ (palette header + command rows) |

| From MK-018 (UX deferred list) | Met? |
|---|---|
| Auto-save with restart-needed-only prompt | ✅ |
| Workspace switcher in top bar | ✅ |
| Keyboard shortcut hints in command palette | ✅ |

| From MK-019 (dark theme deferred) | Met? |
|---|---|
| `Theme::Dark` works without colour artefacts | ✅ |

## Still deferred

- "Remember last workspace" on launch (needs persistence)
- True per-section collapse in wizard
- Undo / redo history (MK-016 partial)
- Animation on dialog/drawer open
- Custom button hover state (density / active ring)
