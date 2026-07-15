# RFC MK-051 — Repository truth and RFC integrity

**Status.** Proposed
**Tracks.** Stabilization roadmap M1 — Repository truth.
**Touches.** RFC metadata and index, changelog/roadmap alignment, maintainer
documentation, historical-document warnings, a repository-owned RFC checker,
and M1 baseline evidence.

## Summary

Restore the live repository as a coherent source of truth before behavior and
integration-boundary work begins. The change is documentation and governance
only: it repairs lifecycle/version records, records the snora de-vendoring
transition, removes stale architecture claims, labels historical design files
inside those files, and adds a deterministic RFC integrity checker with
self-tests.

This RFC is the bootstrap exception required by M1. It is created in
`rfcs/proposed/` and added to the Proposed index before the existing index is
fully repaired. Its implementation will rebuild the entire index, including
MK-051 itself.

M1 also captures truthful baseline gate evidence on current stable, Rust 1.91,
and the dependency audit. A failing baseline is recorded as a known input to
later milestones; M1 does not silently broaden into matcher, model, lint, or
dependency remediation.

## Context

The architecture preparation review found that the repository's records do not
currently agree:

- `rfcs/README.md` omits implemented MK-046 through MK-050;
- the same index omits archived MK-001 through MK-020;
- MK-042 through MK-045 use future v0.11.0–v0.14.0 claims even though the
  changelog records their implementation in v0.9.11–v0.9.14;
- `ROADMAP.md` and `docs/src/architecture.md` contained stale capability and
  source-layout information;
- RFC MK-050 and the v0.10.0 changelog describe vendored snora, while the live
  workspace uses registry snora and has no `vendor/` workspace members; and
- the RFC lifecycle policy has no automated enforcement in this repository.

The approved stabilization roadmap makes this repair M1, before new behavioral
RFCs. M1 is not permission to rewrite historical design intent. It corrects
metadata, indexes, and descriptions only where evidence is available.

## Goals

1. Give contributors one explicit authority rule for each kind of repository
   fact.
2. Reconcile every RFC file with its lifecycle folder and the index.
3. Correct historical shipped-version claims without inventing evidence.
4. Record the current dependency transition from vendored to registry snora.
5. Replace stale maintainer documentation with current, reproducible facts.
6. Make common RFC drift fail a deterministic local check.
7. Capture the M1 quality/security baseline without claiming failures passed.

## Non-goals

- Correct match-test semantics (M2).
- Change the GUI editing boundary or data model (M3).
- Fix clippy warnings, MSRV failures, advisories, or dependency features (M4).
- Split Rust implementation/test modules (M3 preparation and M5).
- Change product behavior, UI copy, dependencies, or release artifacts.
- Create a release, commit, tag, archive, or push.
- Rewrite superseded RFC decisions to match the current implementation.

## Authority model

When records disagree, use the following authority by fact type. There is no
single file that is authoritative for every question.

| Fact | Authority | Reconciliation rule |
|---|---|---|
| Current workspace version | root `Cargo.toml` | Documentation describing the current tree must match `[workspace.package].version` |
| Historical shipped version | `CHANGELOG.md`, corroborated by Git history when ambiguous | RFC/index versions must name the first release that actually contains the implementation; never infer from RFC number |
| RFC lifecycle state | containing folder | `proposed/` = Proposed, `done/` = Implemented, `archive/` = Withdrawn or Superseded |
| RFC identity/title | filename ID plus first heading | Number must agree; slug/title wording may differ without changing identity |
| Current dependency source/version | root `Cargo.toml` plus `Cargo.lock` | Historical release notes remain historical; an Unreleased entry records later transitions |
| Current source layout/metrics | files in the live tree | Maintainer docs use reproducible commands and dated observations, not copied handoff values |
| Programme status | `ROADMAP.md` | RFCs may reference a milestone but do not override its status |
| Historical design intent | the RFC body at the time of decision | Correct metadata/links only; preserve superseded alternatives and rationale |

If `CHANGELOG.md` and a bounded Git-history search cannot establish a shipped
version, use `Implemented (version unverified)` in the RFC and `Unverified` in
the index, then record the missing evidence. The bounded search records:

- changelog headings/lines inspected;
- `git log --follow -- <rfc-file>` from the file's first visible commit to
  `HEAD`;
- `git log -S 'MK-NNN' -- CHANGELOG.md` over repository history; and
- why those results do not establish a release.

Git history must be consulted when the changelog is ambiguous, but it is not
required to produce an answer. Do not guess.

## Reconciliation inventory

Implementation begins with a read-only inventory table in the review package.
For every `MK-*.md` under `rfcs/proposed/`, `rfcs/done/`, and `rfcs/archive/`,
record:

- ID, filename, title, folder-derived state, and Status field;
- version/replacement/withdrawal metadata where applicable;
- whether the index contains the exact path once;
- version evidence source;
- inbound/outbound broken local links; and
- the proposed correction, if any.

Known required reconciliation includes, but is not limited to:

- adding MK-046 through MK-050 to Implemented;
- adding MK-001 through MK-020 to Archive;
- correcting MK-042 through MK-045 using their v0.9.11–v0.9.14 changelog
  evidence;
- adding proposed MK-051;
- auditing MK-000's differing version claim rather than selecting one by
  memory; and
- preserving MK-031 as Withdrawn.

The inventory is review evidence, not a new parallel status database. The
corrected RFC files and index remain the durable result.

## RFC metadata rules

### File and heading

- Project RFC filenames match `MK-NNN-lowercase-hyphen-slug.md`.
- `NNN` is unique across all lifecycle folders and is never reused.
- The first heading starts `# RFC MK-NNN —` with the same number as the file.

### Status field

The opening metadata block—from the first heading up to, but not including, the
first `##` section—contains exactly one `**Status.**` line:

- `proposed/`: `**Status.** Proposed`;
- `done/`: `**Status.** Implemented` plus a verified version when available;
- `archive/`: one of these exact forms:
  - `**Status.** Withdrawn — <non-empty reason>`;
  - `**Status.** Superseded by RFC MK-NNN` with optional
    ` — <non-empty reason>`; or
  - `**Status.** Superseded by RFCs MK-NNN–MK-NNN` with optional
    ` — <non-empty reason>`.

The series form uses one Unicode en dash and denotes an inclusive, ascending,
non-empty range. The checker expands the range and verifies that every
replacement ID exists, that the range does not include the superseded RFC
itself, and that start is not greater than end. Empty/malformed ranges and
missing replacement IDs fail. M1 preserves the evidence-backed
MK-021–MK-037 redesign-series relationship for MK-001–MK-020 unless the
inventory finds a more precise historical mapping.

Folder state wins during reconciliation, but implementation must update the
Status field in the same patch so no disagreement remains.

### Index

`rfcs/README.md` lists every RFC exactly once under its folder-derived state.
Each entry links to the real relative path. The index is a view of the files,
not a second source of lifecycle truth.

Section boundaries are H2 headings whose labels begin `## Proposed`,
`## Implemented`, and `## Archive`, each extending through the next H2 heading.
A proposed RFC link is valid only in Proposed, a done RFC link only in
Implemented, and an archived RFC link only in Archive. An RFC entry outside its
required section fails even if the path exists. A duplicate across sections
fails both uniqueness and placement.

The Proposed section may contain an explicit empty-state sentence only when
`rfcs/proposed/` has no RFC files.

## Documentation repair

### RFC index and files

- Rebuild all Proposed, Implemented, and Archive tables from the inventory.
- Correct Status/version fields only with evidence described above.
- Update local cross-references affected by lifecycle paths.
- Do not renumber, delete, or silently merge historical RFCs.

### Changelog and snora transition

Add an `Unreleased` entry that records the already-observed source transition:

- snora is now resolved from the registry;
- the live workspace no longer includes vendored snora members; and
- the locked snora version is reported from `Cargo.lock` at implementation
  time.

The v0.10.0 entry remains unchanged as a historical statement about that
release. M1 does not assign a new release version.

### Architecture and roadmap documentation

- Replace nonexistent/outdated file references and line-count claims in
  `docs/src/architecture.md`.
- Prefer reproduction commands (`find`, `wc -l`, or the repository checker)
  over approximate counts likely to drift.
- State the current oversized-file findings and link to roadmap M3/M5 rather
  than calling them acceptable under the present rules.
- Keep the approved stabilization roadmap content intact except for normal M1
  status/evidence updates.

### Historical documents

Add a prominent warning immediately after the title of both:

- `docs/src/designer-brief.md`; and
- `docs/src/ux-redesign.md`.

The warning says the document is historical, names its design era, and directs
readers to `ROADMAP.md`, current RFCs, and `docs/src/architecture.md`. Do not
rewrite historical body content such as old snora versions; the warning is what
prevents accidental current use.

### Documentation index

Update `docs/src/README.md` only as necessary to link current maintainer/gate
guidance and keep the historical warning consistent. Do not add user guides
unrelated to M1.

## RFC integrity checker

### Interface

Add:

```text
scripts/check-rfcs.sh [repository-root]
scripts/check-rfcs-self-test.sh
```

`check-rfcs.sh` defaults to the repository root derived from its own location.
The optional root exists so the self-test can run against isolated fixtures.
Both scripts use `#!/usr/bin/env bash` and require Bash 4 or later. Allowed
non-shell utilities are `awk`, `find`, `grep`, `sed`, `sort`, `mktemp`, and
standard core file utilities. The implementation must not require Python,
Ruby, Node.js, `realpath`, a Markdown library, or network access. The checker is
read-only.

Exit codes:

- `0`: no integrity errors;
- `1`: one or more repository integrity errors;
- `2`: invalid invocation, a missing required utility, unreadable required
  input, or checker-internal failure. Repository content violations never use
  exit code 2.

Diagnostics are deterministic and use:

```text
ERROR <path>: <message>
RFC integrity: <N> error(s)
```

Successful output is:

```text
RFC integrity: 0 error(s)
```

### Required checks

1. RFC filenames match the project pattern.
2. No RFC number appears in more than one lifecycle folder.
3. Filename number and first-heading number agree.
4. Exactly one Status field appears in the opening metadata block.
5. Status value agrees with `proposed/`, `done/`, or `archive/` semantics.
6. Every RFC appears exactly once in `rfcs/README.md` at its real path and only
   inside the lifecycle section corresponding to its folder.
7. Every RFC link in the index resolves to an inventoried RFC; RFC entries
   outside Proposed/Implemented/Archive fail.
8. The Proposed empty-state sentence agrees with whether proposed files exist.
9. Superseded Status values match the single-target or inclusive-series grammar,
   and every referenced replacement ID exists.
10. Relative Markdown file links inside RFCs resolve after anchors are removed.
   External URLs, email links, pure anchors, and links inside fenced code blocks
   are ignored.
11. If `rfcs/handoffs/` exists, every `MK-NNN-*` handoff directory maps to an
    existing RFC ID and contains no duplicate lifecycle folders.

### Supported Markdown-link subset

The checker is not a general Markdown parser. It supports single-line inline
links and images in either form:

```text
[label](relative/path.md#optional-fragment)
[label](<relative path with spaces.md#optional-fragment>)
```

The same destination forms apply to `![alt](...)`. Bare destinations may not
contain whitespace or parentheses. Angle-bracket destinations may contain
spaces and shell metacharacters but not a literal closing `>`. Triple-backtick
fenced blocks are excluded from link scanning; a fence begins or ends when the
first non-whitespace characters are three backticks. Tilde fences,
reference-style links, multiline destinations, nested parentheses, and link
titles are outside the automated subset. The M1 inventory must normalize any
current local RFC link outside the subset or record it for manual checking
before the checker can pass.

The parser treats backticks, `$()`, semicolons, whitespace, glob characters,
and leading dashes inside supported destinations only as path data. It must not
execute content extracted from Markdown, use `eval`, expand globs from content,
or treat code-fence text as shell input.

### Self-test

`scripts/check-rfcs-self-test.sh` builds minimal disposable repositories under a
workspace-local temporary directory and verifies at least:

- valid fixture passes;
- missing index entry fails;
- duplicate number fails;
- folder/Status mismatch fails;
- filename/heading mismatch fails;
- valid single-target supersession passes;
- valid inclusive-series supersession passes;
- malformed/reversed/empty series and a missing replacement ID fail;
- a valid RFC path under the wrong index section fails;
- a duplicate entry across lifecycle sections fails;
- broken relative Markdown link fails; and
- incorrect Proposed empty state fails;
- injection-shaped link destinations containing backticks, `$()`, semicolons,
  spaces, glob characters, and leading dashes are checked as literal data and
  create/execute nothing;
- a broken link inside a triple-backtick fence is ignored while a real broken
  link immediately after the fence fails; and
- cleanup leaves a sentinel sibling outside the uniquely created temporary
  directory untouched.

The self-test owns a uniquely created temporary directory, quotes every path,
and removes only that directory via a trap. Before removal, cleanup verifies the
path is non-empty, is below the selected temporary base, and matches the
invocation's generated prefix; otherwise it exits 2 without removal. A sibling
sentinel proves cleanup cannot escape. The test does not depend on writable
system `/tmp`; default temporary storage is under `target/tmp/` unless `TMPDIR`
is explicitly supplied.

## Baseline gate evidence

M1 runs the approved programme baseline without altering Rust code or the
dependency graph:

```sh
rustc --version
cargo --version
cargo audit --version
cargo fmt --check
cargo test --workspace --lib --bins --locked
cargo build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.91 test --workspace --lib --bins --locked
cargo +1.91 build --workspace --locked
cargo audit
scripts/check-rfcs-self-test.sh
scripts/check-rfcs.sh
git diff --check
```

Evidence records command, toolchain version, exit status, and a concise result.
For `cargo audit`, it also records the advisory database update timestamp and
database Git revision. If the installed cargo-audit does not expose a revision,
the implementation checks the public advisory database checkout under the Cargo
home; if neither source exists, the durable summary says `revision unavailable`
and records why. Raw failure detail is preserved in the review package when
useful. M1 explicitly expects that clippy or audit may still fail based on the
preparation review. Those failures are truthful M4 inputs, not M1 failures,
unless the command could not run or the observed result is misreported.

A concise durable summary is added at
`docs/src/development-and-gates.md` and linked from `docs/src/README.md`. Its
dated table records command, tool/toolchain version, exit result, concise
finding, and owning follow-up milestone. It is explicitly a 2026-07 M1
baseline, not a permanently current pass badge. Raw logs may remain in the
ignored review package, but the only baseline record must not live solely under
`.git-exclude/`.

The two RFC-checker commands must pass for M1. Documentation-only M1 changes must
also pass `git diff --check`. Rust commands are baseline observations; no claim
of green release readiness is made until M4/R1.

### Whitespace coverage for new files

`git diff --check` covers modified tracked files but not untracked additions.
Before each design/implementation review, evidence therefore includes:

1. `git diff --check` for tracked changes; and
2. `git diff --no-index --check /dev/null -- <file>` for every newly created
   expected tracked file listed by the review package.

For a clean new file, the no-index command normally returns 1 because the file
differs from `/dev/null` while producing no whitespace diagnostic. Evidence
accepts only exit 0 or 1 with empty diagnostic output; exit greater than 1 or
any output fails. Paths are passed after `--` and quoted. The review package
lists the complete new-file set so ignored review artifacts and unrelated user
files do not define the gate. Intent-to-add/staging is not required.

## Implementation sequence

1. Capture the read-only RFC and documentation conflict inventory.
2. Confirm historical version evidence from changelog and Git history.
3. Repair RFC Status fields, cross-links, and the complete index.
4. Record the snora transition and repair current maintainer/historical docs.
5. Add the checker and its negative/positive self-tests.
6. Run the M1 baseline commands and record exact evidence.
7. Update M1 to `In review` and prepare the Proposed implementation-candidate
   review package.
8. Independent review accepts or rejects that Proposed implementation candidate.
9. After acceptance, the project owner explicitly authorizes lifecycle closure
   and the known stability marker `Implemented (Unreleased)`. Apply the closure
   candidate: move MK-051 to `done/`, set
   `**Status.** Implemented (Unreleased)`, update the index and roadmap evidence,
   but leave M1 `In review` with `closure confirmation pending`.
10. Rerun the RFC checker, checker self-test, tracked/new-file whitespace gates,
    and capture the final closure diff/evidence.
11. An independent reviewer performs a short closure confirmation over the
    final durable state and evidence.
12. The project owner, as evidence approver, records acceptance and only then
    changes M1 to `Complete`.

`Implemented (Unreleased)` is the stability marker for accepted work present in
the repository but not cut in a release. It must not be replaced with
`version unverified`; that phrase is reserved for historical work whose release
evidence is genuinely missing. A future verified release updates MK-051 and its
index row to the real version through the normal release workflow.

## Expected tracked files

The exact reconciliation set is inventory-driven. Expected files are:

- `rfcs/proposed/MK-051-repository-truth-and-rfc-integrity.md` (then moved to
  `rfcs/done/` only after acceptance);
- `rfcs/README.md`;
- RFC files whose Status/version/link metadata is proven stale;
- `CHANGELOG.md`;
- `ROADMAP.md`;
- `docs/src/README.md` and `docs/src/architecture.md`;
- `docs/src/development-and-gates.md`;
- `docs/src/designer-brief.md` and `docs/src/ux-redesign.md`;
- `scripts/check-rfcs.sh`; and
- `scripts/check-rfcs-self-test.sh`.

No file under `crates/`, no Cargo manifest, and no lockfile should change.

## Security and operational considerations

- The checker reads repository-controlled Markdown only and performs no network
  access.
- All content-derived strings are treated as data, quoted, and never executed.
- Temporary self-test cleanup is constrained to the directory created by that
  invocation.
- The checker must not read `.git-exclude/`, release artifacts, secrets, or
  environment credentials.
- Baseline audit output may name public advisories and dependency paths; it must
  not print environment variables or infer secrets.
- No threat-model update is required because M1 introduces no product data flow,
  auth, file-loading behavior, or external integration.

## Alternatives considered

### Repair documents manually without a checker

Rejected. It fixes today's drift but gives no early warning when an RFC moves or
a new RFC is omitted from the index.

### Generate the entire RFC index automatically

Deferred. Generation would need title/reason/version extraction rules and could
silently rewrite human-curated context. A read-only checker is smaller and more
reviewable; generation can be proposed later if maintenance cost justifies it.

### Make one file globally authoritative

Rejected. Lifecycle state, current package version, historical shipped version,
and current dependencies are different fact types with different natural
authorities. Pretending one document owns all of them creates new drift.

### Correct old RFC bodies to match current code

Rejected. Implemented and archived RFC bodies are historical design records.
Only metadata, status, links, and demonstrably false current-facing statements
are repaired.

### Fix current lint and advisory failures in M1

Rejected. That would mix governance repair with product/dependency remediation,
invalidate the documentation-only review boundary, and duplicate M4.

## Acceptance criteria

- Every RFC is inventoried and appears exactly once in the corrected index.
- RFC folder, Status field, heading number, and index path agree.
- Historical shipped versions are evidence-backed or explicitly unverified.
- MK-042 through MK-045 no longer claim v0.11.0–v0.14.0 releases.
- The current snora registry transition is recorded without rewriting v0.10.0
  history.
- Current architecture documentation contains no nonexistent source path or
  stale handoff line-count claim.
- Both historical design files carry an in-file warning.
- `scripts/check-rfcs.sh` passes on the repository.
- The checker self-test proves its required positive and negative cases.
- M1 baseline results record exact successes, failures, toolchains, and
  limitations without claiming later gates passed.
- `git diff --check` passes.
- No Rust source, Cargo manifest, lockfile, or product behavior changes.
- Independent implementation review accepts the Proposed candidate before the
  owner authorizes lifecycle closure.
- Independent closure confirmation accepts the moved RFC, final Status/index,
  checker/self-test output, and whitespace evidence before M1 becomes
  `Complete`.

## Review questions

1. Is the fact-specific authority model sufficient to reconcile version and
   lifecycle conflicts without rewriting history?
2. Should ambiguous shipped versions use `version unverified`, or is Git history
   mandatory before M1 can complete?
3. Are the checker scope, interface, exit codes, and self-test cases adequate?
4. Is recording known clippy/audit failures as baseline evidence—without fixing
   them—the correct M1/M4 boundary?
5. Is the two-stage implementation review plus closure-confirmation sequence
   compatible with the owner's review-before-commit workflow?

## Handoff decision

No separate developer handoff is proposed. The implementation is a bounded,
ordered documentation/checker change and this RFC contains the necessary file
scope, checker contract, tests, evidence commands, and acceptance criteria. If
review expands the checker into a generator or CI workflow, reconsider a small
task breakdown before implementation.
