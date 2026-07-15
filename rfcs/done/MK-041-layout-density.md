# RFC MK-041 — Layout density: common-first, advanced-behind-More

**Status.** Implemented (v0.10.0)
**Tracks.** MK-040 phase 3. Per-screen layout density in Guided mode: the
common 80% is always visible; the advanced 20% lives behind a toggleable
"More" row.
**Touches.** `App` state, `message.rs`, `screens/routes.rs` (WHEN column),
`screens/settings.rs`, i18n.
**Follows.** MK-040 (audience modes).

## Motivation

v0.9.0 made hints and errors mode-aware but did not change which controls are
visible by default. The WHEN panel still shows four cards (URL path, method,
headers, body) even in Guided mode. For a newcomer writing their first rule,
headers and body conditions are advanced concerns; showing them immediately
adds visual load before the user has learned to need them.

Phase 3 applies the "common-first" principle screen by screen:
- **Rule editor WHEN panel**: URL path + method are the 80% path. Headers and
  body conditions are behind "More matching criteria" in Guided.
- **Settings**: Appearance + Server are the 80% path. Logs and Trace are
  behind "More settings" in Guided.

## Design rule

Every "More" section must:
1. Show a count badge if hidden conditions are **active** — never silently hide
   state that affects server behaviour. (`2 headers active`, `1 body active`.)
2. Be expandable in one click. No modal, no navigation.
3. Remain expanded for the rest of the session once opened (not reset per
   rule/navigation — that would be frustrating).
4. Be invisible in Expert mode — the controls are always shown directly.

## State additions

```rust
// In App — layout density toggles (Guided mode only)
pub rule_when_more: bool,       // headers + body cards expanded
pub settings_advanced_more: bool, // Logs + Trace sections expanded
```

Both start `false`. Reset to `false` when `ChooseAudienceMode(Guided)`.
Irrelevant in Expert mode (rendering bypasses the toggle).

## Messages

```rust
ToggleRuleWhenMore,
ToggleSettingsAdvancedMore,
```

## WHEN column: before / after

**Expert (unchanged):**
```
URL Path   [/api/checkout  ] [Equal ▼]
Method     [Any][GET][POST] ...
Headers    (0)  + Add
Body       (0)  + Add
```

**Guided, collapsed (default):**
```
URL Path   [/api/checkout  ] [Equal ▼]
Method     [Any][GET][POST] ...
▸ More matching criteria
```

**Guided, collapsed, with hidden active conditions:**
```
URL Path   [/api/checkout  ] [Equal ▼]
Method     [Any][GET][POST] ...
▸ More matching criteria   1 header · 2 body active
```

**Guided, expanded:**
```
URL Path   [/api/checkout  ] [Equal ▼]
Method     [Any][GET][POST] ...
Headers    (1)  content-type = application/json   ✕
           + Add header condition
Body       (2)  action Equal "create" ...
           + Add body condition
▾ Fewer matching criteria
```

## Settings: before / after

**Expert (unchanged):** Appearance · Server · Logs · Trace all visible.

**Guided, collapsed:**
```
Appearance  [Light][Dark]  [Guided][Expert]  EN ▼
Server      [127.0.0.1] [8080]
▸ More settings
```

**Guided, expanded:**
```
Appearance ...
Server ...
Logs ...
Trace ...
▾ Fewer settings
```

## Acceptance criteria

- Expert mode: no change to any layout (zero regression).
- Guided, WHEN collapsed: only URL path + method cards visible.
- Guided, WHEN collapsed with active conditions: count shown next to toggle.
- Guided, WHEN expanded: all four cards visible.
- Guided toggle state persists across rule navigation (not reset per rule).
- Settings: same pattern for Logs + Trace.
- Zero errors, zero warnings, new + existing tests pass.

## Out of scope

- Respond column (delay is a single small field; not worth a toggle).
- Sidebar density (already "less is more" for all audiences).
- Wizard (already uses progressive disclosure natively).
- Real persistence of the expanded state across app restarts.
