//! apimokka-i18n — MK-036 translation tables.

mod en;
mod ja;
pub mod keys;

pub use keys::Key;

/// Application locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    #[default]
    En,
    Ja,
}

impl Locale {
    pub fn all() -> &'static [Locale] {
        &[Locale::En, Locale::Ja]
    }
    pub fn t(self, key: Key) -> &'static str {
        match self {
            Locale::En => en::t(key),
            Locale::Ja => ja::t(key),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Locale::En => "EN",
            Locale::Ja => "JA",
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}
