use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::Path;

use clap::ValueEnum;

use crate::domain::configuration::AssistantHostKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum AssistantHost {
    Claude,
    Codex,
    Cursor,
    Copilot,
    Antigravity,
}

impl AssistantHost {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Copilot => "copilot",
            Self::Antigravity => "antigravity",
        }
    }

    const fn install_mode(self) -> &'static str {
        match self {
            Self::Claude | Self::Codex | Self::Cursor => "copy_ready_assets",
            Self::Copilot | Self::Antigravity => "manual_fallback",
        }
    }

    const fn package_path(self) -> &'static str {
        match self {
            Self::Claude => "assistant/global/claude",
            Self::Codex => "assistant/global/codex",
            Self::Cursor => "assistant/global/cursor",
            Self::Copilot => "assistant/global/copilot",
            Self::Antigravity => "assistant/global/antigravity",
        }
    }

    const fn fallback_note(self) -> &'static str {
        match self {
            Self::Claude => {
                "Claude-compatible global command assets are copy-ready for user-scoped installation."
            }
            Self::Codex => {
                "Codex-compatible global command assets are copy-ready for user-scoped installation."
            }
            Self::Cursor => {
                "Cursor support is represented as copy-ready command and rule assets; confirm the local Cursor install path before copying."
            }
            Self::Copilot => {
                "Copilot environments vary; global command installation is not claimed for this host."
            }
            Self::Antigravity => {
                "Antigravity exposes a repo-local command pack after `boundline init --assistant antigravity`, but global command installation is not claimed for this host and remains a manual fallback."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum AssistantInstallScope {
    User,
}

impl AssistantInstallScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantInstallReport {
    pub host: AssistantHost,
    pub scope: AssistantInstallScope,
    pub install_mode: &'static str,
    pub package_path: &'static str,
    pub bootstrap_commands: &'static [&'static str],
    pub contextual_commands: &'static [&'static str],
    pub cli_fallback_commands: Vec<String>,
    pub note: &'static str,
}

pub fn install_global_assistant_package(
    host: AssistantHost,
    scope: AssistantInstallScope,
) -> AssistantInstallReport {
    AssistantInstallReport {
        host,
        scope,
        install_mode: host.install_mode(),
        package_path: host.package_path(),
        bootstrap_commands: &[
            "/boundline:init",
            "/boundline:doctor",
            "/boundline:help",
            "/boundline:status",
            "/boundline:continue",
        ],
        contextual_commands: &["/boundline:explain-plan", "/boundline:doctor-context"],
        cli_fallback_commands: vec![
            "boundline init --workspace <workspace> --assistant <host>".to_string(),
            "boundline doctor --workspace <workspace>".to_string(),
            "boundline status --workspace <workspace>".to_string(),
            "boundline continue --workspace <workspace>".to_string(),
        ],
        note: host.fallback_note(),
    }
}

pub fn render_assistant_install_report(report: &AssistantInstallReport) -> String {
    format!(
        concat!(
            "assistant_global_package:\n",
            "host: {}\n",
            "scope: {}\n",
            "install_mode: {}\n",
            "package_path: {}\n",
            "commands:\n",
            "{}\n",
            "contextual_commands:\n",
            "{}\n",
            "fallback_cli:\n",
            "{}\n",
            "note: {}\n"
        ),
        report.host.as_str(),
        report.scope.as_str(),
        report.install_mode,
        report.package_path,
        report
            .bootstrap_commands
            .iter()
            .map(|command| format!("- {command}"))
            .collect::<Vec<_>>()
            .join("\n"),
        report
            .contextual_commands
            .iter()
            .map(|command| format!("- {command}"))
            .collect::<Vec<_>>()
            .join("\n"),
        report
            .cli_fallback_commands
            .iter()
            .map(|command| format!("- {command}"))
            .collect::<Vec<_>>()
            .join("\n"),
        report.note,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssistantSurface {
    SharedReadme,
    Claude,
    Codex,
    Copilot,
    Antigravity,
}

impl AssistantSurface {
    pub const fn plan_label(self) -> &'static str {
        match self {
            Self::SharedReadme => "assistant shared files",
            Self::Claude => "Claude command pack",
            Self::Codex => "Codex command pack",
            Self::Copilot => "Copilot prompt pack",
            Self::Antigravity => "Antigravity command pack",
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
    Antigravity,
}

impl DocsExportSurface {
    pub const fn plan_label(self) -> &'static str {
        match self {
            Self::Canon => "Canon reference docs",
            Self::AssistantShared => "assistant shared docs",
            Self::Claude => "Claude command pack docs",
            Self::Codex => "Codex command pack docs",
            Self::Copilot => "Copilot prompt pack docs",
            Self::Antigravity => "Antigravity command pack docs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantAsset {
    pub relative_path: Cow<'static, str>,
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
            relative_path: Cow::Borrowed($path),
            contents: include_str!(concat!("../../", $path)),
            surface: $surface,
        }
    };
}

const README_ASSET: AssistantAsset = asset!(AssistantSurface::SharedReadme, "assistant/README.md");

static SHARED_SCAFFOLD_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::SharedReadme, "assistant/plugin-metadata.json"),
    asset!(AssistantSurface::SharedReadme, "assistant/commands/session-workflow.json"),
    asset!(AssistantSurface::SharedReadme, "assistant/prompts/goal-template.md"),
    asset!(AssistantSurface::SharedReadme, "assistant/prompts/starter-prompts.md"),
    asset!(AssistantSurface::SharedReadme, "assistant/assets/boundline-plugin-icon.svg"),
    asset!(AssistantSurface::SharedReadme, "assistant/assets/boundline-plugin-logo.svg"),
];

static SHARED_DOC_ASSETS: &[AssistantAsset] =
    &[asset!(AssistantSurface::SharedReadme, "assistant/prompts/goal-template.md")];

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
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-goal.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-config-set-canon.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-config-show.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-doctor.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-govern.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-inspect.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-next.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-plan.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-run.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-status.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-step.md"),
    asset!(AssistantSurface::Claude, "assistant/claude/commands/boundline-update.md"),
];

static CLAUDE_PACKAGE_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Claude, ".claude-plugin/manifest.json"),
    asset!(AssistantSurface::Claude, ".claude-plugin/commands.json"),
];

static CODEX_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-goal.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-config-set-canon.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-config-show.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-doctor.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-govern.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-inspect.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-next.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-plan.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-run.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-status.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-step.md"),
    asset!(AssistantSurface::Codex, "assistant/codex/commands/boundline-update.md"),
];

static CODEX_PACKAGE_ASSETS: &[AssistantAsset] =
    &[asset!(AssistantSurface::Codex, ".codex-plugin/plugin.json")];

static COPILOT_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-goal.prompt.md"),
    asset!(
        AssistantSurface::Copilot,
        "assistant/copilot/prompts/boundline-config-set-canon.prompt.md"
    ),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-config-show.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-doctor.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-govern.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-inspect.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-next.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-plan.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-run.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-status.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-step.prompt.md"),
    asset!(AssistantSurface::Copilot, "assistant/copilot/prompts/boundline-update.prompt.md"),
];

static COPILOT_SHARED_ASSETS: &[AssistantAsset] =
    &[asset!(AssistantSurface::Copilot, "assistant/prompts/copilot-command-pack.md")];

static COPILOT_PACKAGE_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Copilot, ".copilot-prompts/README.md"),
    asset!(AssistantSurface::Copilot, ".copilot-prompts/pack.json"),
];

static ANTIGRAVITY_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/README.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-goal.md"),
    asset!(
        AssistantSurface::Antigravity,
        "assistant/antigravity/commands/boundline-config-set-canon.md"
    ),
    asset!(
        AssistantSurface::Antigravity,
        "assistant/antigravity/commands/boundline-config-show.md"
    ),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-doctor.md"),
    asset!(
        AssistantSurface::Antigravity,
        "assistant/antigravity/commands/boundline-doctor-context.md"
    ),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-govern.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-inspect.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-next.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-next-best.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-plan.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-recover.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-run.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-status.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-step.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-update.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-why.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-risk.md"),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-evidence.md"),
    asset!(
        AssistantSurface::Antigravity,
        "assistant/antigravity/commands/boundline-assumptions.md"
    ),
    asset!(
        AssistantSurface::Antigravity,
        "assistant/antigravity/commands/boundline-hidden-impact.md"
    ),
    asset!(AssistantSurface::Antigravity, "assistant/antigravity/commands/boundline-challenge.md"),
    asset!(
        AssistantSurface::Antigravity,
        "assistant/antigravity/commands/boundline-explain-plan.md"
    ),
];

static ANTIGRAVITY_PACKAGE_ASSETS: &[AssistantAsset] = &[
    asset!(AssistantSurface::Antigravity, ".antigravity-plugin/manifest.json"),
    asset!(AssistantSurface::Antigravity, ".antigravity-plugin/commands.json"),
];

pub fn assets_for_assistants(assistants: &[AssistantHostKind]) -> Vec<AssistantAsset> {
    if assistants.is_empty() {
        return Vec::new();
    }

    let mut assets = vec![README_ASSET.clone()];
    let mut seen = BTreeSet::from([README_ASSET.relative_path.to_string()]);
    extend_assets(&mut assets, &mut seen, SHARED_SCAFFOLD_ASSETS.iter().cloned());
    for host in assistants.iter().copied() {
        extend_assets(&mut assets, &mut seen, host_scaffold_assets(host));
        extend_assets(&mut assets, &mut seen, host_package_assets(host).iter().cloned());
    }

    if assistants.iter().copied().any(|host| host == AssistantHostKind::Copilot) {
        extend_assets(&mut assets, &mut seen, projected_copilot_prompt_assets());
    }

    assets
}

pub fn docs_assets_for_assistants(assistants: &[AssistantHostKind]) -> Vec<DocsExportAsset> {
    docs_assets_for_assistants_under(assistants, Path::new("docs/boundline"))
}

pub fn docs_assets_for_assistants_under(
    assistants: &[AssistantHostKind],
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
    for asset in SHARED_DOC_ASSETS {
        let relative_path = docs_relative_path_for_asset_under(docs_root, asset);
        if seen.insert(relative_path.clone()) {
            assets.push(DocsExportAsset {
                relative_path,
                contents: asset.contents,
                surface: DocsExportSurface::AssistantShared,
            });
        }
    }

    for host in assistants.iter().copied() {
        for asset in host_assets(host) {
            let relative_path = docs_relative_path_for_asset_under(docs_root, asset);
            if seen.insert(relative_path.clone()) {
                assets.push(DocsExportAsset {
                    relative_path,
                    contents: asset.contents,
                    surface: docs_surface_for_host(host),
                });
            }
        }
    }

    assets
}

fn host_assets(host: AssistantHostKind) -> &'static [AssistantAsset] {
    match host {
        AssistantHostKind::Claude => CLAUDE_ASSETS,
        AssistantHostKind::Codex => CODEX_ASSETS,
        AssistantHostKind::Copilot => COPILOT_ASSETS,
        AssistantHostKind::Antigravity => ANTIGRAVITY_ASSETS,
    }
}

fn host_scaffold_assets(host: AssistantHostKind) -> Vec<AssistantAsset> {
    let mut assets = host_assets(host).to_vec();
    if host == AssistantHostKind::Copilot {
        assets.extend(COPILOT_SHARED_ASSETS.iter().cloned());
    }
    assets
}

fn host_package_assets(host: AssistantHostKind) -> &'static [AssistantAsset] {
    match host {
        AssistantHostKind::Claude => CLAUDE_PACKAGE_ASSETS,
        AssistantHostKind::Codex => CODEX_PACKAGE_ASSETS,
        AssistantHostKind::Copilot => COPILOT_PACKAGE_ASSETS,
        AssistantHostKind::Antigravity => ANTIGRAVITY_PACKAGE_ASSETS,
    }
}

fn projected_copilot_prompt_assets() -> Vec<AssistantAsset> {
    COPILOT_ASSETS
        .iter()
        .map(|asset| {
            let file_name = asset
                .relative_path
                .rsplit('/')
                .next()
                .filter(|name| !name.is_empty())
                .unwrap_or(asset.relative_path.as_ref());
            AssistantAsset {
                relative_path: Cow::Owned(format!(".github/prompts/{file_name}")),
                contents: asset.contents,
                surface: asset.surface,
            }
        })
        .collect()
}

fn extend_assets<I>(assets: &mut Vec<AssistantAsset>, seen: &mut BTreeSet<String>, new_assets: I)
where
    I: IntoIterator<Item = AssistantAsset>,
{
    for asset in new_assets {
        if seen.insert(asset.relative_path.to_string()) {
            assets.push(asset);
        }
    }
}

fn docs_relative_path_for_asset_under(docs_root: &Path, asset: &AssistantAsset) -> String {
    let suffix = asset
        .relative_path
        .as_ref()
        .strip_prefix("assistant/")
        .unwrap_or(asset.relative_path.as_ref());
    docs_root.join("assistant").join(suffix).to_string_lossy().into_owned()
}

fn docs_surface_for_host(host: AssistantHostKind) -> DocsExportSurface {
    match host {
        AssistantHostKind::Claude => DocsExportSurface::Claude,
        AssistantHostKind::Codex => DocsExportSurface::Codex,
        AssistantHostKind::Copilot => DocsExportSurface::Copilot,
        AssistantHostKind::Antigravity => DocsExportSurface::Antigravity,
    }
}
