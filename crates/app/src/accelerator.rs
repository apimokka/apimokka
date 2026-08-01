//! Keyboard accelerator single source of truth (dev-team handoff 002).
//!
//! One table maps every accelerator-bearing command to its key, modifier
//! requirement, dispatched [`Message`], and platform-specific display label.
//! The app subscription obtains its matcher from this table; the command
//! palette obtains its display string from this table. Neither site
//! hard-codes a key or a label, so the two can no longer drift
//! independently.
//!
//! Accelerator notation (`⌘Z`, `Ctrl+Z`, …) is platform key-cap notation,
//! not prose, so it is exempt from i18n translation; Japanese interfaces
//! display the same `Ctrl`/`⌘` strings as English ones.

use crate::message::Message;
use iced::keyboard::{Key, Modifiers};

#[cfg(test)]
#[path = "accelerator/tests.rs"]
mod tests;

/// A command that carries a keyboard accelerator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Accelerator {
    Undo,
    Redo,
    Save,
    Reload,
    Palette,
}

/// The target platform for accelerator notation and primary-modifier
/// resolution. An explicit parameter (rather than a `cfg`-gated branch)
/// keeps both branches unit-testable from any development host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Platform {
    MacOs,
    Other,
}

impl Platform {
    pub(crate) fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Other
        }
    }
}

/// Whether the platform's primary accelerator modifier (⌘ on macOS, Ctrl
/// elsewhere) is held. Mirrors `iced::keyboard::Modifiers::command()`,
/// parameterized by [`Platform`] instead of `cfg!` so both branches are
/// reachable from tests on any host.
fn primary_modifier(modifiers: Modifiers, platform: Platform) -> bool {
    match platform {
        Platform::MacOs => modifiers.logo(),
        Platform::Other => modifiers.control(),
    }
}

/// One accelerator binding: which key and modifiers trigger it, the
/// [`Message`] it dispatches, and its display label. `label` is `None` for
/// an unadvertised alias — `Ctrl+Y` (D8, D3) is a redo alias, not a second
/// binding to promote in the palette.
struct Entry {
    accelerator: Accelerator,
    key: &'static str,
    /// `None` means Shift state is not part of the match (the `Ctrl+Y`
    /// alias below).
    shift: Option<bool>,
    modifier: fn(Modifiers, Platform) -> bool,
    message: fn() -> Message,
    /// `(macos, other)` display notation, present only for the advertised
    /// primary binding of a command.
    label: Option<(&'static str, &'static str)>,
}

const TABLE: &[Entry] = &[
    Entry {
        accelerator: Accelerator::Undo,
        key: "z",
        shift: Some(false),
        modifier: primary_modifier,
        message: || Message::Undo,
        label: Some(("⌘Z", "Ctrl+Z")),
    },
    Entry {
        accelerator: Accelerator::Redo,
        key: "z",
        shift: Some(true),
        modifier: primary_modifier,
        message: || Message::Redo,
        label: Some(("⌘⇧Z", "Ctrl+Shift+Z")),
    },
    // Non-macOS `Ctrl+Y` redo alias (D3). Gated on `Platform::Other` rather
    // than the primary-modifier check alone, which would otherwise also
    // accept it on macOS. Unadvertised (D8): listing two bindings for one
    // command conflicts with "less is more".
    Entry {
        accelerator: Accelerator::Redo,
        key: "y",
        shift: None,
        modifier: |modifiers, platform| platform == Platform::Other && modifiers.control(),
        message: || Message::Redo,
        label: None,
    },
    Entry {
        accelerator: Accelerator::Save,
        key: "s",
        shift: Some(false),
        modifier: primary_modifier,
        message: || Message::Save,
        label: Some(("⌘S", "Ctrl+S")),
    },
    Entry {
        accelerator: Accelerator::Reload,
        key: "r",
        shift: Some(false),
        modifier: primary_modifier,
        message: || Message::ReloadConfig,
        label: Some(("⌘R", "Ctrl+R")),
    },
    Entry {
        accelerator: Accelerator::Palette,
        key: "k",
        shift: Some(false),
        modifier: primary_modifier,
        message: || Message::ToggleCommandPalette,
        label: Some(("⌘K", "Ctrl+K")),
    },
];

/// Match a pressed key against the table for an explicit [`Platform`]. Used
/// directly by tests; [`match_pressed`] is the runtime entry point.
pub(crate) fn match_key(key: &Key, modifiers: Modifiers, platform: Platform) -> Option<Message> {
    let Key::Character(pressed) = key else {
        return None;
    };
    let pressed = pressed.as_str();
    TABLE.iter().find_map(|entry| {
        if entry.key != pressed {
            return None;
        }
        if let Some(shift) = entry.shift
            && shift != modifiers.shift()
        {
            return None;
        }
        if !(entry.modifier)(modifiers, platform) {
            return None;
        }
        Some((entry.message)())
    })
}

/// Match a pressed key against the table for the host's actual platform.
pub(crate) fn match_pressed(key: &Key, modifiers: Modifiers) -> Option<Message> {
    match_key(key, modifiers, Platform::current())
}

/// Render an accelerator's display notation for an explicit [`Platform`],
/// reading the label carried by its advertised table entry. Used directly
/// by tests; [`display`] is the runtime entry point.
///
/// Panics if `accelerator` has no advertised table entry: that would be a
/// programming error in this module, not a runtime condition callers need
/// to recover from.
pub(crate) fn notation(accelerator: Accelerator, platform: Platform) -> &'static str {
    TABLE
        .iter()
        .find_map(|entry| {
            if entry.accelerator != accelerator {
                return None;
            }
            let (macos, other) = entry.label?;
            Some(match platform {
                Platform::MacOs => macos,
                Platform::Other => other,
            })
        })
        .unwrap_or_else(|| panic!("no advertised accelerator table entry for {accelerator:?}"))
}

/// Render an accelerator's display notation for the host's actual platform.
pub(crate) fn display(accelerator: Accelerator) -> &'static str {
    notation(accelerator, Platform::current())
}
