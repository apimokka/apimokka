//! Response definition for one rule.
//!
//! Exactly one of `text` or `file_path` should be set per rule
//! (RFC validation; external design § 17.6). The GUI enforces this via
//! a tabbed editor.

/// Response mode the GUI is currently editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespondMode {
    InlineText,
    ServeFile,
}

impl RespondMode {
    pub fn label(self) -> &'static str {
        match self {
            RespondMode::InlineText => "Inline text",
            RespondMode::ServeFile => "Serve file",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RespondPayload {
    pub mode: RespondMode,
    pub text: String,
    pub file_path: String,
    /// Default `"200 OK"`. Combobox in the editor.
    pub status: String,
    pub delay_milliseconds: u64,
}

impl Default for RespondPayload {
    fn default() -> Self {
        Self {
            mode: RespondMode::InlineText,
            text: String::new(),
            file_path: String::new(),
            status: "200 OK".into(),
            delay_milliseconds: 0,
        }
    }
}

impl RespondPayload {
    /// Compact one-line summary used in rule rows and detail panels.
    /// Example: `200 OK · inline JSON · 120ms`.
    pub fn summary(&self) -> String {
        let kind = match self.mode {
            RespondMode::InlineText => "inline",
            RespondMode::ServeFile => "file",
        };
        if self.delay_milliseconds > 0 {
            format!("{} · {} · {}ms", self.status, kind, self.delay_milliseconds)
        } else {
            format!("{} · {}", self.status, kind)
        }
    }
}
