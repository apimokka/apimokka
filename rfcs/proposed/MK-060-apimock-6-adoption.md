# RFC MK-060 — apimock-rs 6.0.0 adoption

**Status.** Proposed
**Tracks.** Stabilization roadmap M11 — engine major upgrade.
**Touches.** `apimock-routing` (production, 4 items), `apimock-config`
(test-only conformance oracle), MK-052's matcher conformance, MK-055's engine
contract conformance, and MK-053's boundary mapping.
**Decided by.** Project owner, 2026-08-21 — *"No. We should. The v6 is for us."*

## Summary

Adopt apimock-rs 6.0.0, from a `Cargo.lock` pinned at 5.10.0. The release ships
a documented `docs/src/library/` section stating that **the library API was
shaped for a GUI**, plus an additive-only API gate promising that what compiles
at 6.0.0 keeps compiling across 6.x.

The architect initially recommended **against** adopting, on the cost of
re-running MK-052 and MK-055. **That recommendation was wrong and is withdrawn.**
It applied a cost argument to a correctness question, and it contradicted the
principle this programme applied four times to snora: do not validate against a
version you are about to abandon. MK-055's entire value is that our mapping is
checked against the *real* engine contract. Checked against a superseded one, it
is archaeology.

## Problem

**We are validating a boundary the production project will not use.** This
repository exists to be a trustworthy executable specification for a production
GUI. That GUI will build against 6.x. Our MK-053 boundary and MK-055 conformance
are recorded against 5.10.0.

**We are further behind than the pin suggests.** `apimock-config = "5"` and
`apimock-routing = "5"` are caret requirements; `Cargo.lock` holds 5.10.0 while
the 5.x line reached **5.19.1**. Nine minors were never taken.

**And 6.0.0 supplies what MK-053 had to infer.** MK-053 derived our editing
boundary by reading an undocumented surface and recording what it concluded.
6.0.0 documents that surface, states which parts are proven, and names the
public-API baseline as authoritative over the prose.

## Sizing — measured against the published baselines, not estimated

`docs/src/library/README.md` states the checked-in baselines are authoritative
over the documentation. Fetched and compared against every engine symbol this
repository references:

| | |
|---|---|
| Distinct engine symbols referenced | **89** |
| Present in the 6.x baseline | **89** |
| Absent | **0** |

**Nothing we use was removed or renamed.**

**State the limit of that check precisely:** it matches symbol *names*, not
signatures. A struct can keep its name and change its fields; an enum variant
can keep its name and change its payload. So this bounds the upgrade to
*compilation and semantics*, not to *nothing to do*. The compile is the real
test and it is cheap; the conformance re-runs are the actual work.

### Where the work is

| Surface | Our use | Cost |
|---|---|---|
| `apimock-server` | **zero references anywhere** | none |
| `apimock-routing` | 4 items, our only production dependency | small; MK-052 re-verified |
| `apimock-config` | ~89 symbols, entirely `[dev-dependencies]` | MK-055's mapping re-run against 6.x |

The `apimock-config` re-run is not overhead. It is MK-055 performed against the
contract that matters.

## Goals

1. Be on the engine contract the production project will inherit.
2. Re-establish MK-052 and MK-055 conformance against 6.x, with divergences
   re-classified rather than assumed to carry over.
3. Read `docs/src/library/` as the reference MK-053 lacked, and record where our
   boundary now disagrees with a *documented* contract rather than an inferred one.
4. Change no UI behaviour that M6 has not seen, or sequence it so M6 sees the
   final shape.

## Non-goals

- **Adopting the unproven surfaces.** `has_external_changes`/`sync_from_disk`,
  `apimock_server::control`, and `TraceEmitter`'s subscription side have no
  consumer anywhere in apimock-rs. Every one maps to something this programme
  already defers — file I/O, external-edit detection, subprocess control, trace
  transport. Adoption of 6.0.0 does not change that, and this RFC does not
  smuggle them in.
- **Depending on `apimock-server`.** The library guide recommends it; we use
  none of it, and running a server is deferred.
- **Redesigning the `WorkspacePort` boundary.** MK-053's mapping is deliberately
  explicit rather than engine-isomorphic. If 6.x makes a cleaner mapping
  possible, that is a finding to record, not a change to make here.

## Decision

### 1. Bump both crates to 6, in one commit, with no source change if possible

```toml
apimock-routing = "6"   # production, crates/app
apimock-config  = "6"   # [dev-dependencies], crates/model
```

If it compiles unchanged, say so and stop. If it does not, the diff is the
finding — record what changed shape, because that is exactly what the production
project needs to know and what our 5.10.0-era boundary assumed.

### 2. Re-run MK-052 matcher conformance against 6.x

`scripts/check-matcher-oracle.sh` pins the oracle's identity. Re-run, and
re-classify rather than assume: an operator whose behaviour changed is a finding,
not a test to update until it passes.

### 3. Re-run MK-055 engine contract conformance against 6.x

MK-055 classified **nine divergences** between our mapping and the engine at
5.10.0, and found one defect on our side. **Every one must be re-derived.** A
divergence that has closed is as important to record as one that persists — and
one that closed because the engine moved toward us is evidence worth sending
back.

### 4. Read `library/` and record the delta against MK-053

Not a code change. MK-053's decisions were made without documentation; some may
now be contradicted, confirmed, or made unnecessary by a documented contract.
Record which.

### 5. Do not chase 5.19.x

We go 5.10.0 → 6.x directly. Stepping through nine unused minors buys nothing
and costs nine verifications.

## Sequencing

**After owner task 001's capture; before M9.**

- **Not before the capture.** No reason to disturb a build that has been
  reviewed and CI-green while a ten-minute human pass is pending. This upgrade
  touches no rendering, so it cannot invalidate the capture — but it also gains
  nothing by preceding it.
- **Before M9 and M10.** Both invest in the application's UI. If 6.x moves the
  editing mapping in a way that changes a screen, it should move before keyboard
  operability and typography are built on top of it — the same reasoning that
  put the snora bump ahead of the capture.
- **Well before M6.** Participants must not evaluate an editing surface we then
  change.

## Acceptance evidence

- `Cargo.lock` resolutions and checksums for both crates at 6.x.
- Compile result, stated plainly: unchanged, or the diff and why.
- MK-052 re-run, with every operator's verdict re-derived.
- MK-055 re-run, with all nine divergences re-classified — closed, persisting,
  or new.
- The MK-053 delta from reading `library/`.
- `bash scripts/check-release-gates.sh` green; CI green on all three platforms.

## Risks and mitigations

| Risk | Assessment |
|---|---|
| A type keeps its name and changes shape | **The main one.** The baseline check catches removals, not signatures. The compile is the detector, and it is cheap. |
| MK-055 divergences change | Expected, and the point. Re-classify; do not update tests until they pass. |
| The editing UI must change | Possible. Sequenced before M9/M10/M6 precisely so it surfaces first. |
| Unproven surfaces look tempting | Explicit non-goal. They map one-to-one onto this programme's deferred list. |

## What we owe apimock-rs

They asked for four things and we can now answer three:

1. **Which modules we depend on** — answered exactly; `apimock-server` is zero.
2. **The four open questions** — cannot answer. Every one concerns a surface this
   programme defers by design. Their unproven surfaces stay unproven until the
   production integration, and they should hear that plainly rather than wait.
3. **Missing `EditCommand`s / unconstructable `#[non_exhaustive]` types** —
   nothing so far, with the caveat that we map a UI to `EditCommand` without
   executing it against a real workspace. MK-055's re-run under this RFC is the
   first thing that could turn that into a real answer.
4. **Documentation errors** — one already found, see below.

### A documentation error found while sizing this RFC

`docs/src/library/README.md` states the API shapes come from checked-in
baselines at `crates/*/public-api.txt`, and that **"if this documentation and a
baseline disagree, the baseline is correct."**

Those files are **not present at the `6.0.0` tag.** They exist on the default
branch. A consumer reading the 6.0.0 documentation and looking in 6.0.0's source
finds no baseline, and therefore no tiebreaker — for a rule whose whole purpose
is to be the tiebreaker.
