/// D-14: severity glyphs have one source, `apimokka_model::Severity::glyph`.
/// These fail if a second table is reintroduced in the widgets module or the
/// validation drawer calls one.
#[test]
fn no_second_severity_glyph_source() {
    let widgets = include_str!("mod.rs");
    let drawer = include_str!("../shell/bottom_drawer.rs");
    assert!(
        !widgets.contains("fn severity_glyph"),
        "widgets::severity_glyph returned; severity glyphs must come from Severity::glyph()"
    );
    assert!(
        !drawer.contains("severity_glyph("),
        "bottom_drawer calls a severity_glyph helper; use Severity::glyph()"
    );
}
