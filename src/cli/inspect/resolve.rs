//! Trace loading and persisted workspace artifact resolution for inspect.

use std::path::{Path, PathBuf};

use crate::adapters::audit_store::{
    FileSessionAuditStore, FrameworkAdapterHookAuditStore, SessionAuditStore,
};
use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::domain::audit::SessionAuditProjection;
use crate::domain::session::{
    session_goal_brief_ref, session_plan_brief_ref, session_run_brief_ref,
};
use crate::domain::trace::{ExecutionTrace, HookEventDispatchRecord};

use super::{InspectCommandError, TraceResolutionTarget, TraceSummaryError, resolve_trace_path};

pub(super) struct ResolvedTraceArtifacts {
    pub(super) goal_brief_ref: Option<String>,
    pub(super) session_plan_brief_ref: Option<String>,
    pub(super) run_brief_ref: Option<String>,
    pub(super) session_audit: Option<SessionAuditProjection>,
    pub(super) latest_framework_adapter_hook_dispatch: Option<HookEventDispatchRecord>,
}

// Load the requested trace using explicit trace path first, then active-session
// trace ref, then the latest workspace trace.
pub(super) fn load_trace(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    session_id: Option<&str>,
) -> Result<(TraceResolutionTarget, PathBuf, ExecutionTrace), InspectCommandError> {
    let session_trace_ref = workspace
        .map(|workspace_path| resolve_session_trace_ref(workspace_path, session_id))
        .transpose()?
        .flatten();
    let (target, trace_path) = resolve_trace_path(trace, workspace, session_trace_ref.as_deref())?;

    let resolved_trace_path = match target {
        TraceResolutionTarget::SessionTraceRef => {
            if let Some(workspace_path) = workspace.filter(|_| trace_path.is_relative()) {
                workspace_path.join(&trace_path)
            } else {
                trace_path.clone()
            }
        }
        TraceResolutionTarget::ExplicitTrace | TraceResolutionTarget::LatestWorkspaceTrace => {
            trace_path.clone()
        }
    };

    let trace = match target {
        TraceResolutionTarget::LatestWorkspaceTrace => {
            let Some(workspace_path) = workspace else {
                return Err(InspectCommandError::MissingTraceReference);
            };
            let store = FileTraceStore::for_workspace(workspace_path);
            store.load(&resolved_trace_path)?
        }
        TraceResolutionTarget::ExplicitTrace | TraceResolutionTarget::SessionTraceRef => {
            let store =
                FileTraceStore::new(resolved_trace_path.parent().unwrap_or_else(|| Path::new(".")));
            store.load(&resolved_trace_path)?
        }
    };

    Ok((target, resolved_trace_path, trace))
}

pub(super) fn resolve_session_trace_ref(
    workspace: &Path,
    session_id: Option<&str>,
) -> Result<Option<String>, InspectCommandError> {
    let store = FileSessionStore::for_workspace(workspace);
    match session_id {
        Some(session_id) => match store.load_session(session_id) {
            Ok(Some(record)) => Ok(record.latest_trace_ref),
            Ok(None) => Err(InspectCommandError::UnknownSession(session_id.to_string())),
            Err(SessionStoreError::InvalidRecord(message)) => {
                Err(InspectCommandError::InvalidSession(format!(
                    "session `{session_id}` is invalid: {message}"
                )))
            }
            Err(error) => Err(InspectCommandError::SessionStore(error)),
        },
        None => match store.load() {
            Ok(Some(record)) => Ok(record.latest_trace_ref),
            Ok(None) => Ok(None),
            Err(SessionStoreError::InvalidRecord(message)) => {
                Err(InspectCommandError::InvalidSession(format!(
                    "active session is invalid: {message}"
                )))
            }
            Err(error) => Err(InspectCommandError::SessionStore(error)),
        },
    }
}

pub(super) fn resolve_trace_artifacts(
    trace_ref: &Path,
    session_id: &str,
) -> Result<ResolvedTraceArtifacts, TraceSummaryError> {
    let workspace_root = trace_workspace_root(trace_ref);
    let goal_brief_ref = workspace_root.as_ref().and_then(|workspace| {
        persisted_session_brief_ref(workspace, &session_goal_brief_ref(session_id))
    });
    let session_plan_brief_ref = workspace_root.as_ref().and_then(|workspace| {
        persisted_session_brief_ref(workspace, &session_plan_brief_ref(session_id))
    });
    let run_brief_ref = workspace_root.as_ref().and_then(|workspace| {
        persisted_session_brief_ref(workspace, &session_run_brief_ref(session_id))
    });
    let session_audit = workspace_root
        .as_ref()
        .map(|workspace| load_session_audit_projection(workspace, session_id))
        .transpose()?
        .flatten();
    let latest_framework_adapter_hook_dispatch = workspace_root
        .as_ref()
        .map(|workspace| load_latest_framework_adapter_hook_dispatch(workspace, session_id))
        .transpose()?
        .flatten();

    Ok(ResolvedTraceArtifacts {
        goal_brief_ref,
        session_plan_brief_ref,
        run_brief_ref,
        session_audit,
        latest_framework_adapter_hook_dispatch,
    })
}

fn trace_workspace_root(trace_ref: &Path) -> Option<PathBuf> {
    let mut current = trace_ref.parent()?;
    loop {
        if current.file_name().and_then(|name| name.to_str()) == Some(".boundline") {
            return current.parent().map(Path::to_path_buf);
        }
        current = current.parent()?;
    }
}

fn persisted_session_brief_ref(workspace: &Path, brief_ref: &str) -> Option<String> {
    workspace.join(brief_ref).is_file().then(|| brief_ref.to_string())
}

fn load_session_audit_projection(
    workspace: &Path,
    session_id: &str,
) -> Result<Option<SessionAuditProjection>, TraceSummaryError> {
    let store = FileSessionAuditStore::for_session(workspace, session_id);
    let entries = store.load_all().map_err(TraceSummaryError::SessionAuditStore)?;
    if entries.is_empty() {
        return Ok(None);
    }

    Ok(Some(SessionAuditProjection::from_entries(session_id.to_string(), entries)))
}

fn load_latest_framework_adapter_hook_dispatch(
    workspace: &Path,
    session_id: &str,
) -> Result<Option<HookEventDispatchRecord>, TraceSummaryError> {
    let store = FileSessionAuditStore::for_session(workspace, session_id);
    let dispatches = store.load_hook_dispatches().map_err(TraceSummaryError::SessionAuditStore)?;
    Ok(dispatches.into_iter().last())
}
