//! Friendly, developer-technical error model (RFC MK-039).
//!
//! Fallible work (save, helper start, trace connection) returns a
//! `FriendlyProblem` instead of a raw error string. The content is in the
//! developer register: it names the real cause (e.g. `EADDRINUSE`, file
//! permissions) and the concrete fix. The form is calm and actionable; the
//! content stays technically accurate.
//!
//! Lives in the model crate so it has no UI dependency — the GUI renders it,
//! the (eventual) engine integration produces it.

/// A user-facing problem with a clear cause and next step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyProblem {
    /// One short line: what happened.
    pub title: String,
    /// One or two lines: why, and what to do about it. Plain; always shown.
    pub detail: String,
    /// Technical detail (errno, raw message). Shown inline in Expert mode,
    /// collapsed behind "Show details" in Guided mode. (RFC MK-040)
    pub technical_detail: Option<String>,
    /// Optional label for a recovery action button (e.g. "Open Settings").
    pub action_label: Option<String>,
}

impl FriendlyProblem {
    pub fn new(
        title: impl Into<String>,
        detail: impl Into<String>,
        action_label: Option<&str>,
    ) -> Self {
        Self {
            title: title.into(),
            detail: detail.into(),
            technical_detail: None,
            action_label: action_label.map(|s| s.to_string()),
        }
    }

    /// Attach a technical detail line (errno, raw error text).
    pub fn with_technical(mut self, detail: impl Into<String>) -> Self {
        self.technical_detail = Some(detail.into());
        self
    }

    /// Listener port already bound by another process.
    pub fn port_in_use(port: u16) -> Self {
        Self::new(
            format!("Port {port} is already in use"),
            "Another process is using this port. Stop it, or change the \
             listener port in Settings.",
            Some("Open Settings"),
        )
        .with_technical(format!("bind {port}: address already in use (EADDRINUSE)"))
    }

    /// Could not persist a rule set / config file.
    pub fn save_failed() -> Self {
        Self::new(
            "Save failed",
            "Could not write the file. Check write permissions or choose \
             another path, then retry.",
            Some("Retry"),
        )
        .with_technical("io::ErrorKind::PermissionDenied")
    }

    /// The mock server process failed to start for a non-port reason.
    pub fn helper_failed(reason: &str) -> Self {
        Self::new(
            "The server could not start",
            "Check the listener and TLS settings, then try again.",
            Some("Open Settings"),
        )
        .with_technical(reason.to_string())
    }

    /// Trace channel dropped (socket closed / server stopped).
    pub fn trace_disconnected() -> Self {
        Self::new(
            "Trace stream disconnected",
            "The live request feed stopped (the server may have stopped). \
             Restart the server to resume it.",
            None,
        )
        .with_technical("trace socket closed: broken pipe")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_in_use_names_the_port_and_errno() {
        let p = FriendlyProblem::port_in_use(8080);
        assert!(p.title.contains("8080"));
        // The errno is preserved in the technical detail, not the plain line.
        assert!(p.technical_detail.as_deref().unwrap().contains("EADDRINUSE"));
        assert!(!p.detail.contains("EADDRINUSE"), "plain line stays plain");
        assert_eq!(p.action_label.as_deref(), Some("Open Settings"));
    }

    #[test]
    fn constructors_populate_all_fields() {
        assert!(FriendlyProblem::save_failed().action_label.is_some());
        assert!(FriendlyProblem::trace_disconnected().action_label.is_none());
        assert!(!FriendlyProblem::helper_failed("Bind error.").detail.is_empty());
    }
}
