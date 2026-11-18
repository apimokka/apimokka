//! Task 043 step 2 — the headless layout matrix (level 2 of the owner's test
//! optimisation rules): layout properties asserted on the rendered tree, with
//! no window, across **size × theme × locale × audience mode**, on every
//! `cargo test`. It replaces the step-1 spike; D-31 and D-33 are its
//! regression anchors.
//!
//! Boundary decision: single-responsibility — one table, one rule engine, one
//! self-test of that engine, for one level of the verification model; the
//! table and the engine are meant to be read together, and splitting them
//! would put the two halves of every added row in different files.
//!
//! Three rules, all structural — never exact pixels, because the simulator
//! draws bundled Fira Sans where the real window draws the system sans:
//!
//! 1. **Nothing declared is lost.** Every non-empty text and every text input
//!    has non-zero width and height (D-31's PATCH/DELETE were 0.0 wide; D-33's
//!    header value was 0 px).
//! 2. **Nothing is clipped sideways.** The tree has no horizontal scrolling,
//!    so a laid-out item must be inside the window horizontally and its
//!    visible bounds must keep its full width.
//! 3. **Vertical loss only behind a scrollable.** An item that is partly or
//!    wholly out of view (`visible_bounds` cut or `None`) must lie inside some
//!    scrollable's content bounds — "below the fold", reachable — otherwise
//!    something outside a scrollable is clipping it.
//!
//! Plus named checks on the groups this programme has lost before: the six
//! method choices, both respond modes (each one line), and the header and body
//! condition fields.
//!
//! Which bounds each check uses: `visible_bounds` for anything above the fold
//! (the stronger assertion, the one that caught D-31), layout `bounds()` plus
//! containment in a scrollable's `content_bounds` for anything below it, and
//! layout `bounds()` alone for the condition fields at 880x700 because they
//! sit below the fold there (a deliberate second choice, per the review).
//!
//! Japanese needs a CJK font on the runner (the simulator's bundled Fira Sans
//! has none); without one labels measure *narrower* and overflow checks would
//! pass spuriously, so the matrix refuses to run Japanese cells until
//! 「メソッド」 measures ≥ 55 px (any CJK font gives ~64; the Latin fallback
//! gave 42.6).

use super::*;
use crate::message::Message;
use apimokka_i18n::{Key, Locale};
use apimokka_model::AudienceMode;
use iced::{Rectangle, Size};
use iced_test::Simulator;
use iced_test::selector::Candidate;
use std::fmt::Write as _;

// ── The declarative table ─────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum Surface {
    ModePicker,
    Welcome,
    Dashboard,
    Wizard,
    RoutesEmpty,
    Editor,
    /// Guided with the "more matching criteria" section expanded.
    EditorMore,
    TestRule,
    Settings,
    SettingsMore,
    SaveDiff,
    Validation,
    Palette,
    WorkspaceMenu,
    Trace,
}

use Surface::*;

const BOTH: &[Option<AudienceMode>] = &[Some(AudienceMode::Guided), Some(AudienceMode::Expert)];
const GUIDED: &[Option<AudienceMode>] = &[Some(AudienceMode::Guided)];
const NONE: &[Option<AudienceMode>] = &[None];

/// (surface, audience modes it is built in). A new surface is one row.
const SURFACES: &[(Surface, &[Option<AudienceMode>])] = &[
    (ModePicker, NONE),
    (Welcome, GUIDED),
    (Dashboard, GUIDED),
    (Wizard, BOTH),
    (RoutesEmpty, BOTH),
    (Editor, BOTH),
    (EditorMore, GUIDED),
    (TestRule, BOTH),
    (Settings, BOTH),
    (SettingsMore, GUIDED),
    (SaveDiff, GUIDED),
    (Validation, GUIDED),
    (Palette, GUIDED),
    (WorkspaceMenu, GUIDED),
    (Trace, GUIDED),
];

/// 880x700 is decision 7's smallest mandated window; 1280x800 its baseline.
const SIZES: &[(f32, f32)] = &[(880.0, 700.0), (1280.0, 800.0)];
const THEMES: &[ThemeChoice] = &[
    ThemeChoice::Light,
    ThemeChoice::Dark,
    ThemeChoice::HighContrastLight,
    ThemeChoice::HighContrastDark,
];
const LOCALES: &[Locale] = &[Locale::En, Locale::Ja];

fn setup(surface: Surface, mode: Option<AudienceMode>, locale: Locale, theme: ThemeChoice) -> App {
    let mut a = App::new().0;
    a.update(Message::ChangeLocale(locale));
    a.update(Message::SetTheme(theme));
    let Some(mode) = mode else { return a };
    a.update(Message::ChooseAudienceMode(mode));
    match surface {
        ModePicker | Welcome => return a,
        Dashboard => {
            a.update(Message::GoDashboard);
            return a;
        }
        Wizard => {
            a.update(Message::GoWizard);
            for section in 0..3 {
                a.update(Message::WizardToggleSection(section));
            }
            return a;
        }
        _ => {}
    }
    a.update(Message::OpenWorkspace("test".into()));
    match surface {
        RoutesEmpty | SaveDiff | Validation | Palette | WorkspaceMenu | Trace | Settings
        | SettingsMore => {}
        Editor | EditorMore | TestRule => {
            let id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[2].id;
            a.update(Message::SelectRule(id));
        }
        ModePicker | Welcome | Dashboard | Wizard => unreachable!(),
    }
    match surface {
        EditorMore => a.update(Message::ToggleRuleWhenMore),
        TestRule => a.update(Message::TestRuleOpen),
        Settings => a.update(Message::SwitchTab(crate::selection::WorkspaceTab::Settings)),
        SettingsMore => {
            a.update(Message::SwitchTab(crate::selection::WorkspaceTab::Settings));
            a.update(Message::ToggleSettingsAdvancedMore);
        }
        SaveDiff => a.update(Message::OpenSaveDiffDrawer),
        Validation => a.update(Message::OpenValidationDrawer),
        Palette => a.update(Message::ToggleCommandPalette),
        WorkspaceMenu => a.update(Message::ToggleWorkspaceMenu),
        Trace => a.update(Message::SwitchTab(crate::selection::WorkspaceTab::Trace)),
        _ => {}
    }
    a
}

// ── What the harness reports ──────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Item {
    text: String,
    is_input: bool,
    bounds: Rectangle,
    visible: Option<Rectangle>,
}

#[derive(Clone, Debug, Default)]
struct Tree {
    items: Vec<Item>,
    scroll_content: Vec<Rectangle>,
}

fn inspect(app: &App, size: Size) -> Tree {
    let mut ui = Simulator::with_size(iced::Settings::default(), size, app.view());
    let mut tree = Tree::default();
    // `find` stops at the first match; a selector that never matches visits
    // every candidate, which is how the whole tree is collected.
    let _ = ui.find(|c: Candidate<'_>| -> Option<()> {
        match c {
            Candidate::Text {
                content,
                bounds,
                visible_bounds,
                ..
            } => tree.items.push(Item {
                text: content.to_owned(),
                is_input: false,
                bounds,
                visible: visible_bounds,
            }),
            Candidate::TextInput {
                state,
                bounds,
                visible_bounds,
                ..
            } => tree.items.push(Item {
                text: state.text().to_owned(),
                is_input: true,
                bounds,
                visible: visible_bounds,
            }),
            Candidate::Scrollable { content_bounds, .. } => {
                tree.scroll_content.push(content_bounds)
            }
            _ => {}
        }
        None
    });
    tree
}

// ── The rule engine (pure: data in, violations out) ───────────────────────

const EPS: f32 = 0.6;

fn contains(outer: Rectangle, inner: Rectangle) -> bool {
    outer.x - EPS <= inner.x
        && inner.x + inner.width <= outer.x + outer.width + EPS
        && outer.y - EPS <= inner.y
        && inner.y + inner.height <= outer.y + outer.height + EPS
}

fn generic_violations(tree: &Tree, size: Size) -> Vec<String> {
    let mut out = Vec::new();
    for it in &tree.items {
        if it.text.is_empty() && !it.is_input {
            continue;
        }
        let name = format!("{:?}", it.text);
        let b = it.bounds;
        if b.width <= 0.0 || b.height <= 0.0 {
            out.push(format!("{name}: zero size {}x{}", b.width, b.height));
            continue;
        }
        if b.x < -EPS || b.x + b.width > size.width + EPS {
            out.push(format!(
                "{name}: outside the window horizontally ({:.1}..{:.1} of {})",
                b.x,
                b.x + b.width,
                size.width
            ));
        }
        let behind_scroll = tree.scroll_content.iter().any(|c| contains(*c, b));
        match it.visible {
            Some(v) => {
                if v.width < b.width - EPS {
                    out.push(format!(
                        "{name}: clipped sideways (visible {:.1} of {:.1} px wide)",
                        v.width, b.width
                    ));
                } else if v.height < b.height - EPS && !behind_scroll {
                    out.push(format!("{name}: clipped vertically outside any scrollable (visible {:.1} of {:.1} px)", v.height, b.height));
                }
            }
            None if !behind_scroll => {
                out.push(format!(
                    "{name}: not visible and not inside any scrollable's content"
                ));
            }
            None => {}
        }
    }
    out
}

fn find<'t>(tree: &'t Tree, text: &str) -> Vec<&'t Item> {
    tree.items.iter().filter(|i| i.text == text).collect()
}

/// A short label must stay on one line. 27 px is 1.5 × the 18.2 px LABEL line.
const ONE_LINE_MAX_HEIGHT: f32 = 27.0;

fn named_violations(
    app: &App,
    tree: &Tree,
    surface: Surface,
    mode: Option<AudienceMode>,
) -> Vec<String> {
    let mut out = Vec::new();
    // Non-vacuity: the cell really is the surface it names. A `setup` that
    // landed on the wrong screen would otherwise pass every layout rule.
    let anchor = match surface {
        ModePicker => Key::ModePickerTitle,
        Welcome => Key::WelcomeOpenWorkspace,
        Dashboard => Key::DashTitle,
        Wizard => Key::WizardTitle,
        RoutesEmpty | Editor | EditorMore => Key::MethodCardTitle,
        TestRule => Key::TestRuleTitle,
        Settings | SettingsMore => Key::SettingsTitle,
        SaveDiff => Key::DrawerSaveDiffTitle,
        Validation => Key::DrawerValidationTitle,
        Palette => Key::PaletteTitle,
        WorkspaceMenu => Key::WorkspaceMenuCreate,
        Trace => Key::TraceTitle,
    };
    if surface != RoutesEmpty && find(tree, app.t(anchor)).is_empty() {
        out.push(format!(
            "anchor text {:?} missing — the cell is not the {surface:?} surface",
            app.t(anchor)
        ));
    }
    let need_one_line = |label: &str, what: &str, out: &mut Vec<String>| {
        let found = find(tree, label);
        if found.is_empty() {
            out.push(format!("{what} {label:?} is not drawn at all"));
        }
        for i in found {
            if i.bounds.height > ONE_LINE_MAX_HEIGHT {
                out.push(format!(
                    "{what} {label:?} wraps ({:.1} px tall, {:.1} wide)",
                    i.bounds.height, i.bounds.width
                ));
            }
        }
    };
    if matches!(surface, Editor | EditorMore) {
        // D-31/D-32/D-34: every method choice and both response modes, one line each.
        for m in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
            need_one_line(m, "method choice", &mut out);
        }
        need_one_line(app.t(Key::MethodAny), "method choice", &mut out);
        need_one_line(app.t(Key::RespondModeInline), "respond mode", &mut out);
        need_one_line(app.t(Key::RespondModeFile), "respond mode", &mut out);

        // D-33: the condition fields have real width where they are shown.
        let shown = mode == Some(AudienceMode::Expert) || surface == EditorMore;
        if shown {
            // A value a user must read and type needs ~100 px, at any width.
            let min_value = 100.0;
            for (v, min, what) in [
                ("content-type", 80.0, "header name"),
                ("application/json", min_value, "header value"),
                ("action", 80.0, "body path"),
                ("create", min_value, "body value"),
                ("items.0.qty", 80.0, "body path"),
                ("0.0", min_value, "body value"),
            ] {
                let found: Vec<_> = find(tree, v).into_iter().filter(|i| i.is_input).collect();
                match found.first() {
                    None => out.push(format!("{what} input {v:?} is not drawn")),
                    Some(i) if i.bounds.width < min => {
                        out.push(format!(
                            "{what} input {v:?} is {:.1} px wide (< {min})",
                            i.bounds.width
                        ));
                    }
                    _ => {}
                }
            }
            let removes = find(tree, "✕").len();
            if removes < 3 {
                out.push(format!("only {removes} remove buttons for 3 conditions"));
            }
            let assistants = find(tree, "…").len();
            if assistants < 2 {
                out.push(format!(
                    "only {assistants} path-assistant buttons for 2 body conditions"
                ));
            }
        }
        // The CJK precondition lives in `japanese_needs_a_cjk_font`; a cell
        // that reaches here has already passed it.
    }
    out
}

// ── The tests ─────────────────────────────────────────────────────────────

fn cjk_width() -> f32 {
    let app = setup(
        Editor,
        Some(AudienceMode::Expert),
        Locale::Ja,
        ThemeChoice::Light,
    );
    let tree = inspect(&app, Size::new(880.0, 700.0));
    find(&tree, app.t(Key::MethodCardTitle))
        .first()
        .map(|i| i.bounds.width)
        .unwrap_or(0.0)
}

#[test]
fn japanese_needs_a_cjk_font() {
    let w = cjk_width();
    assert!(
        w >= 55.0,
        "no CJK font on this runner: 「メソッド」 measures {w:.1} px (a real CJK font gives ~64, the Latin \
         fallback ~42.6). Japanese layout cells would pass spuriously — install a CJK font \
         (Noto Sans CJK) or pin one as a fixture."
    );
}

#[test]
fn layout_matrix_size_theme_locale_mode() {
    let cjk = cjk_width() >= 55.0;
    let mut failures: Vec<String> = Vec::new();
    let mut cells = 0usize;
    for &(surface, modes) in SURFACES {
        for &mode in modes {
            for &theme in THEMES {
                for &locale in LOCALES {
                    if locale == Locale::Ja && !cjk {
                        continue; // reported by `japanese_needs_a_cjk_font`
                    }
                    for &(w, h) in SIZES {
                        cells += 1;
                        let size = Size::new(w, h);
                        let app = setup(surface, mode, locale, theme);
                        let tree = inspect(&app, size);
                        let mut v = generic_violations(&tree, size);
                        v.extend(named_violations(&app, &tree, surface, mode));
                        for msg in v {
                            failures.push(format!(
                                "{surface:?} / {} / {w}x{h} / {theme:?} / {locale:?}: {msg}",
                                mode.map_or("-".into(), |m| format!("{m:?}"))
                            ));
                        }
                    }
                }
            }
        }
    }
    let mut report = String::new();
    for f in failures.iter().take(60) {
        let _ = writeln!(report, "  {f}");
    }
    assert!(
        failures.is_empty(),
        "{} layout violation(s) in {cells} cells (first 60):\n{report}",
        failures.len()
    );
}

/// The rule engine itself, on synthetic data: it must flag each failure shape
/// and pass a clean tree, or a green matrix means nothing.
#[test]
fn the_rule_engine_flags_each_failure_shape_and_passes_a_clean_tree() {
    let r = |x, y, w, h| Rectangle {
        x,
        y,
        width: w,
        height: h,
    };
    let item = |text: &str, bounds: Rectangle, visible: Option<Rectangle>| Item {
        text: text.into(),
        is_input: false,
        bounds,
        visible,
    };
    let win = Size::new(880.0, 700.0);
    let scroll = vec![r(0.0, 100.0, 880.0, 2000.0)];
    let ok = Tree {
        items: vec![
            item(
                "fine",
                r(10.0, 10.0, 50.0, 18.0),
                Some(r(10.0, 10.0, 50.0, 18.0)),
            ),
            item("below the fold", r(10.0, 900.0, 50.0, 18.0), None),
            item(
                "straddles",
                r(10.0, 690.0, 50.0, 18.0),
                Some(r(10.0, 690.0, 50.0, 10.0)),
            ),
            item("", r(0.0, 0.0, 0.0, 0.0), None),
        ],
        scroll_content: scroll.clone(),
    };
    assert_eq!(generic_violations(&ok, win), Vec::<String>::new());

    let bad = |it: Item| {
        generic_violations(
            &Tree {
                items: vec![it],
                scroll_content: scroll.clone(),
            },
            win,
        )
    };
    assert_eq!(
        bad(item("DELETE", r(10.0, 150.0, 0.0, 18.0), None)).len(),
        1,
        "zero width (D-31)"
    );
    assert_eq!(
        bad(item(
            "cut",
            r(10.0, 150.0, 50.0, 18.0),
            Some(r(10.0, 150.0, 20.0, 18.0))
        ))
        .len(),
        1,
        "clipped sideways"
    );
    let off = bad(item(
        "off",
        r(900.0, 150.0, 50.0, 18.0),
        Some(r(900.0, 150.0, 50.0, 18.0)),
    ));
    assert!(
        off.iter().any(|m| m.contains("outside the window")),
        "outside the window: {off:?}"
    );
    assert_eq!(
        bad(item("ghost", r(10.0, 50.0, 50.0, 18.0), None)).len(),
        1,
        "invisible above any scrollable (nothing but a clip can hide it)"
    );
    assert_eq!(
        bad(item(
            "half",
            r(10.0, 50.0, 50.0, 18.0),
            Some(r(10.0, 50.0, 50.0, 8.0))
        ))
        .len(),
        1,
        "clipped vertically outside a scrollable"
    );
}
