// Proof of concept — does the design API compile as documented?
fn _poc() {
    use snora::design::{Tokens, Color};
    let t = Tokens::light();
    let _ = Tokens::dark();
    let _ = Tokens::high_contrast_light();
    let _ = Tokens::high_contrast_dark();
    let _gap = t.spacing.md;
    let _accent = t.palette.accent;
    let _text = t.palette.text_primary;
    let _muted = t.palette.text_muted;
    // Color construction
    let _c = Color::rgb(0.0, 0.5, 0.4);
    // Style bridge
    let _ic = snora::design::style::color::to_iced_color(t.palette.accent);
}
