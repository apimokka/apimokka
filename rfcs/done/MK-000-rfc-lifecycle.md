# RFC MK-000 — RFC lifecycle policy

**Status.** Implemented (v0.1.0)
**Tracks.** Cross-cutting documentation policy for `rfcs/`.
**Touches.** `rfcs/` folder structure, `rfcs/README.md`, Status field convention.

## Summary

Adopts the standard 4-folder RFC lifecycle (proposed / done / archive / draft-optional)
for the apimokka project, following the policy defined in the apimock-rs project's
`rfcs/done/000-rfc-lifecycle-policy.md`.

RFCs are numbered `MK-NNN` to distinguish them from the apimock-rs engine RFCs.
Numbers are sequential, stable forever, and never reused.

## Folder layout

```
rfcs/
  README.md       ← index grouped by state
  proposed/       ← open for review
  done/           ← implemented; historical record
  archive/        ← withdrawn or superseded
```

The folder is the source of truth for each RFC's state. The Status field
in the file header must match.
