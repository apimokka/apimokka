# Handoffs — MK-023, accessibility and keyboard contract

Companion execution documents for
[MK-023](../../done/MK-023-accessibility-keyboard.md). Their lifecycle state
is **inherited from that RFC** and is not managed here.

MK-023's external design was accepted 2026-10-31. The slice plan named six;
evidence extended it four times — slice 7's scenario-5 gap, slice 8's census,
the acceptance clause that census made visible, and slice 9's own overlay
source proving one dimension short. Each extension came from a slice looking
where it was told to look and finding the layer above it; the containers are
region inside overlay inside view, and `App::view` is the root, so slice 10
exhausted them. **All ten complete 2026-11-03.**

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
| 9 | [`034-m12-slice9-inventory-and-overlay-precedence.md`](./034-m12-slice9-inventory-and-overlay-precedence.md) | The acceptance clause's remaining inventory, one overlay-precedence source, and slice 8's two corrections. **Complete** — `83b5ffc`; closed **D-23** and **D-24**'s confirmation half, corrected the capture geometry for the whole cycle, and made the census a test. **D-22 stays open**, narrowed: the one source reports overlays the current view does not draw. |
| 10 | [`035-m12-slice10-view-aware-overlays.md`](./035-m12-slice10-view-aware-overlays.md) | **The last.** Overlays the view actually draws, the confirm dialog drawn wherever it is raised, one source per method table. **Complete** — `9d405e1`; closed **D-22** and **D-26**, and **MK-023 moved to `done/`**. Its capture was run by the architect in review, the dev team's rig having been blocked by niri's asynchronous screenshot write. |

Slice numbering follows the design section's list; the file-name number is the
dev-team task number, which review requests cite.
