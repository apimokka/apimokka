use super::*;

#[test]
fn mock_workspace_is_well_formed() {
    let snap = shop_api_mock();
    // Every workspace should have at least one rule set, each with rules.
    assert!(!snap.rule_sets.is_empty(), "mock must have rule sets");
    for rs in &snap.rule_sets {
        assert!(
            !rs.rules.is_empty(),
            "rule set {:?} has no rules",
            rs.file.path
        );
        // Rule summaries are user-facing labels — never empty.
        for r in &rs.rules {
            assert!(
                !r.summary().trim().is_empty(),
                "rule summary must be non-empty"
            );
        }
    }
}

#[test]
fn mock_has_fallback_files_with_routes() {
    let snap = shop_api_mock();
    assert!(
        !snap.fallback_files.is_empty(),
        "mock must have fallback files"
    );
    // Each fallback file should advertise the route it serves.
    assert!(
        snap.fallback_files.iter().any(|f| f.route_hint.is_some()),
        "at least one fallback file should have a route hint"
    );
}

#[test]
fn blank_workspace_uses_wizard_inputs() {
    let ws = blank_workspace("payments-mock", "0.0.0.0", 9090, true);
    assert_eq!(ws.meta.name, "payments-mock");
    assert_eq!(ws.root_settings.listener_ip, "0.0.0.0");
    assert_eq!(ws.root_settings.listener_port, 9090);
    assert!(ws.root_settings.tls_enabled);
    assert!(
        ws.rule_sets.is_empty(),
        "blank workspace starts with no rules"
    );
    assert!(ws.fallback_files.is_empty());
    assert!(ws.diagnostics.is_empty());
}
