//! Match-trace presentation events used by the mockup.
//!
//! These are currently constructed from canned data. No trace transport is
//! implemented, and the reference does not establish a usable command shape
//! for the prototype trace controls; RFC MK-053 records that as a
//! `ReferenceGap` rather than an engine-mirroring claim.

/// Outcome variant + the small set of fields each variant carries.
#[derive(Debug, Clone)]
pub enum TraceOutcome {
    Matched {
        rule_set_index: usize,
        rule_index: usize,
    },
    Fallback {
        file_path: String,
        status: String,
    },
    Miss {
        status: String,
    },
    Error {
        kind: String,
        message: String,
    },
}

impl TraceOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            TraceOutcome::Matched { .. } => "matched",
            TraceOutcome::Fallback { .. } => "fallback",
            TraceOutcome::Miss { .. } => "miss",
            TraceOutcome::Error { .. } => "error",
        }
    }
    /// Non-colour glyph for the outcome, paired with the text label.
    /// External design § 31.3 non-colour status matrix.
    pub fn glyph(&self) -> &'static str {
        match self {
            TraceOutcome::Matched { .. } => "✓",
            TraceOutcome::Fallback { .. } => "↩",
            TraceOutcome::Miss { .. } => "◯",
            TraceOutcome::Error { .. } => "!",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestSummary {
    pub method: String,
    pub url_path: String,
    pub headers: Vec<(String, String)>,
    /// Optional body preview — engine may omit. The mockup sometimes
    /// carries a short JSON snippet so the match detail screen has
    /// something to render.
    pub body_preview: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MatchTraceEvent {
    pub event_id: u64,
    /// Display string in local `HH:MM:SS.mmm` form. The engine emits
    /// epoch ms; the mockup pre-formats.
    pub time: String,
    pub duration_ms: u64,
    pub request: RequestSummary,
    pub outcome: TraceOutcome,
    pub dropped_count: u32,
}
