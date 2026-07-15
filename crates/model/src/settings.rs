//! Root-level workspace settings.
//!
//! Mirrors `RootSettingKey` + the relevant strategy variants from the
//! engine. The mockup keeps them in one struct because each Settings
//! sub-panel reads and writes only one named field at a time.

#[derive(Debug, Clone)]
pub struct RootSettings {
    // Listener (Restart impact)
    pub listener_ip: String,
    pub listener_port: u16,

    // TLS (Restart impact)
    pub tls_enabled: bool,
    pub tls_cert_file: String,
    pub tls_key_file: String,

    // Logging (level: Reload; file: Restart; format: Reload)
    pub log_level: String,
    pub log_file: String,
    pub log_format: String,

    // Service (Reload impact)
    pub fallback_respond_dir: String,
    pub strategy: Strategy,

    // File-tree filters (Reload impact)
    pub file_tree_show_hidden: bool,
    pub file_tree_builtin_excludes: bool,
    pub file_tree_extra_excludes: Vec<String>,
    pub file_tree_include: Vec<String>,

    // Trace (Reload impact)
    pub trace_enabled: bool,
    pub trace_transport: TraceTransport,
    pub trace_uds_path: String,
    pub trace_tcp_addr: String,
    pub trace_queue_size: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceTransport {
    Uds,
    Tcp,
    Disabled,
}

impl TraceTransport {
    #[allow(dead_code)]
    pub fn all() -> [TraceTransport; 3] {
        [
            TraceTransport::Uds,
            TraceTransport::Tcp,
            TraceTransport::Disabled,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            TraceTransport::Uds => "uds",
            TraceTransport::Tcp => "tcp",
            TraceTransport::Disabled => "disabled",
        }
    }
}

impl std::fmt::Display for TraceTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    FirstMatch,
    UniformRandom,
    WeightedRandom,
    Priority,
    RoundRobin,
}

impl Strategy {
    pub fn all() -> [Strategy; 5] {
        [
            Strategy::FirstMatch,
            Strategy::UniformRandom,
            Strategy::WeightedRandom,
            Strategy::Priority,
            Strategy::RoundRobin,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            Strategy::FirstMatch => "FirstMatch",
            Strategy::UniformRandom => "UniformRandom",
            Strategy::WeightedRandom => "WeightedRandom",
            Strategy::Priority => "Priority",
            Strategy::RoundRobin => "RoundRobin",
        }
    }
    /// One-line description shown next to the strategy dropdown.
    /// Source: external design § 20.5 strategy help copy.
    pub fn help(self) -> &'static str {
        match self {
            Strategy::FirstMatch => "The first matching rule in list order wins.",
            Strategy::UniformRandom => "A matching rule is selected randomly.",
            Strategy::WeightedRandom => {
                "Matching rules are selected randomly using per-rule weights."
            }
            Strategy::Priority => "The highest priority matching rule wins.",
            Strategy::RoundRobin => "Matching rules are selected in rotation.",
        }
    }
    /// Whether per-rule controls (weight / priority) should be visible.
    pub fn needs_per_rule_field(self) -> bool {
        matches!(self, Strategy::WeightedRandom | Strategy::Priority)
    }
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl Default for RootSettings {
    fn default() -> Self {
        Self {
            listener_ip: "127.0.0.1".into(),
            listener_port: 3000,
            tls_enabled: false,
            tls_cert_file: String::new(),
            tls_key_file: String::new(),
            log_level: "info".into(),
            log_file: String::new(),
            log_format: "plain".into(),
            fallback_respond_dir: "responses".into(),
            strategy: Strategy::FirstMatch,
            file_tree_show_hidden: false,
            file_tree_builtin_excludes: true,
            file_tree_extra_excludes: vec![],
            file_tree_include: vec![".json".into(), ".toml".into()],
            trace_enabled: true,
            trace_transport: TraceTransport::Uds,
            trace_uds_path: "/tmp/apimock-trace.sock".into(),
            trace_tcp_addr: "127.0.0.1:0".into(),
            trace_queue_size: 1024,
        }
    }
}
