# RFC MK-025 — Welcome and Dashboard

**Status.** Implemented (v0.6.0)
**Tracks.** S-00 Welcome, S-01 Dashboard.
**Touches.** First-launch surface and workspace picker.
**Supersedes.** MK-002 (dashboard and wizard — wizard portion lives in MK-026).

## Summary

Two pre-workspace screens introduce the product and let the user pick which workspace to open. Welcome serves first-launch and no-recents cases; Dashboard serves users with a history of workspaces. Both screens are generous with whitespace; neither tries to look like a portal page.

## S-00 Welcome

### Purpose
First-launch surface for users with no workspace open or no recents.

### Content requirements

| Element | Requirement |
|---|---|
| App name | Large, calm title using `display` token |
| Tagline | "Visual HTTP mock authoring" (or equivalent — see microcopy MK-036) |
| Primary actions | **Open workspace** (primary), **Create new workspace** (secondary) |
| Educational diagram | Middleware scripts → Rule sets → Fallback files |
| Empty-recents message | Friendly, non-blaming |

### Layout sketch

```
┌─────────────────────────────────────────────────────────────┐
│ apimokka                                                    │
│ Visual HTTP mock authoring                                  │
│                                                             │
│ [Open workspace] [Create new workspace]                     │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ How requests are handled                                │ │
│ │ Middleware scripts → Rule sets → Fallback files         │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ No recent workspaces yet. Create one to get started.        │
└─────────────────────────────────────────────────────────────┘
```

### Interactions

| Action | Result |
|---|---|
| Open workspace | Opens system folder picker |
| Create new workspace | Opens Wizard (MK-026) |
| Click an existing recent (when present) | Opens that workspace |

### Empty-state tone

> "Create a workspace to start authoring mock endpoints."

Do not blame the user; do not over-explain.

### Visual treatment

- The whole page is centred horizontally with a max content width of ~720 px.
- The hero (app name + tagline) uses `space.6` vertical breathing room above and below.
- Educational diagram is a quiet card on `surface.panel`, no animation.
- When recents exist, this screen is not shown; Dashboard takes over.

## S-01 Dashboard

### Purpose
Workspace picker for returning users.

### Content requirements

| Element | Requirement |
|---|---|
| Search bar | Filters by name and path |
| Pinned section | Workspaces the user explicitly pinned |
| Recent section | Most-recently-opened workspaces, sorted by `last_opened` |
| Per-row open button | Primary CTA on the focused row |
| Pin toggle | Icon button per row (with text alternative) |
| Last opened | Caption text per row (`"Last opened today"` etc.) |
| Path | Monospace or muted secondary text |

### Layout sketch

```
┌─────────────────────────────────────────────────────────────┐
│ Workspaces                                      [+ New]     │
│ [Search workspaces…]                                        │
│                                                             │
│ Pinned                                                      │
│ ┌ Project Alpha · /path/to/project · Last opened today · Open
│ └─────────────────────────────────────────────────────────  │
│ Recent                                                      │
│ ┌ Payments Mock · /path/payments · Last opened yesterday · Open
│ └─────────────────────────────────────────────────────────  │
└─────────────────────────────────────────────────────────────┘
```

### Row item fields

| Field | Display |
|---|---|
| Workspace name | Primary row label, `bodyStrong` |
| Path | Monospace, `caption` |
| Last opened | `caption`, secondary text colour |
| Pin toggle | Icon button + screen-reader label "Pin workspace" |
| Open button | Secondary by default; primary when row is focused |

### Workflow

```
Dashboard → search → select row → Open → Workspace Shell
Dashboard → [+ New] → Wizard
```

### Visual treatment

- Section headers ("Pinned", "Recent") use `section` token.
- Rows are cards on `surface.panel` with subtle elevation, no border.
- Search input has a visible label "Search workspaces…" (placeholder is acceptable, but a sr-only label is required for accessibility).
- An entire row is clickable and behaves like its Open button. The Open button on the right is for explicit affordance and keyboard discoverability.

## Open-workspace and create-workspace shortcuts

Both screens expose:
- A keyboard tab to the action buttons
- The same two actions through the command palette (MK-033): "Open workspace", "Create new workspace"

## Acceptance criteria

- Welcome and Dashboard are mutually exclusive: Welcome only renders when recents are empty.
- Both screens centre their content with comfortable max width; they do not stretch full-width.
- Search filtering on Dashboard is instant (no submit button).
- Pin toggle persists across app restarts (implementation detail; the UI just exposes the toggle).
- Both screens render correctly in light and dark themes.

## Out of scope

- Wizard (handled in MK-026).
- Persistence of recents and pins (a v1 implementation reads/writes a small JSON file in the app config directory; this is implementation detail).
- "Remember last workspace" behaviour at launch — flagged as a follow-up; for v1 the app always shows Welcome or Dashboard at startup.
