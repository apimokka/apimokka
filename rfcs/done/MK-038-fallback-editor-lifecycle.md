# RFC MK-038 — Fallback file editor: workflow and data lifecycle

**Status.** Implemented (v0.6.3)
**Tracks.** Viewing, editing, saving, and reverting fallback `.json` files (file-system routing) on the Routes screen.
**Touches.** `App` state, Routes centre panel, sidebar dirty markers, top-bar save chip, confirm dialog.
**Refines.** MK-028 (Routes workbench), MK-035 (state models).

## Summary

v0.6.2 introduced a fallback file editor that proved the surface but had the
wrong data lifecycle: a single mutable string per file, keystroke-incremented
dirty counters, a single-line input widget, and no way back. This RFC defines
the correct model: a **two-buffer (saved/draft) lifecycle per file** with
explicit save, confirmed revert, and live JSON validity feedback.

## Why explicit save (not auto-save)

Rule edits auto-save (MK-035) because each structured field edit is atomic and
always valid. Free-text JSON is different: while typing, the buffer passes
through invalid states. apimock-rs reads fallback files from disk per request,
so persisting transient states would have the running server serving broken
JSON mid-edit. Therefore:

| Surface | Save policy | Rationale |
|---|---|---|
| Rule editor fields | Auto-save (debounced) | Atomic, always-valid edits |
| Fallback file content | **Explicit save** | Transient invalid states while typing |
| Fallback status code | Explicit (saved with the file) | Belongs to the same commit |

JSON validity **warns but never blocks** saving — serving intentionally
malformed JSON is a legitimate test case for client error handling.

## Data model

Per file, keyed by file path:

```
saved:  String                  // baseline — what is "on disk"
draft:  text_editor::Content    // live editor buffer (created on first open)
status_saved / status_draft: String

dirty(path)      := normalize(draft.text()) != normalize(saved)
json_valid(path) := serde_json::from_str(draft.text()).is_ok()
```

`normalize` trims a trailing newline so widget round-trips don't create
phantom dirtiness. Dirtiness is always derived, never counted.

## Lifecycle state machine (per file)

```
            select file (first time)
[Untracked] ───────────────────────► [Clean]    draft ← saved
[Clean]   ── edit ─────────────────► [Dirty]
[Dirty]   ── edit ─────────────────► [Dirty]
[Dirty]   ── Save ─────────────────► [Clean]    saved ← draft
[Dirty]   ── Revert (via confirm) ─► [Clean]    draft ← saved
[Dirty]   ── switch file ──────────► [Dirty]    draft persists; sidebar shows ●
[Dirty]   ── close workspace ──────► confirm (existing DiscardChanges flow)
```

Drafts are per-file and survive switching files: the user can edit
`users.json`, peek at `health.json`, and return without losing work. The
sidebar shows a dirty dot (●) on every file whose draft differs from saved.

## User workflow

1. **View.** Click a file in the sidebar. The editor shows the draft (equal to
   saved on first open). Header: filename, `Serves: GET /users` pill,
   explanation line. Footer: validity badge + state hint.
2. **Edit.** Type in the multi-line editor. The file becomes Dirty
   immediately: sidebar dot appears, top-bar chip increments, Save and Revert
   enable, validity badge updates live (`✓ Valid JSON` / `⚠ Invalid JSON`).
3. **Save.** Per-file Save button, or the global top-bar Save (commits all
   dirty drafts). After save the footer reads "Saved — changes take effect on
   the next request." No server reload is required: apimock-rs reads fallback
   files per request.
4. **Revert.** Ghost button next to Save; destructive, so it routes through
   the standard confirm dialog (MK-034). On confirm, draft ← saved.

## Global integration

- Top-bar `Unsaved (N)` = dirty rule files + dirty fallback files, recomputed
  after every relevant message.
- Global Save commits all dirty fallback drafts (rules already auto-save).
- `Format JSON` pretty-prints the draft in place (an edit, not a save).

## Widget requirements

- `iced::widget::text_editor` with `Content` stored in `App`
  (`HashMap<String, Content>`); `Content` is stateful and not `Clone`.
- Monospace font at `size::MONO`; editor fills available height.
- `text_editor::Action` is `Clone + Debug`, so it can ride in `Message`.

## Acceptance criteria

- Editing never mutates `saved`; only Save does.
- Dirty state is derived; switching files preserves drafts and dots.
- Revert requires confirmation and restores the saved baseline exactly.
- Validity badge updates live and never blocks save.
- Top-bar chip counts rule + file dirtiness without double counting or
  keystroke inflation.
- Zero errors, zero warnings.

## Out of scope

- Multi-level undo within the buffer (the text_editor widget provides its own
  in-session editing; full undo stacks are a v2 item in ROADMAP).
- Creating/renaming/deleting files (separate RFC, `+ Add file` stays stubbed).
- Disk I/O (mockup keeps "disk" in memory as `saved`).
