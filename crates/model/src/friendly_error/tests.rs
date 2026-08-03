use super::*;

#[test]
fn port_in_use_names_the_port_and_errno() {
    let p = FriendlyProblem::port_in_use(8080);
    assert!(p.title.contains("8080"));
    // The errno is preserved in the technical detail, not the plain line.
    assert!(
        p.technical_detail
            .as_deref()
            .unwrap()
            .contains("EADDRINUSE")
    );
    assert!(!p.detail.contains("EADDRINUSE"), "plain line stays plain");
    assert_eq!(p.action_label.as_deref(), Some("Open Settings"));
}

#[test]
fn constructors_populate_all_fields() {
    assert!(FriendlyProblem::save_failed().action_label.is_some());
    assert!(FriendlyProblem::trace_disconnected().action_label.is_none());
    assert!(
        !FriendlyProblem::helper_failed("Bind error.")
            .detail
            .is_empty()
    );
}
