# Handoffs — MK-023, accessibility and keyboard contract

Companion execution documents for
[MK-023](../../proposed/MK-023-accessibility-keyboard.md). Their lifecycle state
is **inherited from that RFC** and is not managed here.

MK-023's external design was accepted 2026-10-31 and is implemented in six
slices. One handoff per slice, added as each is issued.

| Slice | Handoff | Covers |
|---:|---|---|
| 1 | [`026-m12-slice1-capture-status-and-dialog-default.md`](./026-m12-slice1-capture-status-and-dialog-default.md) | Read `event::Status`; dialog default actions. **Complete** — `dbf947c`, closed D-6 and the `Enter`-in-dialog defect. |
| 2 | [`027-m12-slice2-regions-and-cursor.md`](./027-m12-slice2-regions-and-cursor.md) | The region and cursor model. **Complete** — `926a4fc`, tab bar keyboard-operable, `mode_picker_selected` removed. |
| 3 | [`028-m12-slice3-welcome-and-dashboard.md`](./028-m12-slice3-welcome-and-dashboard.md) | Welcome and Dashboard. **Complete** — `cc56256`, closed D-12's headline symptom; a keyboard-only user reaches a workspace with no pointer event. Surfaced D-18. |
| 4 | [`029-m12-slice4-sidebar-tree.md`](./029-m12-slice4-sidebar-tree.md) | The routes sidebar tree. **Complete** — `4ba6552`, three expansion behaviours in one region, plus the keyboard-only rule-set collapse. |
| 5 | [`030-m12-slice5-editor-and-trace.md`](./030-m12-slice5-editor-and-trace.md) | The rule editor and trace screen. **The largest remaining**; scope corrected upward from the design's "segmented regions" after counting 24 controls in the editor alone. |
| 6 | not yet issued | palette moves (theme, audience mode), deleting the left rail (D-16), and D-18 |

Slice numbering follows the design section's list; the file-name number is the
dev-team task number, which review requests cite.
