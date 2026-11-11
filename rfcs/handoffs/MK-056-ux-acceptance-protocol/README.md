# Handoffs — MK-056, UX and accessibility acceptance protocol

Companion execution documents for
[MK-056](../../accepted/MK-056-ux-acceptance-protocol.md). Lifecycle state is
inherited from that RFC.

Two of these are **project-owner facing** rather than dev-team facing: M6's L3
sessions are facilitated by the owner, so the execution guidance is addressed to
them. They are tracked because a production team inheriting this repository needs
the protocol as it was actually run, not only as it was designed. The third
(task 037) is dev-team work — the L2 half of the same protocol, which became
reachable only when D-12 closed.

| Handoff | Covers |
|---|---|
| [`002-m6-participant-recruitment.md`](./002-m6-participant-recruitment.md) | Personas, screening questions, what to say and not say, the one-shot first-boot rule, logistics |
| [`003-m6-session-pack.md`](./003-m6-session-pack.md) | The six scenarios as goals, coverage matrix, facilitator rules, the readability probe, severity, per-session log template. **§0 was withdrawn 2026-09-17 and restored 2026-11-04** when D-12 closed. |
| [`037-m6-l2-mandatory-combinations.md`](./037-m6-l2-mandatory-combinations.md) | **Dev team.** Decision 7's four mandatory combinations as L2 runs, and `scripts/ux/` brought onto the capture recipe that works. **Ran 2026-11-07** (`1634aed`): harness and three combinations accepted — **no clipping at 880×700, Japanese included** — but **combination 2 failed into D-29** and **combination 4 has not yet rendered the rule editor**, the density its own wording names. **Stays open** for those two re-runs; the first waits on task 038.  **Round 2 ran 2026-11-08** (`43d3cad`): §9.2 and §9.3 **accepted** — the editor and Test Rule captured at 880×700, and a frame manifest (MD5 + measured ground per frame) that fails a run on a duplicate or a contradicted theme, verified against task 039's own bad frames. The 880×700 editor opened **D-31 (S1)** and **D-32 (S2)** → task 040. **Open: §9.1 only**, combination 2's wgpu re-run, blocked on a host reboot.  **§9.1 ran 2026-11-08** (`7439269`) on **wgpu** and was accepted: combination 2 records a pass — the ring proved against round 1's own frames (+844 primary pixels on the same screen after one Tab), cursored/selected/unselected in one frame in both HC presets, and the 880×700 condition evidence captured through the application's own scroll. **Task complete; all four mandatory combinations have a recorded L2 pass.** Its last finding is **D-36 (S2)** → task 042. |
| [`043-headless-ui-level.md`](./043-headless-ui-level.md) | **Dev team.** The level the owner's 2026-11-11 rules name and this programme lacked: `iced_test`'s headless simulator for layout and interaction across size × theme × locale. **Spike first** — it is allowed to come back "no". D-31 and D-33 are both unit-testable there. Changes no pixel, so it runs during recruitment. Authorization pending. |

All three carry dated correction blocks where earlier guidance was withdrawn;
read those first.
