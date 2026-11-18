//! Task 043 step 1 — the headless-UI spike. Not the matrix: three questions
//! (does iced_test see clipping, is it deterministic, what does it cost).
//!
//! The first test is red on `59036cb` (D-31: PATCH/DELETE zero-width and not
//! visible) and on `c046485`..`4240803` (D-33: header value width 0), green
//! on `6958496` and later. Run it with `ICED_TEST_BACKEND=tiny-skia` for the
//! software renderer (no GPU adapter probe; 0.07 s vs 0.5 s here).

use super::*;
use crate::message::Message;
use iced_test::Simulator;

fn rule3_in_expert(theme: ThemeChoice, ja: bool) -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    if ja {
        a.update(Message::ChangeLocale(apimokka_i18n::Locale::Ja));
    }
    a.update(Message::SetTheme(theme));
    let id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[2].id;
    a.update(Message::SelectRule(id));
    a
}

/// (layout bounds, visible bounds) of the text target, or the find error.
fn bounds_of(
    ui: &mut Simulator<'_, Message>,
    text: &str,
) -> Result<(iced::Rectangle, Option<iced::Rectangle>), String> {
    match ui.find(text.to_owned()) {
        Ok(t) => Ok((t.bounds(), t.visible_bounds())),
        Err(e) => Err(format!("{e}")),
    }
}

/// D-31/D-33's own geometry, at the mandated 880x700.
#[test]
fn spike_method_choices_and_header_value_have_visible_bounds_at_880x700() {
    let app = rule3_in_expert(ThemeChoice::Light, false);
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(880.0, 700.0),
        app.view(),
    );
    // D-31: every method choice is on screen (visible bounds, not just laid out).
    let mut missing = Vec::new();
    for m in ["Any", "GET", "POST", "PUT", "PATCH", "DELETE"] {
        match bounds_of(&mut ui, m) {
            Ok((_, Some(v))) if v.width > 0.0 && v.height > 0.0 => {}
            other => missing.push(format!("{m}: {other:?}")),
        }
    }
    // D-33: the header value input has real width (its *layout* bounds: it sits
    // below the 700 px fold, so visible bounds is None by design — see report).
    let hv = bounds_of(&mut ui, "application/json");
    let hv_w = hv.as_ref().map(|(b, _)| b.width).unwrap_or(0.0);
    eprintln!("SPIKE methods-missing={missing:?} header-value-width={hv_w}");
    assert!(
        missing.is_empty(),
        "D-31: method choices not visible: {missing:?}"
    );
    assert!(hv_w >= 100.0, "D-33: header value width {hv_w} px (<100)");
}

/// Spike question 2/3 probe: every number the matrix would assert on, printed.
/// `cargo test -p apimokka spike_probe -- --ignored --nocapture`
#[test]
#[ignore = "spike probe: prints layout numbers, asserts nothing"]
fn spike_probe_layout_numbers() {
    for (theme, ja, w, h) in [
        (ThemeChoice::Light, false, 880.0, 700.0),
        (ThemeChoice::Light, true, 880.0, 700.0),
        (ThemeChoice::HighContrastDark, false, 1280.0, 800.0),
    ] {
        let app = rule3_in_expert(theme, ja);
        let mut ui =
            Simulator::with_size(iced::Settings::default(), iced::Size::new(w, h), app.view());
        let labels: &[&str] = if ja {
            &[
                "GET",
                "DELETE",
                "メソッド",
                "ファイル提供",
                "インラインテキスト",
            ]
        } else {
            &["GET", "DELETE", "Method", "Serve file", "Inline text"]
        };
        let mut row = Vec::new();
        for l in labels {
            row.push(match bounds_of(&mut ui, l) {
                Ok((b, v)) => format!(
                    "{l}=[{:.3},{:.3},{:.3},{:.3}] vis={}",
                    b.x,
                    b.y,
                    b.width,
                    b.height,
                    v.is_some()
                ),
                Err(e) => format!("{l}=ERR({e})"),
            });
        }
        eprintln!("PROBE {theme:?} ja={ja} {w}x{h}: {}", row.join(" | "));
    }
}
