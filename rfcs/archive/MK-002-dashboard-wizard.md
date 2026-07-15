# RFC MK-002 — Workspace dashboard and new-workspace wizard

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Workspace entry flow.
**Touches.** `screens/welcome.rs`, `screens/dashboard.rs`, `screens/wizard.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| First-time user can create a workspace without editing TOML | ✅ | `wizard.rs` 5-step flow + `Message::WizardCreate` |
| Wizard shows exactly which files it will create | ⚠️ Partial | Step 5 review screen lists settings; does NOT list files-to-create paths |
| Existing files not overwritten accidentally | ❌ | Mockup creates against a mock workspace; not exercised |
| Created workspace opens to a useful Routes screen | ✅ | `WizardCreate` switches to `AppView::Workspace`, tab=Overview |
| Recent workspace entries handle missing paths gracefully | ❌ | Mockup uses static `mock::recent_workspaces`; no missing-path handling |

## Deferred

The "files-to-create preview" panel on wizard step 5 and missing-path
indicators on the dashboard are deferred. Both are visual additions
that don't change the message flow.
