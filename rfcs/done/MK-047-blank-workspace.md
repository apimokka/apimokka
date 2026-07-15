# RFC MK-047 — Blank workspace from wizard

**Status.** Implemented (v0.9.16)
**Tracks.** Wizard-created workspaces are blank (no rules); OpenWorkspace
loads a full existing mock.
**Touches.** `apimokka-model/src/mock.rs`, `app.rs`, `screens/routes.rs`, i18n.
**Follows.** MK-026 (workspace wizard), MK-046 (first-launch flow).

## Problem

`WizardCreate` and `OpenWorkspace` both called `mock::shop_api_mock()`,
so creating a new workspace and opening an existing one were identical.
This broke the product story: a user who creates "inventory-mock" from
the wizard should see an empty workspace, not the pre-loaded shop API rules.

## Changes

- `mock::blank_workspace(name, host, port, tls) -> WorkspaceSnapshot` added
  to the model crate. Returns a snapshot with the given metadata and settings,
  empty rule_sets, fallback_files, middleware_scripts, and diagnostics.
- `WizardCreate` handler uses `blank_workspace` with the wizard's field values.
- `OpenWorkspace` continues to use `shop_api_mock()` (represents an existing workspace).
- Centre panel: when `snap.rule_sets.is_empty()`, shows the blank-workspace
  empty state ("Add a rule set to start mocking.") with "Add rule set" CTA.
- Server state shows Stopped after create (new workspace is not yet running).
- Success notice shown after workspace creation.
