# RFC MK-015 — Accessibility, keyboard, and command palette contract

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Accessibility / keyboard navigation / command palette.
**Touches.** `main.rs`, `app.rs::subscription`, all screens

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| All primary workflows can be completed without a pointer | ⚠️ Partial | Global keyboard subscription added (Esc, Cmd+K/Ctrl+K); tab-order not verified |
| Dynamic updates do not steal focus | ⚠️ Untested | iced focus not explicitly managed |
| Status and validation information available without colour | ✅ | Non-colour status matrix (full table in v0.1.0 RFC) |
| Command palette exposes major actions | ✅ | Dialog with ~12 commands + text filter |
| Accessibility checklist in review package | ❌ | Not adapted from source RFC package |

## Keyboard subscription (new in v0.2.0)

`App::subscription()` uses `iced::keyboard::listen()` (from
`iced_futures::keyboard`) to receive `keyboard::Event::KeyPressed`:

```rust
keyboard::listen().map(|event| { … Message::EscapePressed / ToggleCommandPalette / Noop })
```

`Message::EscapePressed` closes the topmost overlay in priority order:
1. Destructive-action confirmation
2. Command palette
3. Test-rule dialog
4. Dotted-path assistant
5. Bottom drawer

`Cmd+K` / `Ctrl+K` (platform-aware via `modifiers.command()`) toggles the
command palette.

Wired via `.subscription(App::subscription)` on the iced application builder.

## Non-colour status (unchanged from v0.1.0)

See v0.1.0 RFC for the full glyph table.

## Remaining gaps

- Tab order not verified against element creation order
- No focus ring styling (iced 0.14 `focused` style hook not used)
- No accessibility review checklist
- `Alt+Up/Down` for rule movement not wired (MK-006 keyboard gap)
