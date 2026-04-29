//! Flow inference from goal text and workspace signals (feature 013).

use crate::domain::goal_plan::InferredFlow;

/// Keywords that map to the bug-fix flow.
const BUG_FIX_KEYWORDS: &[&str] =
    &["fix", "bug", "broken", "failing", "regression", "crash", "error"];

/// Keywords that map to the change flow.
const CHANGE_KEYWORDS: &[&str] =
    &["add", "implement", "feature", "new", "create", "extend", "refactor"];

/// Keywords that map to the delivery flow.
const DELIVERY_KEYWORDS: &[&str] = &["deliver", "release", "ship", "deploy", "complete", "launch"];

/// Infer a flow from goal text using keyword matching.
///
/// Returns `Some(InferredFlow)` if a match is found, `None` if no keywords
/// match (caller should default to no-flow mode).
pub fn infer_flow(goal_text: &str) -> Option<InferredFlow> {
    let lower = goal_text.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Check bug-fix first (highest priority for safety)
    for keyword in BUG_FIX_KEYWORDS {
        if words.iter().any(|w| w.contains(keyword)) {
            return Some(InferredFlow {
                flow_name: "bug-fix".to_string(),
                confidence_reason: format!("goal contains keyword '{keyword}'"),
                confirmed: false,
            });
        }
    }

    // Check delivery before change (broader scope takes precedence)
    for keyword in DELIVERY_KEYWORDS {
        if words.iter().any(|w| w.contains(keyword)) {
            return Some(InferredFlow {
                flow_name: "delivery".to_string(),
                confidence_reason: format!("goal contains keyword '{keyword}'"),
                confirmed: false,
            });
        }
    }

    // Check change last
    for keyword in CHANGE_KEYWORDS {
        if words.iter().any(|w| w.contains(keyword)) {
            return Some(InferredFlow {
                flow_name: "change".to_string(),
                confidence_reason: format!("goal contains keyword '{keyword}'"),
                confirmed: false,
            });
        }
    }

    None
}
