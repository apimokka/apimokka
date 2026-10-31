# Handoffs — MK-023, accessibility and keyboard contract

Companion execution documents for
[MK-023](../../proposed/MK-023-accessibility-keyboard.md). Their lifecycle state
is **inherited from that RFC** and is not managed here.

MK-023's external design was accepted 2026-10-31 and is implemented in six
slices. One handoff per slice, added as each is issued.

| Slice | Handoff | Covers |
|---:|---|---|
| 1 | [`026-m12-slice1-capture-status-and-dialog-default.md`](./026-m12-slice1-capture-status-and-dialog-default.md) | Read `event::Status`; dialog default actions. **Complete** — `dbf947c`, closed D-6 and the `Enter`-in-dialog defect. |
| 2 | [`027-m12-slice2-regions-and-cursor.md`](./027-m12-slice2-regions-and-cursor.md) | The region and cursor model, Tab between and arrows within, proved on the tab bar and the mode picker. **The foundation slices 3–6 build on.** |
| 3–6 | not yet issued | Welcome and Dashboard; the sidebar tree; segmented regions and the trace list; palette moves and deleting the left rail (D-16) |

Slice numbering follows the design section's list; the file-name number is the
dev-team task number, which review requests cite.
