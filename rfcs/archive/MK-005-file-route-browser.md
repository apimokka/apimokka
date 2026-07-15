# RFC MK-005 — File route browser and fallback hints

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Fallback file browser surface.
**Touches.** `screens/routes.rs` left sidebar, `screens/settings.rs` file-tree section

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Users see which files may serve which fallback routes | ✅ | Routes-left-sidebar fallback section shows `users.json → /users` etc. via `route_hint` |
| Filter controls align with file tree configuration | ⚠️ Partial | Filters configurable in Settings → File tree; not exposed inline next to the file list |
| Browser does not mutate files except via explicit settings or promote-to-rule | ✅ | Click handler is `Message::SelectFileRoute` only — no mutation messages |
| Missing fallback configuration explained with link to Settings | ❌ | No empty-state link to Settings yet |

## Deferred

The dedicated **File Browser tab** screen mentioned in the original RFC §
"Mockup Surface" was not built; fallback files live as a section inside
the Routes left sidebar instead. Promote-to-rule action also deferred.

## Acceptance gaps for production

1. Dedicated file-browser screen with filter controls inline
2. "Promote to rule" right-click action wired to `Message::AddRule` with
   prefilled `url_path` and `respond.file_path`
3. Empty-state hint linking to Settings → File tree filters
