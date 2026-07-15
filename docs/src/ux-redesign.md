# UX redesign rationale (v0.3.0)

## Mental model

A user opens apimokka to **observe and adjust how their mocked API
responds to traffic**. Everything else (workspace management, settings,
script inspection) supports that core loop:

```
        edit rule
            ↓
        send request from app/curl
            ↓
        observe trace
            ↓
        confirm match / diagnose miss
            ↓
        (back to top)
```

The shell must keep all four steps in the user's peripheral vision at
once. v0.1–v0.2 didn't — it separated edit (Routes) from observe (Trace)
into different tabs, breaking the loop on every iteration.

## What changed

### 1. Trace lives next to Routes

The Routes screen now embeds a **collapsible Live Trace strip** as its
right column. The user can edit a rule and see the latest events without
switching tabs. The strip is toggleable (top-bar button) for users who
want the full editor width.

The standalone Trace tab is kept for full-screen drill-in, but is no
longer the primary trace surface.

### 2. Rule editor is two-column

Old layout: URL → Method → Headers → Body → Respond stacked top-to-bottom.

New layout:

```
┌─────────────────────────┬──────────────────────────┐
│ WHEN                    │ RESPOND                  │
│   URL path              │   ⦿ Inline  ○ Serve file │
│   Method                │   body editor            │
│   Headers               │   status + delay         │
│   Body conditions       │                          │
│                         │                          │
│                         │   [ Test rule ]          │
└─────────────────────────┴──────────────────────────┘
```

This makes the rule's "if X then Y" structure visually explicit and
keeps the response field visible while editing match conditions.

### 3. Overview tab removed

Everything Overview showed is shown more efficiently elsewhere:

| Old (Overview tab) | New location |
|---|---|
| Routing-stack diagram (scripts → rules → files) | Welcome screen only (newcomer education) |
| Health grid (validation / dirty / server / trace) | Top bar (always visible) |
| Quick actions (Save, Routes, Trace) | Command palette + top bar |

Routes is now the default landing tab when a workspace opens.

### 4. Wizard is one page

Five sequential steps collapsed into a single scrolling form with
collapsible "advanced" sections. The required-fields-only path is:

```
Workspace name: [____________]
Location:       [____________]
                              ╲
                               ▼ Create

   ▶ Server (defaults: 127.0.0.1:3000, no TLS)
   ▶ Starter content (defaults: basic REST + error samples)
   ▶ Trace (defaults: UDS, 1024 queue)
```

Most users hit Create immediately. Power users expand the advanced
panels. Either way it's one screen, not five.

### 5. Validation chip in top bar

Old: "Validation" was a tab inside the bottom drawer. Opening it
required clicking the drawer first, then switching tabs.

New: A clickable chip in the top bar shows `⚠ 2 issues`. Clicking opens
a focused side panel listing all issues grouped by file with
click-to-jump navigation. No drawer involved.

## What was kept

- Bottom drawer for Save Diff (the only place that still needs modal
  attention because it confirms a transient action)
- Command palette as the keyboard-first action surface
- Confirmation dialogs for destructive actions
- All status indicators carry glyph + text (ABDD non-colour)

## What's deferred to v0.4

- **Auto-save with explicit restart-needed prompt.** Removes the Save
  button + dirty dots dance. Needs careful edge-case design (what if
  the user is mid-edit and a settings change requires restart?).
- **Workspace switcher in top bar.** Replaces the standalone Dashboard
  with an inline dropdown. Needs recent-workspace persistence.
- **"Remember last workspace" on launch.** Skip Welcome if a workspace
  was open in the last session.
- **Keyboard shortcut hints visible in the command palette.** Currently
  shortcuts exist (Esc, Cmd+K) but are undiscoverable.

## Measurement

How would we know the redesign worked? The hypothesis is **fewer tab
switches per edit-verify cycle**. In v0.2, editing one rule and verifying
its match took at least 2 tab switches (Routes → Trace → Routes). In
v0.3, with the trace strip visible alongside Routes, it takes zero.

Production telemetry could confirm this. For the mockup, it's a design
claim the next reviewer can sanity-check.
