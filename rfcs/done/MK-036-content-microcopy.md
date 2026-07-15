# RFC MK-036 — Content and microcopy

**Status.** Implemented (v0.6.0)
**Tracks.** Voice, empty-state copy, error copy patterns.
**Touches.** Every text string the user sees.

## Summary

apimokka's text should sound like a developer tool, not a marketing site. Direct, specific, concrete. No "Oops", no friendly mascots, no decorative tone words. This RFC defines voice and the standard patterns the visual designer (or implementer) draws from when writing new strings.

## Voice

### Good

- "Replay as test input"
- "Reload required"
- "No rule selected"
- "Use dotted paths such as `user.id`"
- "Port 8080 is already in use."
- "Trace paused"
- "3 files will be written"

### Avoid

- "Oops!"
- "Something went wrong" (without detail)
- "Invalid" (without explaining why)
- "Awesome!"
- "We're here to help"
- Marketing tone inside the workspace
- Apologising for routine state ("Sorry, no events yet")

## Empty states

Every empty surface has paired message + optional CTA.

| Location | Message | CTA |
|---|---|---|
| Welcome, no recents | "Create a workspace to start authoring mock endpoints." | Create new workspace |
| Routes, no rule selected | "Choose a rule from the left, or create a new endpoint." | Add rule |
| Rule set has no rules | "This rule set has no rules yet." | Add rule |
| Trace, no events | "No requests observed yet. Trigger your app or curl the server." | Copy server URL |
| Scripts, none in workspace | "No middleware scripts in this workspace." (also explain what middleware is) | — |
| Validation drawer, no issues | "No validation issues." | — |
| Save diff drawer, nothing dirty | "No unsaved changes." | — |
| Dotted-path assistant, empty input | "Paste sample JSON to build a path." | — |
| Command palette, no match | "No matching commands" | — |

## Error copy pattern

Every error message follows three sentences (or three short phrases):

1. **What happened.**
2. **Why it matters.**
3. **What the user can do.**

### Example (port in use)

```
Port 8080 is already in use.
The mock server cannot start until the port is available.
Choose another port in Settings or stop the process using this port.
```

### Example (TLS cert missing)

```
TLS cert path is empty.
TLS is enabled, but the server can't start without a certificate.
Provide a cert path in Settings or disable TLS.
```

### Example (file write failure)

```
Could not write checkout.toml.
The save failed; your edits are still in memory.
Check folder permissions, then try Save again.
```

### Example (Rhai script error)

```
auth.rhai threw an error.
Middleware blocked this request before rule matching could run.
Open auth.rhai in your editor — error was on line 12.
```

## Validation messages

Each validation message answers two questions: **which field?** and **what's wrong?**

| Bad | Good |
|---|---|
| "Invalid value" | "URL operator requires a URL path." |
| "Required" | "Workspace name cannot be empty." |
| "Bad input" | "Port must be a number between 1 and 65535." |
| "Error" | "Header value is ignored for the Exists operator." |
| "Not allowed" | "Inline text and a served file cannot both be set." |

## Status chip labels

Match the state-machine table in MK-035 — single-word or short-phrase labels:

- `Running`, `Stopped`, `Starting`, `Reload pending`, `Restart required`, `Error`
- `Saved`, `Unsaved (N)`, `Saving…`, `Save error`
- `Trace paused`, `Trace connecting`, `Trace error`

The format `Unsaved (3)` instead of `3 unsaved` keeps the chip noun-led so glanced reading is consistent.

## Button labels

Use verbs in imperative form:

- `Save workspace` (not "Saving" or "Save now")
- `Add rule`
- `Run test`
- `Delete rule` (not "Delete" alone — specify what)
- `Discard` (without an object when the dialog title already specifies; e.g. confirm dialog says "Discard unsaved changes?" then button is `Discard`)
- `Reload`, `Restart`, `Start server`, `Stop server`

Capitalisation: sentence case (`Add rule`, not `Add Rule`).

## Hint and caption microcopy

Hints under inputs use lowercase / sentence-case prose, ending in a period:

- "Use `/api/orders`, not the full URL."
- "Dotted path such as `user.id` or `items.0.name`."
- "TLS requires both a cert and a key file."

Captions under cards or section headers may be sentence fragments if the context is clear:

- "Default: 127.0.0.1 : 8080, no TLS"
- "Restart required after changing host, port, or TLS"

## Localisation rules

Every user-visible string is keyed in `apimokka-i18n::Key` and translated into both EN and JA. Strings are added in pairs.

JA strings are typically 10–30% wider than EN; designers must size button widths and chip widths for the JA case, not EN.

Substitutions in strings (`{n} issues`) use named placeholders, not positional indices, so translators can reorder them.

## Voice for the Welcome / Wizard

The pre-workspace surfaces (Welcome and Wizard) are slightly warmer than the workspace surfaces — these are the first impression for a new user. "Visual HTTP mock authoring" is a tagline; "Create a workspace to start authoring mock endpoints." is gentle.

Once the user is in the Workspace state, voice tightens to direct/technical.

## Acceptance criteria

- Every empty-state surface listed above has a string in the i18n table.
- Every error message follows the three-sentence pattern (or has an explicit exemption noted in code).
- Status chip labels match the MK-035 catalogue exactly.
- Button labels are verbs in sentence case.
- A spot-check of EN vs JA shows no truncation at the reference window width.

## Out of scope

- Marketing copy on the apimokka website
- Help articles / external documentation
- Onboarding tooltips (deferred; v1 relies on empty states + visible affordances)
