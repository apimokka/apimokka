use super::*;

#[test]
fn guided_shows_scaffolding_expert_does_not() {
    assert!(AudienceMode::Guided.shows_scaffolding());
    assert!(!AudienceMode::Expert.shows_scaffolding());
}
