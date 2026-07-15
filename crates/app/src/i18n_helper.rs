//! Thin shim around `apimokka_i18n` so the rest of the app calls a
//! one-arg helper bound to the app's currently-selected locale.
//!
//! Centralising the locale here means individual screen modules need
//! never look up the locale on the app state — they ask the shell for a
//! `Tr` and call it with a key.

use apimokka_i18n::{Key, Locale, t};

/// A locale-bound translator. Cheap to copy; one is held by the shell
/// view function and passed by value into each screen.
#[derive(Debug, Clone, Copy)]
pub struct Tr {
    pub locale: Locale,
}

impl Tr {
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }
    pub fn t(&self, key: Key) -> &'static str {
        t(self.locale, key)
    }
}
