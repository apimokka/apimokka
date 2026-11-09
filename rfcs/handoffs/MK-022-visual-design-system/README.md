# Handoffs — MK-022, visual design system

Companion execution documents for
[MK-022](../../done/MK-022-visual-design-system.md). Their lifecycle state is
**inherited from that RFC** and is not managed here.

MK-022 owns the colour roles and the rule this programme has now applied five
times: **measure the pairing on our surface; never inherit a token's own
claim.** Defect fixes against that rule get a handoff here rather than a
new RFC — the decision was made in MK-022, and what is left is execution.

| Task | Handoff | Covers |
|---:|---|---|
| 036 | [`036-d21-d25-banner-contrast-and-labels.md`](./036-d21-d25-banner-contrast-and-labels.md) | **D-21**, **D-25**, **D-27**. **Round 1 complete** — `33505c7`; D-25 and D-27 closed, the banner's fill and message text fixed and verified. |
| 036 r2 | [`036-round-2-every-colour-on-the-banner.md`](./036-round-2-every-colour-on-the-banner.md) | **D-21's remainder**: the banner's own two buttons, the rule editor's validation header, and the region cursor ring. **Complete** — `1248793`; every colour on the surface now measures 5.43 / 10.63 / 8.42 / 14.85, guarded by a census over `banner_style`'s own call sites. **D-21 closed 2026-11-04.** |
| 038 | [`038-d29-cursor-ring-in-high-contrast.md`](./038-d29-cursor-ring-in-high-contrast.md) | **D-29** — `cursor_style` painted nothing in either high-contrast preset (border width 0, alpha 0), so the keyboard cursor was invisible application-wide there. Found by MK-056's own mandatory combination 2. **Complete** — `3ac0e20`, accepted 2026-11-08: `cursor_style` owns its 2px `primary.base` border, measured 10.25–11.75:1 in the high-contrast presets, guarded by one test over both cursor treatments. **D-29 closed.** |
| 039 | [`039-d30-selection-in-high-contrast.md`](./039-d30-selection-in-high-contrast.md) | **D-30** — in high contrast *selection* is the barely-drawn state: a 1.35–1.60:1 fill wash and **no border** where every unselected row carries 1.5px at ~21:1. Found by the task 038 review's own measurement. **S2**, does not block M6 booking. **Complete** — `7f26ba1`, accepted 2026-11-08: both selected styles take a shared 3 px token border where unselected carries 1.5 px, so the cue is width and primary stays the cursor's; guarded by a rule (fill ≥3:1 **or** border ≥+1 px at ≥3:1) that fails on the pre-fix styles. The cursor guard's fourth ground (`chip_style`'s fill) rode along. **D-30 closed.** |
| 042 | [`042-d36-text-field-focus.md`](./042-d36-text-field-focus.md) | **D-36 (S2)** — text-field focus is a hue change and nothing else, in every preset, clearing the 3:1 state floor only in Light (3.74; dark 2.63, hc_light 1.40, hc_dark 1.19, no width change anywhere). Found by combination 2, the combination whose purpose is focus indication. Fix: a **width** cue, as D-30 took. Should land before the first M6 session; does not block booking. **Authorized 2026-11-09.** |

Earlier defect fixes of the same family — **D-13**/**D-14** (task 021) and
**D-15** (task 023) — predate this directory and were issued before durable
handoffs moved out of `.git-exclude/`; their reviews are the record
(`.git-exclude/reviewed/2026-10-06-task021-d13-d14-review.md`,
`…/2026-10-31-task023-d15-review.md`).
