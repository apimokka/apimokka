//! RFC MK-055 — engine-contract conformance suite.
//!
//! Drives a real `apimock_config::Workspace` against on-disk fixtures in a
//! temporary directory, to verify the MK-053 `WorkspacePort` mapping
//! against the actual crate rather than the 5.10.1 prose reference (which
//! was never published and disagrees with 5.10.0 in places — see the RFC's
//! "Observed divergences" table).
//!
//! `apimock-config` is a test-only dev-dependency of `apimokka-model` (see
//! `Cargo.toml` and `scripts/check-engine-oracle.sh` for its pinned
//! identity); this crate as a whole has no production filesystem, process,
//! or network access, and this suite adds none — every workspace it drives
//! lives in a per-test temporary directory.
//!
//! Module layout, scaled as Tier 1/Tier 2 tests are added:
//! - `fixture` — on-disk workspace builders shared across test modules.
//! - `harness` — the step-3 harness proof: one end-to-end
//!   load → apply → snapshot round trip, checked in before any scenario or
//!   totality test is built on top of it.

#[path = "engine_conformance/fixture.rs"]
mod fixture;
#[path = "engine_conformance/harness.rs"]
mod harness;
#[path = "engine_conformance/tier1_mapping.rs"]
mod tier1_mapping;
#[path = "engine_conformance/tier2_scenarios.rs"]
mod tier2_scenarios;
#[path = "engine_conformance/to_engine.rs"]
mod to_engine;
