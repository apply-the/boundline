//! Session filesystem path helpers.
//!
//! Centralized path construction for session storage, briefs, traces,
//! checkpoints, and audit artifacts. All paths are relative to the
//! `.boundline` state root.

const BOUNDLINE_STATE_ROOT: &str = ".boundline";
const LEGACY_SESSION_RECORD_FILE_NAME: &str = "session.json";
const ACTIVE_SESSION_POINTER_FILE_NAME: &str = "active-session";
const SESSION_STORAGE_ROOT: &str = ".boundline/sessions";
const SESSION_BRANCH_PREFIX: &str = "session";
const SESSION_REF_DEFAULT_SLUG: &str = "session";
const SESSION_REF_SEPARATOR: char = '-';
const SESSION_REF_MAX_SLUG_LENGTH: usize = 32;
const SESSION_REF_DAILY_SEQ_WIDTH: usize = 3;
const SESSION_BRIEFS_DIRECTORY_NAME: &str = "briefs";
const SESSION_TRACES_DIRECTORY_NAME: &str = "traces";
const SESSION_CHECKPOINTS_DIRECTORY_NAME: &str = "checkpoints";
const SESSION_AUDIT_DIRECTORY_NAME: &str = "audit";
const SESSION_GOAL_BRIEF_FILE_NAME: &str = "goal.md";
const SESSION_PLAN_BRIEF_FILE_NAME: &str = "plan.md";
const SESSION_RUN_BRIEF_FILE_NAME: &str = "run.md";
const SESSION_AUDIT_EVENTS_FILE_NAME: &str = "events.jsonl";
const SESSION_AUDIT_CURSOR_FILE_NAME: &str = "cursor.json";

pub fn legacy_session_record_ref() -> String {
    format!("{BOUNDLINE_STATE_ROOT}/{LEGACY_SESSION_RECORD_FILE_NAME}")
}

pub fn active_session_pointer_ref() -> String {
    format!("{BOUNDLINE_STATE_ROOT}/{ACTIVE_SESSION_POINTER_FILE_NAME}")
}

pub fn session_storage_root_ref() -> &'static str {
    SESSION_STORAGE_ROOT
}

pub fn session_root_ref(session_ref: &str) -> String {
    format!("{SESSION_STORAGE_ROOT}/{session_ref}")
}

pub fn session_record_ref(session_ref: &str) -> String {
    format!("{}/{LEGACY_SESSION_RECORD_FILE_NAME}", session_root_ref(session_ref))
}

pub fn session_branch_ref(session_ref: &str) -> String {
    format!("{SESSION_BRANCH_PREFIX}/{session_ref}")
}

pub fn session_briefs_root_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_BRIEFS_DIRECTORY_NAME}", session_root_ref(session_ref))
}

pub fn session_traces_root_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_TRACES_DIRECTORY_NAME}", session_root_ref(session_ref))
}

pub fn session_checkpoints_root_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_CHECKPOINTS_DIRECTORY_NAME}", session_root_ref(session_ref))
}

pub fn session_audit_root_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_AUDIT_DIRECTORY_NAME}", session_root_ref(session_ref))
}

pub fn session_audit_events_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_AUDIT_EVENTS_FILE_NAME}", session_audit_root_ref(session_ref))
}

pub fn session_audit_cursor_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_AUDIT_CURSOR_FILE_NAME}", session_audit_root_ref(session_ref))
}

/// Converts a Unix-epoch millisecond timestamp to a `YYYYMMDD` date string
/// (UTC), used as the first segment of a session reference.
///
/// Uses Howard Hinnant's civil-calendar algorithm (public domain):
/// <https://howardhinnant.github.io/date_algorithms.html>
pub fn date_prefix_from_millis(millis: u64) -> String {
    let days = (millis / 86_400_000) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { yoe as i64 + era * 400 + 1 } else { yoe as i64 + era * 400 };
    format!("{y:04}{m:02}{d:02}")
}

pub fn generate_session_ref(
    goal_hint: Option<&str>,
    date_prefix: &str,
    daily_seq: u16,
    slug_override: Option<&str>,
) -> String {
    let slug = normalize_session_slug(slug_override.or(goal_hint));
    let seq = format!("{daily_seq:0>width$}", width = SESSION_REF_DAILY_SEQ_WIDTH);
    format!("{date_prefix}{SESSION_REF_SEPARATOR}{seq}{SESSION_REF_SEPARATOR}{slug}")
}

pub fn normalize_session_slug(goal_hint: Option<&str>) -> String {
    let raw = goal_hint.unwrap_or("").trim();
    if raw.is_empty() {
        return SESSION_REF_DEFAULT_SLUG.to_string();
    }

    let mut slug = String::with_capacity(SESSION_REF_MAX_SLUG_LENGTH);
    let mut hit_limit = false;

    for ch in raw.chars() {
        if slug.len() >= SESSION_REF_MAX_SLUG_LENGTH {
            hit_limit = true;
            break;
        }

        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with(SESSION_REF_SEPARATOR) {
            slug.push(SESSION_REF_SEPARATOR);
        }
    }

    // When truncated, backtrack to the last word boundary (separator) so the
    // slug never ends mid-word.
    if hit_limit && let Some(last_sep) = slug.rfind(SESSION_REF_SEPARATOR) {
        slug.truncate(last_sep);
    }

    let final_slug = slug.trim_end_matches(SESSION_REF_SEPARATOR);

    if final_slug.is_empty() {
        SESSION_REF_DEFAULT_SLUG.to_string()
    } else {
        final_slug.to_string()
    }
}

pub fn session_goal_brief_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_GOAL_BRIEF_FILE_NAME}", session_briefs_root_ref(session_ref))
}

pub fn session_plan_brief_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_PLAN_BRIEF_FILE_NAME}", session_briefs_root_ref(session_ref))
}

pub fn session_run_brief_ref(session_ref: &str) -> String {
    format!("{}/{SESSION_RUN_BRIEF_FILE_NAME}", session_briefs_root_ref(session_ref))
}
