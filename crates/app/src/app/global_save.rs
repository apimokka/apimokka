use apimokka_model::{FileDiff, RuntimeEffect, SaveError, WorkspaceRelativePath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalSaveCompletion {
    Complete,
    Partial,
    Failed,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSaveReport {
    pub workspace: WorkspaceSaveReport,
    pub fallback: FallbackSaveReport,
}

impl GlobalSaveReport {
    pub fn completion(&self) -> GlobalSaveCompletion {
        if matches!(
            self.workspace.integrity,
            SaveIntegrity::ContractFault {
                progress_trust: ProgressTrust::Unverified,
                ..
            }
        ) {
            return GlobalSaveCompletion::Indeterminate;
        }

        let workspace_written = self.workspace.progress.written_files().len();
        match (&self.workspace.progress, &self.fallback) {
            (WorkspaceSaveProgress::Saved { .. }, FallbackSaveReport::Completed { .. })
                if self.workspace.integrity == SaveIntegrity::Valid =>
            {
                GlobalSaveCompletion::Complete
            }
            (_, FallbackSaveReport::Failed { written_keys, .. })
                if workspace_written + written_keys.len() > 0 =>
            {
                GlobalSaveCompletion::Partial
            }
            (WorkspaceSaveProgress::Failed { .. }, _)
            | (_, FallbackSaveReport::NotEntered { .. })
                if workspace_written > 0 =>
            {
                GlobalSaveCompletion::Partial
            }
            _ => GlobalSaveCompletion::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSaveReport {
    pub progress: WorkspaceSaveProgress,
    pub integrity: SaveIntegrity,
    pub unsaved_hint: RuntimeEffect,
    pub runtime_pending: RuntimeEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSaveProgress {
    Saved {
        written_files: Vec<WorkspaceRelativePath>,
        diffs: Vec<FileDiff>,
    },
    Failed {
        written_files: Vec<WorkspaceRelativePath>,
        diffs: Vec<FileDiff>,
        failed_file: WorkspaceRelativePath,
        cause: SaveError,
    },
}

impl WorkspaceSaveProgress {
    pub fn written_files(&self) -> &[WorkspaceRelativePath] {
        match self {
            Self::Saved { written_files, .. } | Self::Failed { written_files, .. } => written_files,
        }
    }

    pub fn diffs(&self) -> &[FileDiff] {
        match self {
            Self::Saved { diffs, .. } | Self::Failed { diffs, .. } => diffs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveIntegrity {
    Valid,
    ContractFault {
        reason: String,
        progress_trust: ProgressTrust,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressTrust {
    Verified,
    Unverified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackSaveReport {
    NotEntered {
        reason: FallbackSkipReason,
        remaining_keys: Vec<String>,
    },
    Completed {
        written_keys: Vec<String>,
    },
    Failed {
        written_keys: Vec<String>,
        failure: FallbackSaveFailure,
        remaining_keys: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackSkipReason {
    WorkspaceFailed,
    WorkspaceContractFault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackSaveFailure {
    pub key: String,
    pub cause: FallbackSaveError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackSaveError {
    detail: String,
}

impl FallbackSaveError {
    #[cfg(test)]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
