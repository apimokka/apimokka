//! Audience presentation mode (RFC MK-040).
//!
//! A user-chosen preference for how much explanatory scaffolding the UI shows
//! by default. It never changes vocabulary — only density of explanation.
//! Lives in the model crate so it has no UI dependency.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudienceMode {
    /// Extra explanations expanded inline; best for newcomers to HTTP mocking.
    Guided,
    /// Compact; explanations available on demand. Best for experienced users.
    Expert,
}

impl AudienceMode {
    pub fn all() -> [AudienceMode; 2] {
        [AudienceMode::Guided, AudienceMode::Expert]
    }

    /// True when explanatory scaffolding should be expanded inline by default.
    pub fn shows_scaffolding(self) -> bool {
        matches!(self, AudienceMode::Guided)
    }
}

#[cfg(test)]
#[path = "audience/tests.rs"]
mod tests;
