use super::Severity;

/// D-14: the documented glyph API must return the values the app renders
/// (`✕` / `⚠` / `ℹ`). The previous values (`!` / `△` / `i`) were dead code.
#[test]
fn severity_glyphs_are_the_rendered_values() {
    assert_eq!(Severity::Error.glyph(), "✕");
    assert_eq!(Severity::Warning.glyph(), "⚠");
    assert_eq!(Severity::Info.glyph(), "ℹ");
}
