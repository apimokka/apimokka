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
| 040 | [`040-d31-d32-editor-groups-overflow.md`](./040-d31-d32-editor-groups-overflow.md) | **D-31 (S1)**, **D-32 (S2)** — the Method and RESPOND mode groups clipped at 880×700. **Complete** — `c046485`, accepted 2026-11-08: both groups wrap, all six methods and both modes drawn in each locale, the old layout's clipping width measured at **~1120 logical px**, and the manifest self-test wired into the release gate. **D-31 and D-32 closed.** |
| 041 | [`041-d33-condition-rows-collapse.md`](./041-d33-condition-rows-collapse.md) | **D-33 (S1)** — the header and body condition rows collapse at 880×700; the header value field resolves to zero width and `application/json` is never drawn. Found by task 040's row walk. Option (a) decided: two lines per condition at every width. **Complete** — `6958496`, accepted 2026-11-08: `application/json` and both body values legible at 880 wide in both locales, cursor order proved unchanged by an untouched control census, nine **wgpu** runs. `test_rule.rs`'s preventive `.wrap()` landed, the bounded audit found nothing unreachable, and **D-34** closed by `mode_tab`'s `S4 → S3` padding. **D-33 and D-34 closed**; the audit's one latent finding is **D-35 (S3)**. |
