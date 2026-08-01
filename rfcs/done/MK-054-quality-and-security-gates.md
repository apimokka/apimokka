# RFC MK-054 — Quality and security gates

**Status.** Implemented (Unreleased)
**Tracks.** Stabilization roadmap M4 — Quality and security gates.
**Touches.** Rust warnings and lints, the iced feature surface, `Cargo.lock`,
release-gate automation, security evidence, and M4/R1 documentation.

## Summary

Make the repository's release-decision gates genuinely green and repeatable.
Implementation removes warnings instead of allowing them, removes the optional
iced syntax highlighter, updates the two compatible transitive packages that
currently hold `quick-xml` below its patched release, and adds one
repository-owned script that runs the accepted stable, Rust 1.91, security, and
governance checks.

The dependency investigation found a direct remediation path. Both
`wayland-scanner` 0.31.11 and `plist` 1.10.0 accept `quick-xml` 0.41; Rust 1.91
is new enough for both packages. M4 therefore adopts `quick-xml` 0.41 or newer
through compatible transitive updates and permits no RustSec vulnerability
exception. Removing iced's `highlighter` feature also removes a single cosmetic
use and its `syntect`/`two-face` asset stack. It does not remove the fallback
JSON editor or narrow Linux X11/Wayland support.

M4 ends in one integrated implementation review. Intermediate correction loops
are ordinary development work unless they reveal a design change. R1 remains a
separate independent architecture re-review after M4 evidence freezes.

## Context and observed baseline

The accepted M3 evidence leaves two truthful non-green gates:

- strict workspace Clippy stops in `apimokka-model` on three
  `field_reassign_with_default` findings and one `derivable_impls` finding; and
- `cargo audit` reports RUSTSEC-2026-0194 and RUSTSEC-2026-0195 against
  `quick-xml` 0.39.4, plus seven non-blocking RustSec warnings.

Because Clippy stops at the model crate, four findings are only the known first
layer. The app already emits established warnings when linted without
`-D warnings`. Implementation must continue the strict lint/fix cycle until the
complete workspace and all targets pass; it must not treat the initial four as
the full inventory.

The current vulnerable package has two paths:

```text
iced highlighter -> iced_highlighter -> two-face -> syntect -> plist 1.9.0
iced default Wayland support -> wayland-scanner 0.31.10
```

Both paths select `quick-xml` 0.39.4. The first exists only because the app
enables iced's `highlighter` feature and calls `.highlight(...)` once in the
fallback JSON editor. The second is a Wayland protocol build-time proc-macro
path and is retained because Wayland support is part of iced's default desktop
surface.

Design-time Cargo metadata and dry runs observed on 2026-07-23 established:

| Package | Current | Compatible target | Target `quick-xml` requirement | Target Rust version |
|---|---:|---:|---:|---:|
| `wayland-scanner` | 0.31.10 | 0.31.11 | `0.41` | 1.71 |
| `plist` | 1.9.0 | 1.10.0 | `0.41.0` | 1.88 |
| `quick-xml` | 0.39.4 | 0.41.0 or newer compatible release | patched by advisory definition | 1.79 |

`cargo update --dry-run` accepted each target without a manifest override. This
is feasibility evidence only: the Proposed RFC does not change `Cargo.lock` and
does not claim that the advisories are already resolved.

## Goals

1. Make warnings fatal and green across workspace library, binary, example,
   benchmark, and test targets on current stable.
2. Preserve behavior while fixing compiler and Clippy findings.
3. Remove both `quick-xml` vulnerabilities without a fork, source patch,
   platform reduction, or security exception.
4. Remove the unjustified highlighter dependency surface while retaining a
   usable plain-text JSON editor.
5. Verify locked tests and builds on stable and Rust 1.91.
6. Give developers and R1 one canonical local command with explicit evidence.
7. Preserve the existing matcher-oracle and RFC-integrity controls.

## Non-goals

- Refactor oversized modules or test files; that is M5.
- Change application behavior, workflows, localization, the M3 port contract,
  or production-integration boundaries; no visual redesign or visual change is
  included beyond the explicitly approved removal of syntax colouring from the
  fallback JSON editor.
- Remove Wayland or X11 support, disable iced defaults, or change renderers.
- Deny every RustSec warning category. The accepted roadmap blocks
  vulnerabilities through `cargo audit`; unmaintained and unsound warnings are
  inventoried for R1 but are not silently promoted to a new M4 release policy.
- Upgrade unrelated direct or transitive packages merely because newer versions
  exist.
- Add a source fork, `[patch.crates-io]`, vulnerability ignore, or exception
  registry when compatible patched packages solve the active advisories.
- Create a release, version bump, commit, tag, or push.

## Ownership and review boundary

The project owner is accountable and approves evidence. Codex is the proposed
delivery implementer, subject to explicit project-owner authorization after
this design is accepted. An independent security/release reviewer reviews the
integrated implementation and gate evidence. R1's architecture reviewer remains
independent of this M4 delivery decision.

No separate implementation handoff is needed: this RFC contains a short,
ordered implementation plan and its verification contract. If review expands
the work beyond that plan, add a handoff before implementation rather than
embedding a second project plan in review correspondence.

## Decision

### 1. Clear warnings without lint suppression

Production and test code is rewritten until this command succeeds:

```sh
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

The initial mechanical corrections use struct update syntax for the three
model defaults and derive `Default` for `RulePayload`. After those unblock the
app crate, every newly exposed finding is fixed and the command is rerun until
green.

`#[allow(...)]`, workspace lint-level relaxation, feature omission, target
omission, or a warning baseline file is not an M4 fix. A narrowly necessary
allowance would be a design amendment identifying the exact item and why code
cannot express the invariant more clearly.

Corrections must remain behavior-preserving. Existing tests are the primary
regression evidence; focused tests are added only when a lint rewrite exposes
an untested semantic choice. M4 does not restructure modules to make lint work
look cleaner.

### 2. Remove the iced highlighter feature

Remove `highlighter` from the workspace iced feature list and remove the sole
`.highlight("json", ...)` call. Keep the same `text_editor` content, editing,
copy, and save behavior as plain text.

This decision is proportionate because syntax color is cosmetic in one fallback
surface, while the feature brings the `iced_highlighter`, `two-face`, `syntect`,
`plist`, `bincode`, and `yaml-rust` chain. It currently contributes one
vulnerable path and two of the seven allowed RustSec warnings. The implementation
review must show that `iced_highlighter`, `syntect`, and `plist` are absent from
the resolved graph. Their absence is evidence, not a new permanent package
denylist.

No visual redesign follows. Existing screen tests plus a focused assertion that
the fallback editor still renders and edits JSON are sufficient; a screenshot
campaign is not required for deleting syntax color.

### 3. Update the retained Wayland build path

After feature removal, update `wayland-scanner` within its existing compatible
0.31 line to 0.31.11. Its declared dependency accepts patched `quick-xml` 0.41,
and its Rust 1.71 minimum is below the workspace MSRV. Let Cargo prune packages
made unreachable by highlighter removal; do not hand-edit `Cargo.lock`.

The reviewed resolution must satisfy all of these conditions:

- exactly one resolved `quick-xml` package remains;
- its version is at least 0.41.0 and comes from crates.io;
- its only expected reason in the active graph is the retained
  `wayland-scanner` build path;
- `wayland-scanner` is 0.31.11 from crates.io;
- no `[patch]`, alternate registry, Git dependency, vendoring, or advisory
  ignore is introduced; and
- X11 and Wayland remain enabled in the resolved iced feature graph.

The dependency change is accepted only after both stable and Rust 1.91 locked
tests/builds pass. If Cargo cannot produce this resolution or either toolchain
fails, implementation stops for an RFC amendment. It must not fall back to an
exception automatically.

### 4. Keep security policy explicit

The blocking security command remains:

```sh
cargo audit
```

It must exit zero with no vulnerability findings. The implementation evidence
also records every allowed warning by advisory ID, package, version, category,
and dependency path. Removing the highlighter should remove the current
`bincode` and `yaml-rust` warnings; the final count is observed rather than
predicted.

M4 approves no `--ignore` argument. Any vulnerability left after the planned
updates, or any new one disclosed before evidence freeze, blocks completion and
requires either remediation or an independently reviewed RFC amendment with the
roadmap's full path/owner/approval/expiry/trigger record and a machine check.

### 5. Add one canonical release-gate script

Add `scripts/check-release-gates.sh` and
`scripts/check-release-gates-self-test.sh`. The gate runs from the repository
root, stops on the first failure, uses a repository-local `TMPDIR` when none is
supplied, and prints the tool versions and each command before execution. It
does not install tools, fetch arbitrary scripts, alter manifests, update the
lockfile, or encode expected test counts.

Its complete ordered external-command contract is:

```sh
bash --version
git --version
rustc --version
cargo --version
rustc +1.91 --version
cargo +1.91 --version
cargo fmt --version
cargo clippy --version
cargo audit --version
mkdir -p "$TMPDIR"
cargo fmt --all -- --check
cargo test --workspace --lib --bins --locked
cargo test --workspace --doc --locked
cargo build --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.91 test --workspace --lib --bins --locked
cargo +1.91 build --workspace --locked
cargo audit
bash scripts/check-matcher-oracle-self-test.sh
bash scripts/check-matcher-oracle.sh
bash scripts/check-rfcs-self-test.sh
bash scripts/check-rfcs.sh
git diff --check
```

Before substantive gates begin, the script uses shell command discovery for
Bash, Git, `rustc`, Cargo, and `mkdir`, then runs all nine version probes above
and prepares the selected `TMPDIR`. The
stable Rust/Cargo pair is the default toolchain; the `+1.91` probes prove the
declared MSRV compiler and Cargo pair. `cargo fmt --version`, `cargo clippy
--version`, and `cargo audit --version` prove that rustfmt, Clippy, and
cargo-audit are installed and callable. A missing command, toolchain, or
component, a failed version probe, an unsupported argument to the gate script,
or failure to prepare its selected temporary directory exits 2 before
the first substantive `cargo fmt` gate. Once substantive gates begin, the
script stops immediately and propagates the exact nonzero status from the
failed command. It never converts failure to success.

Root discovery uses Bash facilities rather than adding an unrecorded external
command. The self-test stubs every external command used by the gate, including
`mkdir`. Each stub writes an
argv-safe invocation record consisting of the command name, argument count, and
every argument as NUL-delimited fields. For a successful run, the self-test
compares the complete ordered log against a separately constructed expected
log for the exact command contract above. It rejects an omitted, duplicated,
reordered, or argument-mutated invocation, including loss of `--locked`,
`--all-targets`, `--all-features`, or `-D warnings`.

Targeted self-test cases inject distinctive nonzero statuses into the first
substantive gate (`cargo fmt`), a middle gate (`cargo clippy`), and the final
gate (`git diff --check`). Each case proves that the exact status is returned
and no later invocation occurs. Additional cases cover invocation from outside
the repository root, an unsupported script argument, a missing external
command, missing Rust 1.91, and unavailable rustfmt, Clippy, and cargo-audit
components. Tool failures must return 2 before substantive gates begin.

This remains a lightweight shell self-test: stubs and byte-for-byte invocation
log comparison replace real builds. The self-test itself is run directly before
the real gate; the gate does not recursively invoke its own self-test.

This script is the repeatable local gate requested by M4. Adding hosted CI is
not required for a mockup repository that currently has no CI configuration.
Future CI should call this script rather than duplicate its command list.

### 6. Freeze truthful M4 evidence for R1

After the integrated candidate is stable, run the canonical script once without
stubs. Record the date, stable Rust/Cargo versions, Rust 1.91 versions,
cargo-audit version and advisory database revision, command exits, actual test
counts, dependency identities/paths, and allowed RustSec warning inventory in
`docs/src/development-and-gates.md`.

Update `ROADMAP.md` to `In review` only after implementation and evidence are
ready. It becomes `Complete` only after the independent security/release review
and project-owner acceptance. Do not claim R1 GO, release eligibility, or a new
release from the M4 implementation review.

## Implementation sequence

1. Accept this design and explicitly authorize the proposed implementer.
2. Remove the highlighter call and feature; update the focused fallback-editor
   test and resolve/prune the lockfile.
3. Update `wayland-scanner` to 0.31.11 and verify the exact dependency graph on
   stable and Rust 1.91 before unrelated code cleanup.
4. Fix the known model lints, then iterate strict all-target/all-feature Clippy
   through every newly exposed warning until green.
5. Add the canonical gate script and its small fail-fast/tool-discovery
   self-test.
6. Run the complete script, capture exact evidence, and request one integrated
   implementation/security review.
7. After acceptance, perform the ordinary RFC lifecycle closure and M4 evidence
   finalization before freezing R1 inputs.

Steps 2–5 may be one implementation changeset. Separate review requests are not
required unless a dependency incompatibility or semantic lint correction
changes this RFC's design.

## Acceptance evidence

The implementation review package must reference this RFC, `ROADMAP.md`, the
accepted M3 implementation and closure reviews, and the relevant external
integration specifications so a reviewer new to the design can reconstruct the
boundary. It must list every changed path and include observed output for:

- the canonical release-gate script and its self-test;
- `cargo tree --locked -e features -i quick-xml`;
- negative resolved-graph checks for `iced_highlighter`, `syntect`, and `plist`;
- the retained iced X11 and Wayland features;
- the `Cargo.lock` package blocks for `quick-xml` and `wayland-scanner`,
  including crates.io `source` and checksum fields, plus locked Cargo metadata
  and a manifest sweep showing no patch, Git, vendored, or alternate-registry
  source override;
- `cargo audit`, including allowed warnings rather than only its exit code; and
- focused fallback-editor behavior coverage; and
- explicit `git diff --no-index --check /dev/null -- <path>` results for both
  new release-gate scripts while they are untracked. Exit 1 with no whitespace
  diagnostics is the expected new-file result. After those files are committed,
  the canonical tracked `git diff --check` gate covers later edits.

Expected acceptance outcome:

- no compiler or Clippy warnings in the declared stable gate;
- all stable and Rust 1.91 tests/builds pass with `--locked`;
- no RustSec vulnerability remains;
- no vulnerability exception or source override exists;
- plain-text fallback JSON editing remains usable; and
- repository governance checks still pass.

## Alternatives considered

### Retain highlighter and update `plist`

Technically viable: `plist` 1.10.0 accepts `quick-xml` 0.41 and Rust 1.88. It is
rejected because one cosmetic use does not justify retaining the larger parser
and serialized syntax-asset chain or its two allowed RustSec warnings.

### Disable Wayland

Rejected. It would remove an active Linux desktop backend to avoid a build-time
dependency even though a compatible patched `wayland-scanner` exists.

### Add a time-bounded vulnerability exception

Rejected for this design. The compatible remediation is available and dry-run
resolution succeeded. An exception would add policy machinery while leaving two
known high-severity advisories in the lockfile.

### Patch or fork transitive crates

Rejected. Official compatible releases already provide the necessary version
constraints, so a source override would increase supply-chain and maintenance
risk.

### Add hosted CI now

Deferred. One repository-owned script satisfies the roadmap and gives later CI
a single entry point without choosing a hosting platform in M4.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Later Clippy layers are larger than the four known findings | Iterate until the actual workspace command is green; no fixed warning-count estimate |
| Lint cleanup changes behavior | Prefer mechanical rewrites; use existing and focused tests; amend design for semantic choices |
| Dependency update breaks MSRV or desktop backends | Verify both toolchains and resolved X11/Wayland features before acceptance |
| Audit database changes during implementation | Record revision and inventory at evidence freeze; new vulnerabilities block or require amendment |
| Gate script diverges from roadmap | Treat this RFC's explicit command contract as M4 authority and have future CI call it |
| M4 expands into structural cleanup | Leave file splitting and broad refactoring to M5 |

## Review questions

1. Is removing the one highlighter use the right tradeoff for its dependency and
   advisory-warning footprint?
2. Is the compatible `wayland-scanner`/`quick-xml` update sufficiently bounded
   without a new lockfile oracle script?
3. Is prohibiting a vulnerability exception appropriate now that an official,
   MSRV-compatible resolution exists?
4. Does the canonical script cover the M4/R1 evidence contract without adding
   unnecessary CI or project-management machinery?
5. Is one integrated implementation review sufficient for this bounded M4
   changeset?

Implementation was not authorized merely by creation of the proposed RFC; the
project owner authorized it separately after design acceptance.
