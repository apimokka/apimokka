# RFC MK-030 — Settings

**Status.** Implemented (v0.6.0)
**Tracks.** S-13 Settings screen.
**Touches.** Workspace and server configuration.
**Supersedes.** Settings portion of MK-003 (TLS/Log) and MK-010 (strategy).

## Summary

A sectioned form for the rare configuration changes the user makes once during setup and seldom thereafter. The screen's main UX commitment is **honesty about change impact**: each section states up front whether changes save instantly, require reload, or require restart.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Settings                                                    │
│                                                             │
│ General · Save only                                         │
│ ┌ Workspace name [payments-mock]                          ┐ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ Server · Restart required after changing host, port, or TLS│
│ ┌ Host [127.0.0.1]  Port [8080]  TLS [Off]               ┐ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ Logs · Restart or reload depending on field                 │
│ Trace · Reload                                              │
│ Strategy · Reload                                           │
└─────────────────────────────────────────────────────────────┘
```

The screen is a single scrolling column with section cards stacked vertically. Each card is a `surface.panel` card with `space.4` internal padding.

## Sections

| Section | Fields | Change impact |
|---|---|---|
| General | Workspace name, path (read-only display) | Save only |
| Server | Host, port, TLS toggle, TLS cert path, TLS key path | Restart |
| Logs | Log file path, log level (Trace · Debug · Info · Warn · Error) | Log file: restart. Log level: reload. |
| Trace | Enable trace, transport (UDS / TCP), socket path / address, queue size | Reload (may need restart if transport changes) |
| Strategy | Default rule-selection strategy: First match · Weighted random · Priority · Round robin (if supported by apimock-rs version) | Reload |

## Impact disclosure rule

Every section header explicitly states the change impact **before the user edits**:
- "Save only" — change applies immediately when saved
- "Reload" — the running server must reload its config (a single click)
- "Restart" — the running server must be stopped and started again

Once the user makes a change that introduces a stronger requirement (e.g. they were just changing log level — "reload" — but then also changed port — "restart"), a notice at the bottom of the section explains the upgrade: "Changing port requires restart, which overrides the reload-only level change."

## Action flow

The screen does not need its own Save button — settings save through the global top-bar Save flow. After save, the top-bar chip flips to `Reload pending` or `Restart required` based on what changed.

A persistent **footer bar** inside the Settings tab shows:
- "All changes saved" (when clean)
- "Unsaved changes — Save to apply" + `[Save]` (when dirty)
- "Reload required to take effect" + `[Reload now]` (after save, when reload-class settings were touched)
- "Restart required to take effect" + `[Restart now]` (after save, when restart-class settings were touched)

The footer bar is sticky at the bottom of the Settings content area.

## Validation

- Port: must be a number between 1 and 65535
- TLS cert / key paths: required when TLS is enabled; checked on save
- Log file path: must be writable; checked on save
- Trace queue size: must be a positive integer

Inline errors appear under the relevant field; the global Save button stays disabled if any field has an error.

## Visual treatment

- Section headers use `section` token; the impact disclosure text uses `caption` token in `text.secondary`.
- Field labels are above inputs (consistent with MK-022 component rules).
- The currently-focused section's card gets a subtle border-left accent in `accent.primary` for keyboard users.

## Acceptance criteria

- Every section displays its change-impact label.
- Saving with a restart-class change updates the top bar chip to `Restart required` immediately after save.
- The footer bar shows the right call-to-action based on save+reload+restart state.
- All inputs have above-field labels; no placeholder-only fields.
- The screen renders correctly in light and dark themes.

## Out of scope

- Per-rule-set strategy (this is set in the rule-set TOML; out of v1 Settings)
- Workspace-level secrets (TLS material) — referenced by path, not embedded
- Hot-reload monitoring of disk changes (a v2 RFC will cover external-edit detection)
