//! Step 3 harness proof (RFC MK-055 implementation sequence).
//!
//! Deliberately narrow: its only job is to prove the fixture helper
//! produces a workspace that `apimock_config::Workspace::load` accepts, and
//! that one `apply`/`snapshot` round trip behaves as expected, before any
//! Tier 1 (mapping totality) or Tier 2 (behavioural equivalence) test is
//! built on top of it. "A harness that cannot load a fixture invalidates
//! everything built on it" — this test is that fixture's load-bearing
//! proof, not a conformance scenario in its own right.

use apimock_config::{ConfigFileKind, EditCommand, NodeKind, RulePayload, Workspace};

use crate::fixture::minimal_workspace;
use crate::to_engine;

#[test]
fn load_apply_snapshot_round_trip_on_a_real_engine_workspace() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = Workspace::load(root).expect("load fixture workspace");

    let before = workspace.snapshot();
    let rule_set_id = before
        .files
        .iter()
        .find(|file| matches!(file.kind, ConfigFileKind::RuleSet))
        .expect("fixture has a rule-set file")
        .nodes
        .iter()
        .find(|node| matches!(node.kind, NodeKind::RuleSet))
        .expect("fixture rule-set file has a RuleSet node")
        .id;
    let rules_before = before
        .files
        .iter()
        .flat_map(|file| &file.nodes)
        .filter(|node| matches!(node.kind, NodeKind::Rule))
        .count();

    // `RulePayload` is `#[non_exhaustive]` as of apimock-config 6.0.0 (RFC
    // MK-060); build from `Default::default()` and set fields directly.
    let mut rule = RulePayload::default();
    rule.url_path = Some("/api/orders".to_owned());
    rule.respond = to_engine::respond_text_only("created");
    let outcome = workspace
        .apply(EditCommand::AddRule {
            parent: rule_set_id,
            rule,
        })
        .expect("apply AddRule against the real engine");
    // Confirmed against apimock-config's own
    // `apply_add_rule_to_existing_rule_set` test: `changed_nodes` includes
    // the parent rule-set plus the new rule plus its new respond block, not
    // only the new rule. Our own `changed_nodes` semantics against this are
    // Tier 2 scenario work, not this harness proof's concern.
    assert!(
        outcome.changed_nodes.len() >= 3,
        "AddRule should report the parent, new rule, and new respond as changed; got {}",
        outcome.changed_nodes.len()
    );

    let after = workspace.snapshot();
    let rules_after = after
        .files
        .iter()
        .flat_map(|file| &file.nodes)
        .filter(|node| matches!(node.kind, NodeKind::Rule))
        .count();
    assert_eq!(
        rules_after,
        rules_before + 1,
        "snapshot after apply should show the new rule"
    );
}
