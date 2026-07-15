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
