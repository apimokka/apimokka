# RFC MK-021 — Workflow-centred information architecture

**Status.** Implemented (v0.6.0)
**Tracks.** Product foundation — what apimokka is, who uses it, how its screens are organised.
**Touches.** The entire app shell; every screen and overlay in the rest of the MK-021..MK-037 series builds on this RFC.
**Supersedes.** MK-001 (app shell), MK-004 (route overview), MK-018 (workflow-centred redesign).

## Summary

apimokka is a desktop GUI for the apimock-rs HTTP mock server. The product's identity is a **workbench**, not a dashboard: the user edits a rule and watches live traffic in the same view, iterating without changing tabs.

This RFC establishes the top-level information architecture and the visual priority that the rest of the redesign serves. Every screen RFC (MK-025..MK-032) refines a part of the structure laid out here.

## Design philosophy

### One sentence
A native desktop app where authors of HTTP mock rules edit those rules visually and watch live request traffic match — or miss — them in the same view.

### What apimokka is *not*
A full IDE; a prettier TOML editor; a network proxy/debugger; a multi-user collaborative tool; a visual scripting editor for Rhai; an analytics dashboard.

### Experience goals
The app should feel **focused** (the current rule or event is always clear), **inspectable** (every decision has a visible reason), **safe** (destructive actions are explicit; save/reload/restart state is visible), **fast** (frequent actions need minimal navigation), **accessible** (every status is text + icon, never colour-only), and **native** (respects iced/snora capabilities; no web-only patterns).

## User roles

| Role | Primary need | Primary screen |
|---|---|---|
| Backend developer | Stub external dependencies and switch responses quickly | Routes |
| Frontend developer | Mock unavailable backend endpoints | Routes + Wizard |
| QA engineer | Force edge cases and verify app behaviour | Routes + Trace |
| API designer | Prototype endpoint behaviour before implementation | Routes + Dashboard |
| New user | Open or create a workspace without learning apimock-rs internals | Welcome + Wizard |
| Power user | Operate without a pointer | Command palette + shortcuts |

## Outer application states

```
[*] → Welcome
Welcome → Dashboard (recent workspaces exist)
Welcome → Wizard (create new)
Welcome → Workspace (open existing)
Dashboard → Workspace
Dashboard → Wizard
Wizard → Workspace (after Create)
Workspace → Dashboard (switch / close)
Workspace → Wizard (new workspace)
```

## Workspace shell composition

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Top Bar: Workspace identity · Server state · Save state · Global actions   │
├──────────────┬───────────────────────────────────────────────┬─────────────┤
│ Left Rail    │ Body                                          │ Right/Sheet │
│ Routes       │ Per destination content                       │ Optional    │
│ Trace        │                                               │             │
│ Scripts      │                                               │             │
│ Settings     │                                               │             │
├──────────────┴───────────────────────────────────────────────┴─────────────┤
│ Bottom Drawer: Validation / Save Diff                                      │
└────────────────────────────────────────────────────────────────────────────┘
```

## Navigation destinations (left rail)

| Destination | Frequency | Purpose | Design priority |
|---|---:|---|---|
| Routes | Very high | Edit rules and watch live traffic | Most refined, lowest friction |
| Trace | High | Deep request/match diagnosis | Filterable, inspectable, replayable |
| Scripts | Low | Inspect middleware Rhai scripts | Read-only clarity |
| Settings | Low | Configure server, logs, TLS, trace, strategy | Safe changes with restart/reload hints |

Routes is the **product's identity**. Visual investment is allocated proportionally: Routes > Trace > Welcome/Wizard > Settings > Scripts.

## Overlay model

Only one overlay at a time. Esc closes the topmost overlay; outside-click dismisses where the snora primitive supports it. Priority order (highest first):

1. **Confirm dialog** — destructive-action confirmation
2. **Workspace menu** — header-menu dropdown
3. **Command palette**
4. **Test Rule dialog**
5. **Dotted-Path Assistant**
6. **Bottom drawer** — validation / save-diff

A new overlay opening from underneath an existing one is not allowed; the current overlay must be closed first.

## Visual priority rules

### Routes screen priority
1. Selected rule title and summary
2. URL/method/response editing controls
3. Live trace latest outcome
4. Validation status
5. Rule-set organisation
6. Advanced strategy fields

### Trace screen priority
1. Outcome and path
2. Selected event detail
3. Match/miss explanation
4. Replay action
5. Raw headers/body

### Settings priority
1. Current server binding and state
2. Fields that require restart
3. Trace enablement
4. Logs
5. Advanced strategy

## Screen inventory

| ID | Surface | Owning RFC |
|---|---|---|
| S-00 | Welcome | MK-025 |
| S-01 | Dashboard | MK-025 |
| S-02 | New workspace wizard | MK-026 |
| S-03 | Workspace shell | MK-027 |
| S-05 | Routes workbench | MK-028 |
| S-06 | Add new endpoint flow | MK-028 |
| S-07 | Live trace strip | MK-028 |
| S-08 | Rule inspector | MK-028 |
| S-11 | Trace screen | MK-029 |
| S-12 | Match detail panel | MK-029 |
| S-13 | Settings | MK-030 |
| S-14 | Scripts | MK-031 |
| D-01 | Validation drawer | MK-032 |
| D-02 | Save-diff drawer | MK-032 |
| O-01 | Workspace menu | MK-034 |
| O-02 | Command palette | MK-033 |
| O-03 | Test Rule dialog | MK-034 |
| O-04 | Dotted-path assistant | MK-034 |
| O-05 | Confirm dialog | MK-034 |

## Acceptance criteria

- The default landing tab in a freshly-opened workspace is Routes.
- The Routes screen can show editor and live trace strip simultaneously without a tab switch.
- The four left-rail destinations are Routes, Trace, Scripts, Settings — in that order.
- No screen shows information that is more efficiently shown elsewhere (e.g. health duplicated between Overview and top bar is not allowed; Overview is removed).
- Every overlay can be opened from a keyboard shortcut or command palette entry.

## Out of scope (for this RFC; covered elsewhere)

- Visual tokens, typography, colour — see **MK-022**.
- Keyboard contract and accessibility rules — see **MK-023**.
- Window sizing and breakpoints — see **MK-024**.
- Per-screen detail — see **MK-025..MK-032**.
- State machines for server/save/trace — see **MK-035**.
