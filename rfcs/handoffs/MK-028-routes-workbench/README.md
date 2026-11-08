# Handoffs — MK-028, Routes workbench

Companion execution documents for
[MK-028](../../done/MK-028-routes-workbench.md). Their lifecycle state is
**inherited from that RFC** and is not managed here.

MK-028 owns the rule editor's structure, including its non-colour rule for
state. This directory exists because the editor's **layout** began producing
defects of its own at the smallest supported window — found by MK-056's
mandatory 880×700 combinations, which had never been run until task 037.

| Task | Handoff | Covers |
|---:|---|---|
| 040 | [`040-d31-d32-editor-groups-overflow.md`](./040-d31-d32-editor-groups-overflow.md) | **D-31 (S1)** — the Method group clips PATCH and DELETE at 880×700, unreachable by pointer; **D-32 (S2)** — the RESPOND mode group collapses, worst in Japanese. Same mechanism: a `row` of fixed-width items with no wrap. Authorization pending. |
