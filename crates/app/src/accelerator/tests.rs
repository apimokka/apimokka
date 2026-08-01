//! Tests for the keyboard accelerator single source of truth.
use super::*;
use iced::keyboard::key::Named;
use std::collections::HashSet;

fn character(c: &str) -> Key {
    Key::Character(c.into())
}

// ── Test 1: table invariant — the regression guard for this defect class ──

#[test]
fn every_accelerator_has_exactly_one_advertised_table_entry() {
    let advertised: HashSet<Accelerator> = TABLE
        .iter()
        .filter(|entry| entry.label.is_some())
        .map(|entry| entry.accelerator)
        .collect();
    let expected = HashSet::from([
        Accelerator::Undo,
        Accelerator::Redo,
        Accelerator::Save,
        Accelerator::Reload,
        Accelerator::Palette,
    ]);
    assert_eq!(advertised, expected);
}

#[test]
fn every_unadvertised_entry_is_the_documented_redo_alias() {
    let unadvertised: Vec<&Entry> = TABLE.iter().filter(|entry| entry.label.is_none()).collect();
    assert_eq!(
        unadvertised.len(),
        1,
        "expected exactly one unadvertised alias entry"
    );
    assert_eq!(unadvertised[0].key, "y");
    assert_eq!(unadvertised[0].accelerator, Accelerator::Redo);
}

// ── Test 2: matching, positive and negative per command ───────────────────

#[test]
fn undo_matches_primary_modifier_without_shift() {
    assert!(matches!(
        match_key(&character("z"), Modifiers::CTRL, Platform::Other),
        Some(Message::Undo)
    ));
}

#[test]
fn shift_distinguishes_undo_from_redo() {
    assert!(matches!(
        match_key(&character("z"), Modifiers::CTRL, Platform::Other),
        Some(Message::Undo)
    ));
    assert!(matches!(
        match_key(
            &character("z"),
            Modifiers::CTRL | Modifiers::SHIFT,
            Platform::Other
        ),
        Some(Message::Redo)
    ));
}

#[test]
fn bare_key_without_modifier_does_not_match() {
    assert!(match_key(&character("z"), Modifiers::NONE, Platform::Other).is_none());
    assert!(match_key(&character("s"), Modifiers::NONE, Platform::Other).is_none());
    assert!(match_key(&character("r"), Modifiers::NONE, Platform::Other).is_none());
    assert!(match_key(&character("k"), Modifiers::NONE, Platform::Other).is_none());
    assert!(match_key(&character("y"), Modifiers::NONE, Platform::Other).is_none());
}

#[test]
fn save_matches_primary_modifier() {
    assert!(matches!(
        match_key(&character("s"), Modifiers::CTRL, Platform::Other),
        Some(Message::Save)
    ));
}

#[test]
fn reload_matches_primary_modifier() {
    assert!(matches!(
        match_key(&character("r"), Modifiers::CTRL, Platform::Other),
        Some(Message::ReloadConfig)
    ));
}

#[test]
fn palette_matches_primary_modifier() {
    assert!(matches!(
        match_key(&character("k"), Modifiers::CTRL, Platform::Other),
        Some(Message::ToggleCommandPalette)
    ));
}

#[test]
fn ctrl_y_matches_redo_alias_on_non_macos() {
    assert!(matches!(
        match_key(&character("y"), Modifiers::CTRL, Platform::Other),
        Some(Message::Redo)
    ));
}

#[test]
fn primary_modifier_on_macos_is_logo_not_control() {
    assert!(matches!(
        match_key(&character("z"), Modifiers::LOGO, Platform::MacOs),
        Some(Message::Undo)
    ));
    assert!(matches!(
        match_key(
            &character("z"),
            Modifiers::LOGO | Modifiers::SHIFT,
            Platform::MacOs
        ),
        Some(Message::Redo)
    ));
    assert!(matches!(
        match_key(&character("s"), Modifiers::LOGO, Platform::MacOs),
        Some(Message::Save)
    ));
    assert!(matches!(
        match_key(&character("r"), Modifiers::LOGO, Platform::MacOs),
        Some(Message::ReloadConfig)
    ));
    assert!(matches!(
        match_key(&character("k"), Modifiers::LOGO, Platform::MacOs),
        Some(Message::ToggleCommandPalette)
    ));
}

#[test]
fn named_keys_never_match_an_accelerator() {
    assert!(match_key(&Key::Named(Named::Escape), Modifiers::CTRL, Platform::Other).is_none());
}

// ── Test 4: macOS Ctrl behaviour ───────────────────────────────────────────

#[test]
fn ctrl_z_does_not_match_undo_on_macos() {
    assert!(match_key(&character("z"), Modifiers::CTRL, Platform::MacOs).is_none());
}

#[test]
fn ctrl_y_does_not_match_redo_on_macos() {
    assert!(match_key(&character("y"), Modifiers::CTRL, Platform::MacOs).is_none());
}

// ── Test 3: notation, both platform branches, all five commands ───────────

#[test]
fn notation_renders_platform_specific_strings_for_every_accelerator() {
    assert_eq!(notation(Accelerator::Undo, Platform::MacOs), "⌘Z");
    assert_eq!(notation(Accelerator::Undo, Platform::Other), "Ctrl+Z");
    assert_eq!(notation(Accelerator::Redo, Platform::MacOs), "⌘⇧Z");
    assert_eq!(notation(Accelerator::Redo, Platform::Other), "Ctrl+Shift+Z");
    assert_eq!(notation(Accelerator::Save, Platform::MacOs), "⌘S");
    assert_eq!(notation(Accelerator::Save, Platform::Other), "Ctrl+S");
    assert_eq!(notation(Accelerator::Reload, Platform::MacOs), "⌘R");
    assert_eq!(notation(Accelerator::Reload, Platform::Other), "Ctrl+R");
    assert_eq!(notation(Accelerator::Palette, Platform::MacOs), "⌘K");
    assert_eq!(notation(Accelerator::Palette, Platform::Other), "Ctrl+K");
}
