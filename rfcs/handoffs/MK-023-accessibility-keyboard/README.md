# Handoffs — MK-023, accessibility and keyboard contract

Companion execution documents for
[MK-023](../../proposed/MK-023-accessibility-keyboard.md). Their lifecycle state
is **inherited from that RFC** and is not managed here.

MK-023's external design was accepted 2026-10-31. The slice plan named six;
evidence has extended it three times — slice 7's scenario-5 gap, slice 8's
census, and the acceptance clause that census made visible. One handoff per
slice, added as each is issued.

| Slice | Handoff | Covers |
|---:|---|---|
| 1 | [`026-m12-slice1-capture-status-and-dialog-default.md`](./026-m12-slice1-capture-status-and-dialog-default.md) | Read `event::Status`; dialog default actions. **Complete** — `dbf947c`, closed D-6 and the `Enter`-in-dialog defect. |
| 2 | [`027-m12-slice2-regions-and-cursor.md`](./027-m12-slice2-regions-and-cursor.md) | The region and cursor model. **Complete** — `926a4fc`, tab bar keyboard-operable, `mode_picker_selected` removed. |
| 3 | [`028-m12-slice3-welcome-and-dashboard.md`](./028-m12-slice3-welcome-and-dashboard.md) | Welcome and Dashboard. **Complete** — `cc56256`, closed D-12's headline symptom; a keyboard-only user reaches a workspace with no pointer event. Surfaced D-18. |
| 4 | [`029-m12-slice4-sidebar-tree.md`](./029-m12-slice4-sidebar-tree.md) | The routes sidebar tree. **Complete** — `4ba6552`, three expansion behaviours in one region, plus the keyboard-only rule-set collapse. |
| 5 | [`030-m12-slice5-editor-and-trace.md`](./030-m12-slice5-editor-and-trace.md) | The rule editor and trace screen. **Complete** — `556102b`, scenario 3 keyboard-completable. Exposed D-19. |
| 6 | [`031-m12-slice6-settings-and-wizard.md`](./031-m12-slice6-settings-and-wizard.md) | Settings and the Wizard. **Complete** — `1258ce9`; all six scenarios keyboard-completable. Surfaced D-20. |
| 7 | [`032-m12-slice7-final.md`](./032-m12-slice7-final.md) | D-16, D-18, D-19, D-20 and the palette commands. **Complete** — `0af9e09`; five of six scenarios captured, and **scenario 5 found broken**. |
| 8 | [`033-m12-slice8-remaining-panes.md`](./033-m12-slice8-remaining-panes.md) | The three uncovered panes, the scroll-dispatch refactor, and **the census**. **Complete** — `7df87ad`; closed **D-12** (scenario 5 captured end to end) and covered a fourth surface, the confirm dialog. Its census found seven more, three of them named in MK-023's own acceptance clause. |
| 9 | [`034-m12-slice9-inventory-and-overlay-precedence.md`](./034-m12-slice9-inventory-and-overlay-precedence.md) | **The last.** The acceptance clause's remaining inventory (**D-23**), one overlay-precedence source (**D-22**), and the two corrections from slice 8's review. On acceptance MK-023 moves to `done/`. |

Slice numbering follows the design section's list; the file-name number is the
dev-team task number, which review requests cite.
