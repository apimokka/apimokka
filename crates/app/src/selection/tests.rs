use super::*;

#[test]
fn reconciliation_uses_only_the_captured_target_and_parent() {
    let original = apimokka_model::mock::shop_api_canonical_seed();
    let selected_rule = original.rule_sets[0].rules[0].id;
    let parent = original.rule_sets[0].id;
    let unrelated = original.rule_sets[1].id;
    let fallback = original.fallback_files[0].path.clone();

    let mut selection = RouteSelection {
        rule_set: Some(unrelated),
        rule: Some(selected_rule),
        file_route: Some(fallback),
        script: Some("contradictory.rhai".into()),
    };
    let target = selection.capture(&original);
    selection.reconcile(&original, target);
    assert_eq!(
        selection,
        RouteSelection {
            rule_set: Some(parent),
            rule: Some(selected_rule),
            file_route: None,
            script: None,
        }
    );

    let target = selection.capture(&original);
    let mut after = original.clone();
    after.rule_sets[0].rules.remove(0);
    selection.reconcile(&after, target);
    assert_eq!(
        selection,
        RouteSelection {
            rule_set: Some(parent),
            ..RouteSelection::default()
        }
    );
}

#[test]
fn unavailable_or_contradictory_targets_never_select_an_unrelated_first_node() {
    let original = apimokka_model::mock::shop_api_canonical_seed();
    let removed_set = original.rule_sets[0].id;
    let mut selection = RouteSelection {
        rule_set: Some(removed_set),
        ..RouteSelection::default()
    };
    let target = selection.capture(&original);
    let mut after = original.clone();
    after.rule_sets.remove(0);
    selection.reconcile(&after, target);
    assert_eq!(selection, RouteSelection::default());

    selection.rule = Some(NodeId::new());
    selection.file_route = original
        .fallback_files
        .first()
        .map(|file| file.path.clone());
    let target = selection.capture(&original);
    selection.reconcile(&original, target);
    assert_eq!(selection, RouteSelection::default());
}

#[test]
fn exact_file_and_script_targets_survive_only_while_present() {
    let original = apimokka_model::mock::shop_api_canonical_seed();
    let fallback = original.fallback_files[0].path.clone();
    let script = original.middleware_scripts[0].path.clone();
    let mut selection = RouteSelection::default();

    selection.select_fallback(fallback.clone());
    let target = selection.capture(&original);
    selection.reconcile(&original, target);
    assert_eq!(selection.file_route.as_deref(), Some(fallback.as_str()));

    selection.select_script(script.clone());
    let target = selection.capture(&original);
    let mut without_script = original.clone();
    without_script.middleware_scripts.clear();
    selection.reconcile(&without_script, target);
    assert_eq!(selection, RouteSelection::default());
}
