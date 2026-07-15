# RFC MK-026 — New workspace wizard

**Status.** Implemented (v0.6.0)
**Tracks.** S-02 wizard.
**Touches.** New-workspace creation flow.
**Supersedes.** Wizard portion of MK-002.

## Summary

A single-page form for creating a workspace. Required fields are always visible; advanced sections (Server defaults, Starter content, Trace) are collapsed by default with sensible defaults pre-applied so the user can hit **Create workspace** without expanding anything.

The wizard replaces the prior 5-step sequential flow: most users only need a name and a folder, and the Next/Next/Next/Next pattern made that effort feel longer than it was.

## Design principle

**Single page, progressive disclosure.** Sections beyond the required block are collapsed; expanding any of them shows its inputs inline.

## Layout

```
┌──────────────────────────────────────────────────────────────┐
│ Create workspace                                             │
│                                                              │
│ Workspace                                                    │
│ Name              [ payments-mock                          ] │
│ Parent folder     [ /Users/me/dev                          ] │
│                                                              │
│ ▸ Server defaults                                            │
│ ▸ Starter content                                            │
│ ▸ Trace                                                      │
│                                                              │
│                                  [Cancel] [Create workspace] │
└──────────────────────────────────────────────────────────────┘
```

Each section's chevron (`▸` / `▾`) indicates collapse state. Clicking the section header (or its chevron) toggles. Tab-focusing the header and pressing Enter or Space also toggles.

## Fields

| Section | Field | Default | Required |
|---|---|---|---|
| Workspace | Name | empty | yes |
| Workspace | Parent folder | last used folder, or `$HOME` | yes |
| Server | Host | `127.0.0.1` | no |
| Server | Port | `8080` (or next available) | no |
| Server | TLS | Off | no |
| Server | TLS cert/key paths | empty (shown when TLS is on) | yes if TLS on |
| Starter content | Template | `Basic REST + error samples` | no |
| Starter content | Include delay sample | off | no |
| Trace | Enable trace | On | no |
| Trace | Queue size | `1024` | no |

## Validation

| Condition | UI response |
|---|---|
| Missing name | Inline error under the Name field; **Create workspace** disabled |
| Invalid parent folder | Inline error under the Parent folder field; **Create workspace** disabled |
| Port out of range or non-numeric | Inline error under Port; **Create workspace** disabled |
| Folder already contains a workspace | Warning banner with two choices: "Choose another" (focuses Parent folder) and "Overwrite" (requires explicit confirmation via MK-034 confirm dialog) |
| TLS on with empty cert/key | Inline error under the relevant field |

Validation runs on blur and on submit; never on keystroke (don't fight the user mid-typing).

## Visual treatment

- Modal-style centred page; not a snora `Dialog` overlay because the user has not yet entered the Workspace state. It's a standalone view between Welcome/Dashboard and Workspace.
- The container is a card on `surface.raised`, ~680 px wide, ~620 px tall.
- The bottom action row is sticky inside the card (always visible even when the form scrolls).
- **Cancel** is `ghost` (the user can return to Welcome or Dashboard).
- **Create workspace** is `primary`.

## Section collapse behaviour

When a section is collapsed:
- The chevron points right (`▸`)
- A one-line hint describes the defaults: e.g. "Default: 127.0.0.1 : 8080, no TLS"
- The hint uses `caption` token

When expanded:
- Chevron points down (`▾`)
- All inputs visible

Section open/close state is local to this wizard instance; not persisted.

## Interactions

| Action | Result |
|---|---|
| Type in Name or Parent folder | Inline validation on blur |
| Click section header / chevron | Toggle expand/collapse |
| Press Enter on a focused section header | Toggle expand/collapse |
| Cancel | Return to Welcome (if no recents) or Dashboard |
| Create workspace | Validate → on success, create workspace files and open Routes screen with the starter rule selected (if any) |
| `Esc` | Same as Cancel |

## After creation

After successful creation:
1. App transitions to Workspace state with the new workspace loaded.
2. The default left-rail destination is Routes.
3. If starter content was selected, the first rule in the first rule-set is preselected so the user lands on a non-empty editor.
4. The top bar's workspace identity now shows the new workspace name.

## Acceptance criteria

- Most users complete creation in less than 30 seconds (default-only path: name → folder → Create).
- Expanding all three advanced sections does not require scrolling on a 1280 × 800 window.
- Pressing Enter while focused on a primary input (Name, Parent folder) does **not** accidentally submit — only the Create button submits.
- Inline validation messages are screen-reader-readable.
- JA locale renders without clipping.

## Out of scope

- Templates beyond "Basic REST + error samples" — a v2 feature.
- Git initialisation or VCS hooks.
- Multi-workspace creation in one pass.
