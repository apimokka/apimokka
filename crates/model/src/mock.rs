//! Canned mock data for the v1 mockup.
//!
//! Split by fixture domain: workspace snapshots (RFC MK-047, MK-048,
//! MK-053), trace events, and recent-workspace dashboard cards.

mod recent;
mod trace_fixtures;
mod workspace_fixtures;

pub use recent::{RecentWorkspace, recent_workspaces};
pub use trace_fixtures::sample_trace_events;
pub use workspace_fixtures::{
    blank_workspace, minimal_workspace, shop_api_canonical_seed, shop_api_mock,
};
