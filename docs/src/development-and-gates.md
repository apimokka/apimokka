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
