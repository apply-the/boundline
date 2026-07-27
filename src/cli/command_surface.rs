//! Canonical runtime classification for the Boundline command surface.
//!
//! The parser, stable help, completion metadata, and migration diagnostics all
//! validate against this inventory so a planning-only command cannot become a
//! public compatibility promise accidentally.
//!
//! Clap derives its parser from Rust enum variants and cannot consume a runtime
//! slice as its source. The CLI contract therefore binds the derived
//! registrations and rendered help back to this lower-dependency inventory.

/// Runtime compatibility classification for an implemented or retired command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandClassification {
    /// Public command with an operational handler and stable compatibility promise.
    StableOperational,
    /// Explicitly unstable command available only through the preview gateway.
    Preview,
    /// Preview command retained temporarily until its stable replacement exists.
    PreviewTransitional,
    /// Rust capability intentionally absent from the public parser.
    Internal,
    /// Retired public spelling with deterministic migration guidance.
    Removed,
}

/// One canonical command classification entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSurfaceEntry {
    /// Command spelling without the executable prefix.
    pub name: &'static str,
    /// Current runtime compatibility classification.
    pub classification: CommandClassification,
    /// Complete diagnostic emitted when a retired root spelling is invoked.
    pub migration_diagnostic: Option<&'static str>,
}

const fn entry(name: &'static str, classification: CommandClassification) -> CommandSurfaceEntry {
    CommandSurfaceEntry { name, classification, migration_diagnostic: None }
}

const fn removed(name: &'static str, diagnostic: &'static str) -> CommandSurfaceEntry {
    CommandSurfaceEntry {
        name,
        classification: CommandClassification::Removed,
        migration_diagnostic: Some(diagnostic),
    }
}

const COMMAND_SURFACE: &[CommandSurfaceEntry] = &[
    entry("init", CommandClassification::StableOperational),
    entry("goal", CommandClassification::StableOperational),
    entry("plan", CommandClassification::StableOperational),
    entry("run", CommandClassification::StableOperational),
    entry("status", CommandClassification::StableOperational),
    entry("inspect", CommandClassification::StableOperational),
    entry("doctor", CommandClassification::StableOperational),
    entry("config", CommandClassification::StableOperational),
    entry("models", CommandClassification::StableOperational),
    entry("provider", CommandClassification::StableOperational),
    entry("adapter", CommandClassification::StableOperational),
    entry("index", CommandClassification::StableOperational),
    entry("session", CommandClassification::StableOperational),
    entry("assistant", CommandClassification::StableOperational),
    entry("update", CommandClassification::StableOperational),
    entry("flow", CommandClassification::Preview),
    entry("workflow", CommandClassification::Preview),
    entry("cluster", CommandClassification::Preview),
    entry("council", CommandClassification::Preview),
    entry("evals", CommandClassification::Preview),
    entry("override", CommandClassification::PreviewTransitional),
    entry("trace", CommandClassification::PreviewTransitional),
    entry("checkpoint", CommandClassification::Internal),
    entry("exec", CommandClassification::Internal),
    removed(
        "orchestrate",
        "`orchestrate` was removed in Boundline 0.90.\nUse `boundline run --until ...`.",
    ),
    removed("step", "`step` was removed in Boundline 0.90.\nUse `boundline run --one-step`."),
    removed("continue", "`continue` was removed in Boundline 0.90.\nUse `boundline run --resume`."),
    removed(
        "next",
        "`next` was removed in Boundline 0.90.\nUse structured `next_actions` from `boundline status`.",
    ),
    removed(
        "probe",
        "`probe` was removed in Boundline 0.90.\nUse `boundline doctor` or `boundline status`.",
    ),
    removed(
        "help-next",
        "`help-next` was removed in Boundline 0.90.\nUse `boundline doctor` or `boundline status`.",
    ),
    removed(
        "govern",
        "`govern` was removed in Boundline 0.90.\nUse `boundline plan` or `boundline run`.",
    ),
];

/// Returns the canonical implemented and retired command inventory.
pub const fn command_surface() -> &'static [CommandSurfaceEntry] {
    COMMAND_SURFACE
}

/// Returns the ordered StableOperational command names used by help and completion checks.
pub fn stable_command_names() -> Vec<&'static str> {
    command_names(CommandClassification::StableOperational)
}

/// Returns stable shell-completion metadata from the canonical inventory.
pub fn stable_completion_command_names() -> Vec<&'static str> {
    stable_command_names()
}

/// Returns the deterministic migration diagnostic for a removed root spelling.
pub fn removed_command_diagnostic(name: &str) -> Option<&'static str> {
    COMMAND_SURFACE
        .iter()
        .find(|entry| entry.name == name && entry.classification == CommandClassification::Removed)
        .and_then(|entry| entry.migration_diagnostic)
}

/// Returns the ordered names in one canonical runtime classification.
pub fn command_names(classification: CommandClassification) -> Vec<&'static str> {
    COMMAND_SURFACE
        .iter()
        .filter(|entry| entry.classification == classification)
        .map(|entry| entry.name)
        .collect()
}

/// Returns both preview classes in their canonical gateway order.
pub fn preview_command_names() -> Vec<&'static str> {
    COMMAND_SURFACE
        .iter()
        .filter(|entry| {
            matches!(
                entry.classification,
                CommandClassification::Preview | CommandClassification::PreviewTransitional
            )
        })
        .map(|entry| entry.name)
        .collect()
}
