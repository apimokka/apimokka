# Development and gates

This page records dated stabilization evidence. It is not a continuously
updated badge and does not imply release readiness.

## M1 repository-truth baseline — 2026-07-15

The baseline was captured without changing Rust source, Cargo manifests, or
`Cargo.lock`. Current stable was Rust 1.97.0 / Cargo 1.97.0. The MSRV
toolchain was Rust 1.91.1 / Cargo 1.91.1.

| Command | Tool | Exit | Observed result | Follow-up |
|---|---|---:|---|---|
| `rustc --version` | stable | 0 | `rustc 1.97.0 (2d8144b78 2026-07-07)` | Re-run at later implementation gates |
| `cargo --version` | stable | 0 | `cargo 1.97.0 (c980f4866 2026-06-30)` | Re-run at later implementation gates |
| `cargo audit --version` | cargo-audit | 0 | `cargo-audit-audit 0.22.2` | Re-run at the programme audit cadence |
| `cargo fmt --check` | stable | 0 | Rust formatting is clean | M4/R1 final gate |
| `cargo test --workspace --lib --bins --locked` | stable | 0 | 99 tests passed (92 app, 7 model); six existing compiler warnings were emitted | Warnings are M4 quality input |
| `cargo build --workspace --locked` | stable | 0 | Workspace build passed | M4/R1 final gate |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | stable | 101 | Failed on four existing model-crate findings: three `field_reassign_with_default` and one `derivable_impls` | M4 quality remediation |
| `cargo +1.91 test --workspace --lib --bins --locked` | Rust 1.91.1 | 0 | 99 tests passed; the same six existing compiler warnings were emitted | M4/R1 MSRV gate |
| `cargo +1.91 build --workspace --locked` | Rust 1.91.1 | 101 | First attempt could not create a temporary directory because sandbox `/tmp` was read-only | Environment artifact; controlled rerun below |
| `TMPDIR=$PWD/target/tmp cargo +1.91 build --workspace --locked` | Rust 1.91.1 | 0 | Workspace build passed with repository-local temporary storage | M4/R1 MSRV gate |
| `cargo audit` | cargo-audit 0.22.2 | 1 | Two high-severity `quick-xml 0.39.4` vulnerabilities (RUSTSEC-2026-0194 and RUSTSEC-2026-0195); seven allowed warnings: five unmaintained and two unsound transitive crates | M4 security remediation |
| `scripts/check-rfcs-self-test.sh` | Bash | 0 | 25 adversarial checks passed after implementation-review hardening | Required at every RFC-governance change |
| `scripts/check-rfcs.sh` | Bash | 0 | `RFC integrity: 0 error(s)` | Required at every RFC-governance change |
| `git diff --check` | Git | 0 | No tracked-file whitespace diagnostics | Required before implementation review |
| `git diff --no-index --check /dev/null -- <new-file>` | Git | 1 for each | Clean no-index differences with zero diagnostics for the development/gates page and both checker scripts | Required before implementation review |

The audit fetched 1,160 advisories from the public RustSec database. The
database checkout was at revision
`9f3e138091487e69144f536d36976e427a7a3307`, whose commit timestamp was
`2026-07-13T19:31:41+02:00`.

The clippy and audit failures are programme baseline findings, not M1 checker
failures and not release passes. M4 owns remediation; R1 decides release
eligibility from frozen evidence.

## RFC checker runtime contract

`scripts/check-rfcs.sh` requires Bash 4 or newer and checks for the external
utilities `awk`, `basename`, `dirname`, `find`, and `sort`. Its
self-test additionally checks for `cat`, `cp`, `dirname`, `mkdir`,
`mktemp`, `rm`, `sed`, and `touch`. Other operations used by the scripts
are Bash builtins.

The checker returns 0 for a coherent repository, 1 for deterministic
repository findings, and 2 for invalid invocation, missing utilities,
unreadable required paths, or internal operational failures.

## M1 lifecycle-closure candidate — 2026-07-15

After independent acceptance of the Proposed implementation candidate, the
project owner authorized the lifecycle-closure patch. MK-051 moved to
`rfcs/done/` with `Implemented (Unreleased)`; its index entry moved to
Implemented, and the Proposed section became explicitly empty. M1 remains
`In review` with closure confirmation pending.

| Closure check | Exit | Observed result |
|---|---:|---|
| `scripts/check-rfcs-self-test.sh` | 0 | 25 checks passed |
| `scripts/check-rfcs.sh` | 0 | `RFC integrity: 0 error(s)` |
| `git diff --check` | 0 | No tracked-file whitespace diagnostics |
| `git diff --no-index --check /dev/null -- rfcs/done/MK-051-repository-truth-and-rfc-integrity.md` | 1 | Clean moved-file difference with zero diagnostics |

The no-index check covers the move destination before the closure commit; exit
1 with no diagnostics is the expected clean result. There are no other new
expected tracked files in the closure patch. Rust, dependency, clippy, audit,
and MSRV gates were not rerun because the closure patch changes lifecycle
documentation only; their M1 baseline above remains the observed record.

## M2 match-test implementation candidate — 2026-07-16

RFC MK-052 permits the `apimock-routing` 5.x and `http` 1.4 compatibility lines
in the workspace manifest. For fail-closed Test Rule conformance, `Cargo.lock`
fixes the reviewed build to `apimock-routing` 5.10.0 and `http` 1.4.2. The
lockfile and downloaded crates agreed on these checksums:

| Crate | SHA-256 / Cargo checksum |
|---|---|
| `apimock-routing 5.10.0` | `72118fbc81807a3a3e511ec638b3fc798b5eee035c8d287158ae487763003cf1` |
| `http 1.4.2` | `6970f50e31d6fc17d3fa27329444bfa74e196cf62e95052a3f6fee181dba6425` |

| Command | Exit | Observed result |
|---|---:|---|
| `cargo fmt --check` | 0 | Rust formatting is clean |
| `bash scripts/check-matcher-oracle-self-test.sh` | 0 | 6 checks passed: valid contract plus version, source, checksum, routing-feature, and HTTP-feature drift rejection |
| `bash scripts/check-matcher-oracle.sh` | 0 | Lockfile versions/sources/checksums and resolved feature sets match the reviewed oracle contract |
| `TMPDIR=$PWD/target/tmp cargo test -p apimokka match_test --locked` | 0 | 30 focused conformance/diagnostic/screen tests passed |
| `TMPDIR=$PWD/target/tmp cargo test --workspace --lib --bins --locked` | 0 | 121 tests passed (114 app, 7 model); three existing `dropping_references` warnings |
| `TMPDIR=$PWD/target/tmp cargo build --workspace --locked` | 0 | Workspace build passed |
| `TMPDIR=$PWD/target/tmp cargo +1.91 test --workspace --lib --bins --locked` | 0 | 121 tests passed; the same three warnings |
| `TMPDIR=$PWD/target/tmp cargo +1.91 build --workspace --locked` | 0 | Workspace build passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 101 | Same four M1 model findings; no MK-052 code was reached before failure |
| `cargo clippy -p apimokka --all-targets --no-deps` | 0 | Existing app warnings only; none in MK-052 paths |
| `cargo audit` | 1 | Same two `quick-xml` vulnerabilities and seven allowed warnings as M1; no MK-052-added package appeared in an advisory |
| `scripts/check-rfcs-self-test.sh` | 0 | 25 checks passed |
| `scripts/check-rfcs.sh` | 0 | `RFC integrity: 0 error(s)` |
| tracked plus new-file whitespace checks | expected | No whitespace diagnostics; each of eleven untracked files returned expected no-index status 1 |

The audit used cargo-audit 0.22.2 and loaded 1,160 RustSec advisories. Clippy and
audit remain truthful M4 inputs, not M2 passes. M2 is
`In review`; this candidate does not claim implementation acceptance or
milestone completion.

The oracle guard requires Bash 4 or newer, `awk`, `dirname`, `sort`, and Cargo.
It reads package identity from `Cargo.lock` and uses `cargo tree --locked -e
features` to require `apimock-routing` features `[default]` and `http` features
`[default, std]`. Exit 0 means the reviewed contract matches, exit 1 means
identity or feature drift, and exit 2 means the check could not run. Its
self-test additionally requires `chmod`, `cp`, `env`, `mkdir`, `mktemp`, `rm`,
and `sed`.

## M2 lifecycle-closure candidate — 2026-07-16

After independent acceptance of the implementation and compatible-manifest
oracle-guard amendment, MK-052 moved to `rfcs/done/` with `Implemented
(Unreleased)`. Its index entry moved to Implemented, and the Proposed section is
explicitly empty. M2 remains `In review` with closure confirmation pending.
Before restaging this move, the still-Proposed RFC's risk table was reconciled
to name the accepted compatible-range, committed-lockfile, and executable-
oracle-guard control instead of the superseded exact-manifest-pin policy. Its
historical implementation-authorization sentence was also recast in past tense.

| Closure check | Exit | Observed result |
|---|---:|---|
| `bash scripts/check-matcher-oracle-self-test.sh` | 0 | 6 checks passed |
| `bash scripts/check-matcher-oracle.sh` | 0 | Reviewed package identities and resolved features verified |
| `bash scripts/check-rfcs-self-test.sh` | 0 | 25 checks passed |
| `bash scripts/check-rfcs.sh` | 0 | `RFC integrity: 0 error(s)` |
| `git diff --check` | 0 | No tracked-file whitespace diagnostics |
| `git diff --no-index --check /dev/null -- rfcs/done/MK-052-test-rule-matcher-conformance.md` | 1 | Clean moved-file difference with zero diagnostics |

The closure patch changes lifecycle and evidence documentation only. Rust,
dependency, lint, audit, stable, and MSRV implementation gates were not rerun;
the accepted M2 evidence above remains the observed implementation record. The
oracle guard and its self-test were rerun because the adopted dependency
contract is part of MK-052 closure.

## M3 integration-boundary implementation candidate — 2026-07-22

RFC MK-053 replaces direct configuration mutation with the local
`WorkspacePort` mapping boundary, stable condition identity, atomic semantic
history, complete snapshot adoption/correlation, runtime request correlation,
and typed Global Save reporting. The adapter remains in memory; production
filesystem, subprocess, file-watching, merge, and trace-transport work is not
included.

The final documentation pass removes direct engine-mirroring claims from the
model and architecture descriptions, distinguishes canonical port state from
lossy render/prototype types, records the complete `ReferenceGap` inventory,
and refreshes the source-size baseline for later M5 planning.

| Command | Exit | Observed result |
|---|---:|---|
| `cargo fmt --all -- --check` | 0 | Rust formatting is clean |
| `cargo doc -q --workspace --no-deps --locked` | 0 | Model intra-doc links and workspace documentation build |
| `cargo test -q --workspace --locked` | 0 | app 187, model 55, model doctests 4 |
| `cargo +1.91 test -q --workspace --lib --bins --locked` | 0 | app 187 and model 55 on Rust 1.91.1 |
| stable and Rust 1.91 workspace builds | 0 | Both workspace builds passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 101 | Same four M1/M2 model findings; no new M3 finding before the model crate stopped the gate |
| `cargo clippy -q -p apimokka --all-targets --all-features --locked` | 0 | Established warnings only; no M3-specific finding |
| `bash scripts/check-matcher-oracle-self-test.sh` | 0 | 6 checks passed |
| `bash scripts/check-matcher-oracle.sh` | 0 | apimock-routing 5.10.0 and http 1.4.2 contract verified |
| `bash scripts/check-rfcs.sh` | 0 | `RFC integrity: 0 error(s)` |
| `bash scripts/check-rfcs-self-test.sh` | 0 | 25 checks passed |
| `git diff --check` | 0 | No tracked-file whitespace diagnostics |
| `cargo audit` | 1 | Same two high-severity quick-xml vulnerabilities and seven allowed warnings; M4 remains owner |

The audit scanned 477 locked packages against 1,167 loaded RustSec advisories.
The two vulnerabilities remain RUSTSEC-2026-0194 and RUSTSEC-2026-0195 in
`quick-xml 0.39.4`; the seven allowed warnings remain five unmaintained and two
unsound transitive crates. This is current M4 input, not a passing security
gate or a claim of release readiness.

The full M3 range from accepted M2 base `a2213ae` contains 58 tracked paths.
The inventory includes the accepted app-test extraction, model port/mapping/
memory modules, app session and behavior-focused test modules, reducer and
presentation integration, RFC records, and this documentation correction.
Each implementation slice received independent review before the next slice;
the final review evaluates the integrated range and does not supersede those
accepted checkpoint records. The integrated verdict and separate lifecycle
closure were accepted on 2026-07-23.

## M3 lifecycle closure — 2026-07-23

After independent acceptance of the integrated implementation and project-owner
acceptance of the final documentation corrections, MK-053 moved to `rfcs/done/`
with `Implemented (Unreleased)`. Its index entry moved to Implemented, the
Proposed section became explicitly empty, and the Unreleased changelog records
the delivered boundary without claiming production integration. Independent
closure confirmation was accepted in
`.git-exclude/reviewed/2026-07-23-rfc-mk053-closure-confirmation-review.md`, and
the project owner committed the closure candidate before M3 was marked
`Complete`.

| Closure check | Exit | Observed result |
|---|---:|---|
| `bash scripts/check-matcher-oracle-self-test.sh` | 0 | 6 checks passed |
| `bash scripts/check-matcher-oracle.sh` | 0 | Reviewed matcher package identities and features verified |
| `bash scripts/check-rfcs-self-test.sh` | 0 | 25 checks passed |
| `bash scripts/check-rfcs.sh` | 0 | `RFC integrity: 0 error(s)` |
| `git diff --check` | 0 | No tracked-file whitespace diagnostics |

The closure patch changes lifecycle, roadmap, changelog, and evidence
documentation only. Rust, dependency, lint, audit, stable, and MSRV gates were
not rerun; the independently accepted M3 implementation evidence above remains
the observed implementation record.

## M4 quality and security implementation candidate — 2026-07-23

RFC MK-054 removes the optional iced syntax highlighter while preserving the
fallback JSON editor as an editable plain-text surface, clears the complete
workspace warning backlog without lint suppression, updates the retained
Wayland build path to patched `quick-xml`, and adds one canonical local release
gate with a stubbed command-contract self-test.

### Toolchains and integrated gate

The complete `bash scripts/check-release-gates.sh` run was observed after the
implementation candidate was assembled. It used repository-local temporary
storage and exited 0 with `Release gates: all checks passed`.

| Command / probe | Observed result |
|---|---|
| `bash --version` | Bash 5.3.15 |
| `git --version` | Git 2.55.0 |
| `rustc --version`; `cargo --version` | Rust 1.97.1; Cargo 1.97.1 |
| `rustc +1.91 --version`; `cargo +1.91 --version` | Rust 1.91.1; Cargo 1.91.1 |
| `cargo fmt --version` | rustfmt 1.9.0-stable |
| `cargo clippy --version` | Clippy 0.1.97 |
| `cargo audit --version` | cargo-audit 0.22.2 |
| `cargo fmt --all -- --check` | Pass |
| `cargo test --workspace --lib --bins --locked` | Pass: app 188, model 55; app/i18n library targets contain no unit tests |
| `cargo test --workspace --doc --locked` | Pass: four model compile-fail doctests; app/i18n have none |
| `cargo build --workspace --locked` | Pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Pass with no warnings |
| `cargo +1.91 test --workspace --lib --bins --locked` | Pass: app 188, model 55 |
| `cargo +1.91 build --workspace --locked` | Pass |
| `cargo audit` | Pass: zero vulnerabilities; five allowed warnings inventoried below |
| matcher oracle self-test / checker | Pass: 6 checks; apimock-routing 5.10.0 and http 1.4.2 verified |
| RFC checker self-test / checker | Pass: 25 checks; 0 errors |
| `git diff --check` | Pass: no tracked whitespace diagnostics |

The first sandboxed integrated attempt reached `cargo audit` after the preceding
stable and MSRV gates passed, then exited 1 because the sandbox made the
advisory-database lock path read-only. The complete command was rerun outside
that restriction and passed from its first probe through its final whitespace
gate. The passing rerun, not the environmental failure, is the candidate gate
evidence.

The separately invoked
`bash scripts/check-release-gates-self-test.sh` passed 11 cases. Its independent
NUL-delimited argv log verifies the complete successful command order and exact
arguments. It also covers first-format, middle-Clippy, and final-Git failure
propagation; missing Rust 1.91, rustfmt, Clippy, cargo-audit, and Git; temporary-
directory failure; unsupported arguments; and invocation outside the repository
root. No real Rust suite runs inside those stub cases.

Focused command
`cargo test -p apimokka fallback_plain_text_editor_builds_and_preserves_draft_edits
--locked` passed one test. It builds the fallback editor with plain-text JSON,
preserves the edited draft, and keeps the draft dirty for the normal save flow.

### Dependency and audit evidence

`Cargo.lock` now contains these reviewed registry identities:

| Package | Version | Source | Checksum |
|---|---:|---|---|
| `quick-xml` | 0.41.0 | `registry+https://github.com/rust-lang/crates.io-index` | `e660451e55124f798a69a5af3f49ccfbefbd41910eefd25caf2393e1f3473ec1` |
| `wayland-scanner` | 0.31.11 | `registry+https://github.com/rust-lang/crates.io-index` | `338e30461b3a2b67d70eb30a6d89f8e0c93a833e07d2ae89085cd070c4a00ac0` |

Locked Cargo metadata reports the same two versions and crates.io sources.
Manifest/config inspection found no patch, Git dependency, alternate registry,
vendored source, or source replacement. The resolved feature graph contains one
`quick-xml` package, reached only through the `wayland-scanner` proc-macro path,
and retains iced X11 and Wayland features. `iced_highlighter`, `syntect`,
`plist`, `bincode`, and `yaml-rust` are absent from `Cargo.lock` and the active
graph.

The passing audit scanned 460 locked packages against 1,167 advisories. Its
database revision was `b54e9b51596ad6a02ca5355c1f2743cc5b5d502f`, timestamped
`2026-07-22T19:32:51+02:00`. No vulnerability or exception remains. The five
allowed warnings are recorded, but are not silently promoted beyond the M4
policy:

| Advisory | Package | Version | Category | Resolved workspace dependency path |
|---|---|---:|---|---|
| RUSTSEC-2024-0436 | `paste` | 1.0.15 | unmaintained | `paste → metal → wgpu-hal → wgpu → iced_wgpu → iced_renderer → iced → apimokka` (Apple Metal target path) |
| RUSTSEC-2026-0206 | `rustybuzz` | 0.20.1 | unmaintained | `rustybuzz → usvg → resvg → iced_tiny_skia → iced_renderer → iced → apimokka`; also `rustybuzz → usvg → resvg → iced_wgpu → iced_renderer → iced → apimokka` |
| RUSTSEC-2026-0192 | `ttf-parser` | 0.25.1 | unmaintained | Font/text: `ttf-parser → fontdb → cosmic-text → iced_graphics → iced_program → iced_winit → iced → apimokka`; window decoration: `ttf-parser → owned_ttf_parser → ab_glyph → sctk-adwaita → winit → iced_winit → iced → apimokka`; SVG: `ttf-parser → rustybuzz → usvg → resvg → iced_tiny_skia → iced_renderer → iced → apimokka` |
| RUSTSEC-2026-0190 | `anyhow` | 1.0.102 | unsound | No resolved workspace path: both host and `--target all` inverse trees are empty. It is lockfile-only unreachable residue referenced by the `wasm-metadata`, `wit-bindgen-core`, `wit-bindgen-rust`, `wit-bindgen-rust-macro`, `wit-component`, and `wit-parser` lock entries. |
| RUSTSEC-2026-0186 | `memmap2` | 0.9.10 | unsound | Font/text: `memmap2 → fontdb → cosmic-text → iced_graphics → iced_program → iced_winit → iced → apimokka`; Wayland decoration: `memmap2 → sctk-adwaita → winit → iced_winit → iced → apimokka`; clipboard: `memmap2 → smithay-client-toolkit → smithay-clipboard → clipboard_wayland → window_clipboard → iced_winit → iced → apimokka`; software rendering: `memmap2 → softbuffer → iced_tiny_skia → iced_renderer → iced → apimokka` |

Explicit no-index whitespace checks for both untracked release-gate scripts
returned the expected exit 1 with no whitespace diagnostics. The scripts are
executable. M4 is `In review`: this evidence does not claim independent
acceptance, milestone completion, R1 GO, release eligibility, or a release.
