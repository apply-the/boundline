use std::collections::BTreeSet;
use std::path::Path;

use crate::domain::configuration::RuntimeKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssistantSurface {
    SharedReadme,
    Claude,
    Codex,
    Copilot,
    Gemini,
}

impl AssistantSurface {
    pub const fn plan_label(self) -> &'static str {
        match self {
            Self::SharedReadme => "assistant shared docs",
            Self::Claude => "Claude command pack",
            Self::Codex => "Codex command pack",
            Self::Copilot => "Copilot prompt pack",
            Self::Gemini => "Gemini CLI notes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocsExportSurface {
    Canon,
    AssistantShared,
    Claude,
    Codex,
    Copilot,
    Gemini,
}

impl DocsExportSurface {
    pub const fn plan_label(self) -> &'static str {
        match self {
            Self::Canon => "Canon reference docs",
            Self::AssistantShared => "assistant shared docs",
            Self::Claude => "Claude command pack docs",
            Self::Codex => "Codex command pack docs",
            Self::Copilot => "Copilot prompt pack docs",
            Self::Gemini => "Gemini CLI docs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistantAsset {
    pub relative_path: &'static str,
    pub contents: &'static str,
    pub surface: AssistantSurface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsExportAsset {
    pub relative_path: String,
    pub contents: &'static str,
    pub surface: DocsExportSurface,
}

macro_rules! asset {
    ($surface:expr, $path:literal) => {
        AssistantAsset {
            relative_path: $path,
            contents: include_str!(concat!("../../", $path)),
            surface: $surface,
        }
    };
}

const README_ASSET: AssistantAsset = asset!(AssistantSurface::SharedReadme, "assistant/README.md");

const CANON_DOCS_EXPORT_CONTENT: &str = r#"# Boundline And Canon

Boundline is the primary workspace tool. Canon is optional and only participates
when you explicitly choose governed execution.

## Where Files Go

- Boundline session state: `.boundline/session.json`
- Boundline routing and workspace preferences: `.boundline/config.toml`
- Boundline compatibility execution profile: `.boundline/execution.json`
- Boundline traces and checkpoints: `.boundline/traces/`, `.boundline/checkpoints/`
- Canon governed artifacts, when a governed runtime runs: `.canon/runs/<run-ref>/...`

## Session Naming

This file is a stable repo-local reference exported by `boundline init --export-docs`.
Documentation export is create-only by default. Rerun with `--refresh` to update
it in place, `--diff` to preview changes without writing, or `--to <path>` to
export it under another root. This file is not emitted per session, so it does
not use slugs or timestamps.
"#;

static CLAUDE_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-architecture.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-backlog.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-capture.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-change.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-config-set-canon.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-config-show.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-discovery.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-doctor.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-implementation.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-incident.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-init.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-inspect.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-migration.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-next.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-plan.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-refactor.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-requirements.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-review.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-run.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-security-assessment.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-start.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-status.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-step.md"),
    asset!(
        AssistantSurface::Claude,
        "assistant/claude/commands/boundline-supply-chain-analysis.md"
    ),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-system-assessment.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-system-shaping.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-verification.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-workflow-inspect.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-workflow-list.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-workflow-resume.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-workflow-run.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-workflow-status.md"),
];

static CODEX_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-architecture.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-backlog.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-capture.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-change.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-config-set-canon.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-config-show.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-discovery.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-doctor.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-implementation.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-incident.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-init.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-inspect.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-migration.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-next.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-plan.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-refactor.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-requirements.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-review.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-run.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-security-assessment.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-start.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-status.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-step.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-supply-chain-analysis.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-system-assessment.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-system-shaping.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-verification.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-workflow-inspect.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-workflow-list.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-workflow-resume.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-workflow-run.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-workflow-status.md"),
];

static COPILOT_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-architecture.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-backlog.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-capture.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-change.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-config-set-canon.prompt.md"
    ),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-config-show.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-discovery.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-doctor.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-implementation.prompt.md"
    ),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-incident.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-init.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-inspect.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-migration.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-next.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-plan.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-refactor.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-requirements.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-review.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-run.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-security-assessment.prompt.md"
    ),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-start.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-status.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-step.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-supply-chain-analysis.prompt.md"
    ),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-system-assessment.prompt.md"
    ),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-system-shaping.prompt.md"
    ),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-verification.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-workflow-inspect.prompt.md"
    ),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-workflow-list.prompt.md"
    ),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-workflow-resume.prompt.md"
    ),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-workflow-run.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-workflow-status.prompt.md"
    ),
];

static GEMINI_ASSETS: &[AssistantAsset] =
    &[asset!(AssistantSurface::Gemini, "assistant/gemini/README.md")];

pub fn assets_for_assistants(assistants: &[RuntimeKind]) -> Vec<&'static AssistantAsset> {
    if assistants.is_empty() {
        return Vec::new();
    }

    let mut assets = vec![&README_ASSET];
    let mut seen = BTreeSet::from([README_ASSET.relative_path]);
    for runtime in assistants.iter().copied() {
        for asset in runtime_assets(runtime) {
            if seen.insert(asset.relative_path) {
                assets.push(asset);
            }
        }
    }
    assets
}

pub fn docs_assets_for_assistants(assistants: &[RuntimeKind]) -> Vec<DocsExportAsset> {
    docs_assets_for_assistants_under(assistants, Path::new("docs/boundline"))
}

pub fn docs_assets_for_assistants_under(
    assistants: &[RuntimeKind],
    docs_root: &Path,
) -> Vec<DocsExportAsset> {
    let mut assets = vec![DocsExportAsset {
        relative_path: docs_root.join("canon.md").to_string_lossy().into_owned(),
        contents: CANON_DOCS_EXPORT_CONTENT,
        surface: DocsExportSurface::Canon,
    }];

    if assistants.is_empty() {
        return assets;
    }

    let assistant_readme_path = docs_relative_path_for_asset_under(docs_root, &README_ASSET);
    let mut seen = BTreeSet::from([
        docs_root.join("canon.md").to_string_lossy().into_owned(),
        assistant_readme_path.clone(),
    ]);
    assets.push(DocsExportAsset {
        relative_path: assistant_readme_path,
        contents: README_ASSET.contents,
        surface: DocsExportSurface::AssistantShared,
    });

    for runtime in assistants.iter().copied() {
        for asset in runtime_assets(runtime) {
            let relative_path = docs_relative_path_for_asset_under(docs_root, asset);
            if seen.insert(relative_path.clone()) {
                assets.push(DocsExportAsset {
                    relative_path,
                    contents: asset.contents,
                    surface: docs_surface_for_runtime(runtime),
                });
            }
        }
    }

    assets
}

fn runtime_assets(runtime: RuntimeKind) -> &'static [AssistantAsset] {
    match runtime {
        RuntimeKind::Claude => CLAUDE_ASSETS,
        RuntimeKind::Codex => CODEX_ASSETS,
        RuntimeKind::Copilot => COPILOT_ASSETS,
        RuntimeKind::Gemini => GEMINI_ASSETS,
    }
}

fn docs_relative_path_for_asset_under(docs_root: &Path, asset: &AssistantAsset) -> String {
    let suffix = asset.relative_path.strip_prefix("assistant/").unwrap_or(asset.relative_path);
    docs_root.join("assistant").join(suffix).to_string_lossy().into_owned()
}

fn docs_surface_for_runtime(runtime: RuntimeKind) -> DocsExportSurface {
    match runtime {
        RuntimeKind::Claude => DocsExportSurface::Claude,
        RuntimeKind::Codex => DocsExportSurface::Codex,
        RuntimeKind::Copilot => DocsExportSurface::Copilot,
        RuntimeKind::Gemini => DocsExportSurface::Gemini,
    }
}
