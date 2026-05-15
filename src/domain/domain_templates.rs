//! Domain-family detection and external-context template models.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported domain families used to tailor guidance and context assembly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum DomainFamily {
    Systems,
    JvmService,
    DotNetService,
    PythonService,
    NodeService,
    WebUi,
    React,
    Vue,
    Angular,
    Ruby,
    Php,
    Data,
    Mobile,
}

impl DomainFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Systems => "systems",
            Self::JvmService => "jvm_service",
            Self::DotNetService => "dotnet_service",
            Self::PythonService => "python_service",
            Self::NodeService => "node_service",
            Self::WebUi => "web_ui",
            Self::React => "react",
            Self::Vue => "vue",
            Self::Angular => "angular",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Data => "data",
            Self::Mobile => "mobile",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Systems => "Systems",
            Self::JvmService => "JVM Service",
            Self::DotNetService => ".NET Service",
            Self::PythonService => "Python Service",
            Self::NodeService => "Node Service",
            Self::WebUi => "Web UI",
            Self::React => "React",
            Self::Vue => "Vue",
            Self::Angular => "Angular",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::Data => "Data",
            Self::Mobile => "Mobile",
        }
    }

    pub const fn built_in_summary(self) -> &'static str {
        match self {
            Self::Systems => {
                "Prefer small, explicit changes with strong type safety, deterministic validation, and clear ownership of source and test targets."
            }
            Self::JvmService => {
                "Preserve service boundaries, framework conventions, and configuration clarity while keeping changes easy to validate end to end."
            }
            Self::DotNetService => {
                "Respect solution structure, dependency-injection boundaries, and concise diagnostics while keeping changes aligned with existing service conventions."
            }
            Self::PythonService => {
                "Favor readable modules, explicit validation paths, and framework-appropriate patterns without drifting into implicit magic."
            }
            Self::NodeService => {
                "Preserve API boundaries, runtime safety, and testability while keeping server-side changes scoped to the relevant route or service layer."
            }
            Self::WebUi => {
                "Keep UI behavior intentional, accessible, and bounded by visible component or page targets instead of generic styling churn."
            }
            Self::React => {
                "Prefer clear component boundaries, predictable state updates, and framework-idiomatic rendering over ad hoc UI rewrites."
            }
            Self::Vue => {
                "Respect component single-responsibility, reactive data flow, and framework conventions without mixing unrelated UI concerns."
            }
            Self::Angular => {
                "Keep module, template, and service boundaries explicit while aligning with existing dependency-injection and change-detection patterns."
            }
            Self::Ruby => {
                "Prefer idiomatic application flow, small model/controller/service changes, and framework-consistent tests over broad rewrites."
            }
            Self::Php => {
                "Preserve application conventions, request lifecycle clarity, and validation boundaries while keeping changes easy to inspect and verify."
            }
            Self::Data => {
                "Favor traceable data movement, explicit assumptions, and reproducible validation over opaque notebook-style experimentation."
            }
            Self::Mobile => {
                "Respect platform patterns, bounded screen or feature changes, and visible design-system constraints while keeping validation targeted."
            }
        }
    }

    pub const fn all() -> [Self; 13] {
        [
            Self::Systems,
            Self::JvmService,
            Self::DotNetService,
            Self::PythonService,
            Self::NodeService,
            Self::WebUi,
            Self::React,
            Self::Vue,
            Self::Angular,
            Self::Ruby,
            Self::Php,
            Self::Data,
            Self::Mobile,
        ]
    }
}

/// Supported kinds of external context that can be bound to a domain template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ExternalContextKind {
    DesignReference,
    DesignSystem,
    DesignTokens,
    PlatformGuidance,
    ApiContract,
    Custom,
}

impl ExternalContextKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DesignReference => "design_reference",
            Self::DesignSystem => "design_system",
            Self::DesignTokens => "design_tokens",
            Self::PlatformGuidance => "platform_guidance",
            Self::ApiContract => "api_contract",
            Self::Custom => "custom",
        }
    }
}

/// Availability status of one resolved external-context binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalContextStatus {
    Used,
    Unavailable,
    Stale,
    Skipped,
}

impl ExternalContextStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Used => "used",
            Self::Unavailable => "unavailable",
            Self::Stale => "stale",
            Self::Skipped => "skipped",
        }
    }
}

/// One configured external-context binding attached to a domain template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalContextBinding {
    pub kind: ExternalContextKind,
    pub reference: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ExternalContextBinding {
    /// Validates the external-context binding.
    pub fn validate(&self) -> Result<(), DomainTemplateError> {
        if self.reference.trim().is_empty() {
            return Err(DomainTemplateError::MissingExternalContextReference);
        }
        if self.notes.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(DomainTemplateError::InvalidExternalContextNotes);
        }
        Ok(())
    }

    /// Resolves the binding reference to a workspace-local path when possible.
    pub fn resolved_path(&self, workspace_ref: &Path) -> Option<PathBuf> {
        if has_external_scheme(&self.reference) {
            return None;
        }

        let raw = self.reference.strip_prefix("file:").unwrap_or(self.reference.as_str());
        let candidate = PathBuf::from(raw);
        if candidate.is_absolute() { Some(candidate) } else { Some(workspace_ref.join(candidate)) }
    }

    /// Computes the effective status of the binding for the selected target.
    pub fn status_for_target(
        &self,
        workspace_ref: &Path,
        selected_target: Option<&str>,
    ) -> ExternalContextStatus {
        let Some(reference_path) = self.resolved_path(workspace_ref) else {
            return ExternalContextStatus::Used;
        };

        if !reference_path.is_file() {
            return ExternalContextStatus::Unavailable;
        }

        let Some(selected_target) = selected_target else {
            return ExternalContextStatus::Used;
        };
        let target_path = workspace_ref.join(selected_target);
        if !target_path.is_file() {
            return ExternalContextStatus::Used;
        }

        let Ok(reference_meta) = fs::metadata(reference_path) else {
            return ExternalContextStatus::Unavailable;
        };
        let Ok(target_meta) = fs::metadata(target_path) else {
            return ExternalContextStatus::Used;
        };
        let Ok(reference_modified) = reference_meta.modified() else {
            return ExternalContextStatus::Used;
        };
        let Ok(target_modified) = target_meta.modified() else {
            return ExternalContextStatus::Used;
        };

        if target_modified > reference_modified {
            ExternalContextStatus::Stale
        } else {
            ExternalContextStatus::Used
        }
    }
}

/// Persisted settings for one domain-family template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DomainTemplateSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards: Option<String>,
    #[serde(default)]
    pub external_context_bindings: Vec<ExternalContextBinding>,
}

impl DomainTemplateSettings {
    /// Validates the domain-template settings.
    pub fn validate(&self) -> Result<(), DomainTemplateError> {
        if self.standards.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(DomainTemplateError::InvalidStandardsText);
        }

        for binding in &self.external_context_bindings {
            binding.validate()?;
        }

        Ok(())
    }
}

/// Credibility assigned to applied domain context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedDomainCredibility {
    Credible,
    Insufficient,
    Stale,
}

impl AppliedDomainCredibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credible => "credible",
            Self::Insufficient => "insufficient",
            Self::Stale => "stale",
        }
    }
}

/// One external input carried into an applied domain-context summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedExternalContextInput {
    pub kind: ExternalContextKind,
    pub reference: String,
    pub status: ExternalContextStatus,
    pub required: bool,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Applied domain context selected for planning and guidance assembly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedDomainContext {
    pub families: Vec<DomainFamily>,
    pub summary: String,
    pub credibility: AppliedDomainCredibility,
    pub selected_target: String,
    #[serde(default)]
    pub guidance_sources: Vec<String>,
    #[serde(default)]
    pub external_inputs: Vec<AppliedExternalContextInput>,
    #[serde(default)]
    pub governed_artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_reason: Option<String>,
}

impl AppliedDomainContext {
    /// Validates the applied domain context.
    pub fn validate(&self) -> Result<(), DomainTemplateError> {
        if self.summary.trim().is_empty() {
            return Err(DomainTemplateError::InvalidAppliedDomainSummary);
        }
        if self.selected_target.trim().is_empty() {
            return Err(DomainTemplateError::MissingAppliedDomainTarget);
        }
        if self.credibility == AppliedDomainCredibility::Credible && self.families.is_empty() {
            return Err(DomainTemplateError::MissingAppliedDomainFamily);
        }
        if self.credibility != AppliedDomainCredibility::Credible
            && self.blocking_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(DomainTemplateError::MissingBlockingReason);
        }
        Ok(())
    }
}

/// Detects likely domain families from workspace signals and an optional target.
pub fn detect_domain_families(workspace_ref: &Path, target: Option<&str>) -> Vec<DomainFamily> {
    let mut families = BTreeSet::new();

    if let Some(target) = target {
        families.extend(target_domain_families(workspace_ref, target));
    }

    if families.is_empty() {
        families.extend(workspace_domain_families(workspace_ref));
    }

    families.into_iter().collect()
}

fn workspace_domain_families(workspace_ref: &Path) -> Vec<DomainFamily> {
    let mut families = Vec::new();

    if workspace_ref.join("Cargo.toml").is_file() || workspace_ref.join("go.mod").is_file() {
        families.push(DomainFamily::Systems);
    }
    if workspace_ref.join("pom.xml").is_file()
        || workspace_ref.join("build.gradle").is_file()
        || workspace_ref.join("build.gradle.kts").is_file()
    {
        families.push(DomainFamily::JvmService);
    }
    if contains_top_level_extension(workspace_ref, "sln")
        || workspace_ref.join("Directory.Build.props").is_file()
    {
        families.push(DomainFamily::DotNetService);
    }
    if workspace_ref.join("pyproject.toml").is_file() || workspace_ref.join("setup.py").is_file() {
        families.push(DomainFamily::PythonService);
    }
    if workspace_ref.join("Gemfile").is_file() {
        families.push(DomainFamily::Ruby);
    }
    if workspace_ref.join("composer.json").is_file() {
        families.push(DomainFamily::Php);
    }
    if workspace_ref.join("Package.swift").is_file()
        || workspace_ref.join("android").is_dir()
        || workspace_ref.join("ios").is_dir()
    {
        families.push(DomainFamily::Mobile);
    }

    let package_json = package_json_contents(workspace_ref);
    if package_json.is_some() {
        if package_json_contains(workspace_ref, "react")
            || package_json_contains(workspace_ref, "next")
        {
            families.push(DomainFamily::React);
            families.push(DomainFamily::WebUi);
        }
        if package_json_contains(workspace_ref, "vue")
            || package_json_contains(workspace_ref, "nuxt")
        {
            families.push(DomainFamily::Vue);
            families.push(DomainFamily::WebUi);
        }
        if package_json_contains(workspace_ref, "angular") {
            families.push(DomainFamily::Angular);
            families.push(DomainFamily::WebUi);
        }
        if package_json_contains(workspace_ref, "express")
            || package_json_contains(workspace_ref, "nest")
        {
            families.push(DomainFamily::NodeService);
        }
        if !package_json_contains_any(
            workspace_ref,
            &["react", "next", "vue", "nuxt", "angular", "express", "nest"],
        ) {
            families.push(DomainFamily::NodeService);
        }
    }

    dedup_families(families)
}

fn target_domain_families(workspace_ref: &Path, target: &str) -> Vec<DomainFamily> {
    let lower = target.to_lowercase();
    let extension = Path::new(target)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    let mut families = Vec::new();

    match extension.as_deref() {
        Some("rs") | Some("go") | Some("c") | Some("h") | Some("cc") | Some("cpp")
        | Some("hpp") | Some("zig") => families.push(DomainFamily::Systems),
        Some("java") | Some("kt") => {
            if lower.contains("android") {
                families.push(DomainFamily::Mobile);
            } else {
                families.push(DomainFamily::JvmService);
            }
        }
        Some("cs") => families.push(DomainFamily::DotNetService),
        Some("py") => {
            if lower.contains("/data/")
                || lower.contains("/ml/")
                || lower.contains("notebook")
                || lower.contains("analytics")
            {
                families.push(DomainFamily::Data);
            } else {
                families.push(DomainFamily::PythonService);
            }
        }
        Some("sql") => families.push(DomainFamily::Data),
        Some("rb") => families.push(DomainFamily::Ruby),
        Some("php") => families.push(DomainFamily::Php),
        Some("swift") | Some("dart") => families.push(DomainFamily::Mobile),
        Some("jsx") | Some("tsx") => {
            families.push(DomainFamily::WebUi);
            if package_json_contains(workspace_ref, "react")
                || package_json_contains(workspace_ref, "next")
            {
                families.push(DomainFamily::React);
            }
        }
        Some("js") | Some("ts") => {
            let is_frontend_target = lower.contains("component")
                || lower.contains("/ui/")
                || lower.contains("/app/")
                || lower.contains("/pages/")
                || lower.contains("frontend")
                || lower.contains("client");
            let is_backend_target = lower.contains("server")
                || lower.contains("backend")
                || lower.contains("/api/")
                || lower.contains("service");

            if package_json_contains(workspace_ref, "react")
                || package_json_contains(workspace_ref, "next")
            {
                families.push(DomainFamily::React);
            }
            if package_json_contains(workspace_ref, "vue")
                || package_json_contains(workspace_ref, "nuxt")
            {
                families.push(DomainFamily::Vue);
            }
            if package_json_contains(workspace_ref, "angular") {
                families.push(DomainFamily::Angular);
            }
            if package_json_contains(workspace_ref, "express")
                || package_json_contains(workspace_ref, "nest")
            {
                families.push(DomainFamily::NodeService);
            }
            if is_frontend_target || !is_backend_target {
                families.push(DomainFamily::WebUi);
            }
            if is_backend_target {
                families.push(DomainFamily::NodeService);
            }
        }
        _ => {}
    }

    if families.is_empty() {
        families.extend(workspace_domain_families(workspace_ref));
    }

    dedup_families(families)
}

fn dedup_families(families: Vec<DomainFamily>) -> Vec<DomainFamily> {
    let mut seen = BTreeSet::new();
    families.into_iter().filter(|family| seen.insert(*family)).collect()
}

fn package_json_contains(workspace_ref: &Path, needle: &str) -> bool {
    package_json_contents(workspace_ref)
        .as_deref()
        .is_some_and(|contents| contents.contains(&format!("\"{needle}\"")))
}

fn package_json_contains_any(workspace_ref: &Path, needles: &[&str]) -> bool {
    needles.iter().any(|needle| package_json_contains(workspace_ref, needle))
}

fn package_json_contents(workspace_ref: &Path) -> Option<String> {
    fs::read_to_string(workspace_ref.join("package.json"))
        .ok()
        .map(|contents| contents.to_lowercase())
}

fn contains_top_level_extension(workspace_ref: &Path, extension: &str) -> bool {
    let Ok(entries) = fs::read_dir(workspace_ref) else {
        return false;
    };

    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(extension))
    })
}

fn has_external_scheme(reference: &str) -> bool {
    reference.starts_with("mcp:")
        || reference.starts_with("http:")
        || reference.starts_with("https:")
        || reference.starts_with("tool:")
        || reference.starts_with("canon:")
}

/// Validation errors for domain-template settings and applied domain context.
#[derive(Debug, Error)]
pub enum DomainTemplateError {
    #[error("standards text cannot be empty when provided")]
    InvalidStandardsText,
    #[error("external context reference cannot be empty")]
    MissingExternalContextReference,
    #[error("external context notes cannot be empty when provided")]
    InvalidExternalContextNotes,
    #[error("applied domain summary cannot be empty")]
    InvalidAppliedDomainSummary,
    #[error("applied domain target cannot be empty")]
    MissingAppliedDomainTarget,
    #[error("credible domain context requires at least one selected family")]
    MissingAppliedDomainFamily,
    #[error("non-credible domain context requires a blocking reason")]
    MissingBlockingReason,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use super::{
        AppliedDomainContext, AppliedDomainCredibility, DomainFamily, DomainTemplateSettings,
        ExternalContextBinding, ExternalContextKind, ExternalContextStatus, detect_domain_families,
    };

    static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_workspace(name: &str) -> PathBuf {
        let id = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("boundline-{name}-{id}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn domain_family_metadata_is_defined_for_every_variant() {
        let families = DomainFamily::all();
        let names = families
            .iter()
            .map(|family| family.as_str())
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(families.len(), 13);
        assert_eq!(names.len(), families.len());

        for family in families {
            assert!(!family.as_str().is_empty());
            assert!(!family.display_name().is_empty());
            assert!(!family.built_in_summary().is_empty());
        }
    }

    #[test]
    fn external_context_helpers_cover_variants_and_resolution_paths() {
        let workspace = temp_workspace("domain-template-external-context");
        fs::create_dir_all(workspace.join("docs")).unwrap();
        fs::create_dir_all(workspace.join("src")).unwrap();

        let kinds = [
            ExternalContextKind::DesignReference,
            ExternalContextKind::DesignSystem,
            ExternalContextKind::DesignTokens,
            ExternalContextKind::PlatformGuidance,
            ExternalContextKind::ApiContract,
            ExternalContextKind::Custom,
        ];
        for kind in kinds {
            assert!(!kind.as_str().is_empty());
        }

        let statuses = [
            ExternalContextStatus::Used,
            ExternalContextStatus::Unavailable,
            ExternalContextStatus::Stale,
            ExternalContextStatus::Skipped,
        ];
        for status in statuses {
            assert!(!status.as_str().is_empty());
        }

        let relative = ExternalContextBinding {
            kind: ExternalContextKind::DesignReference,
            reference: "docs/reference.md".to_string(),
            required: true,
            notes: None,
        };
        assert_eq!(relative.resolved_path(&workspace), Some(workspace.join("docs/reference.md")));

        let file_prefixed = ExternalContextBinding {
            kind: ExternalContextKind::DesignSystem,
            reference: "file:docs/system.md".to_string(),
            required: false,
            notes: None,
        };
        assert_eq!(file_prefixed.resolved_path(&workspace), Some(workspace.join("docs/system.md")));

        let absolute_path = workspace.join("docs/absolute.md");
        let absolute = ExternalContextBinding {
            kind: ExternalContextKind::ApiContract,
            reference: absolute_path.to_string_lossy().into_owned(),
            required: false,
            notes: None,
        };
        assert_eq!(absolute.resolved_path(&workspace), Some(absolute_path.clone()));

        let external = ExternalContextBinding {
            kind: ExternalContextKind::Custom,
            reference: "https://example.com/spec".to_string(),
            required: false,
            notes: None,
        };
        assert_eq!(external.resolved_path(&workspace), None);
        assert_eq!(
            external.status_for_target(&workspace, Some("src/app.tsx")),
            ExternalContextStatus::Used
        );

        fs::write(workspace.join("src/app.tsx"), "export const App = () => null;\n").unwrap();
        thread::sleep(Duration::from_millis(20));
        fs::write(workspace.join("docs/reference.md"), "reference\n").unwrap();
        assert_eq!(relative.status_for_target(&workspace, None), ExternalContextStatus::Used);
        assert_eq!(
            relative.status_for_target(&workspace, Some("src/missing.tsx")),
            ExternalContextStatus::Used
        );
        assert_eq!(
            relative.status_for_target(&workspace, Some("src/app.tsx")),
            ExternalContextStatus::Used
        );

        thread::sleep(Duration::from_millis(20));
        fs::write(workspace.join("src/app.tsx"), "export const App = () => 'new';\n").unwrap();
        assert_eq!(
            relative.status_for_target(&workspace, Some("src/app.tsx")),
            ExternalContextStatus::Stale
        );
    }

    #[test]
    fn settings_reject_blank_standards_and_notes() {
        let settings = DomainTemplateSettings {
            enabled: Some(true),
            standards: Some("   ".to_string()),
            external_context_bindings: Vec::new(),
        };
        assert!(settings.validate().is_err());

        let binding = ExternalContextBinding {
            kind: ExternalContextKind::DesignSystem,
            reference: "ui/design-system.md".to_string(),
            required: false,
            notes: Some(" ".to_string()),
        };
        assert!(binding.validate().is_err());
    }

    #[test]
    fn domain_template_settings_and_applied_context_validate_expected_states() {
        let settings = DomainTemplateSettings {
            enabled: Some(true),
            standards: Some("Prefer bounded UI changes".to_string()),
            external_context_bindings: vec![ExternalContextBinding {
                kind: ExternalContextKind::DesignSystem,
                reference: "design/system.md".to_string(),
                required: true,
                notes: Some("Use shared spacing tokens".to_string()),
            }],
        };
        assert!(settings.validate().is_ok());

        let invalid_settings = DomainTemplateSettings {
            enabled: Some(true),
            standards: Some("valid".to_string()),
            external_context_bindings: vec![ExternalContextBinding {
                kind: ExternalContextKind::DesignReference,
                reference: "   ".to_string(),
                required: false,
                notes: None,
            }],
        };
        assert!(invalid_settings.validate().is_err());

        let credible = AppliedDomainContext {
            families: vec![DomainFamily::React, DomainFamily::WebUi],
            summary: "credible context".to_string(),
            credibility: AppliedDomainCredibility::Credible,
            selected_target: "src/App.tsx".to_string(),
            guidance_sources: vec!["workspace rules".to_string()],
            external_inputs: Vec::new(),
            governed_artifact_refs: Vec::new(),
            blocking_reason: None,
        };
        assert!(credible.validate().is_ok());

        let insufficient = AppliedDomainContext {
            families: vec![DomainFamily::WebUi],
            summary: "insufficient context".to_string(),
            credibility: AppliedDomainCredibility::Insufficient,
            selected_target: "src/App.tsx".to_string(),
            guidance_sources: Vec::new(),
            external_inputs: Vec::new(),
            governed_artifact_refs: Vec::new(),
            blocking_reason: None,
        };
        assert!(insufficient.validate().is_err());

        let stale = AppliedDomainContext {
            families: vec![DomainFamily::React],
            summary: "stale context".to_string(),
            credibility: AppliedDomainCredibility::Stale,
            selected_target: "src/App.tsx".to_string(),
            guidance_sources: Vec::new(),
            external_inputs: Vec::new(),
            governed_artifact_refs: Vec::new(),
            blocking_reason: Some("design reference is older than target".to_string()),
        };
        assert!(stale.validate().is_ok());
    }

    #[test]
    fn detect_domain_families_covers_workspace_markers_and_target_extensions() {
        let workspace = temp_workspace("domain-template-detect-workspace");
        fs::create_dir_all(workspace.join("android")).unwrap();
        fs::create_dir_all(workspace.join("ios")).unwrap();
        fs::write(workspace.join("Cargo.toml"), "[package]\nname = 'x'\nversion = '0.1.0'\n")
            .unwrap();
        fs::write(workspace.join("pom.xml"), "<project />\n").unwrap();
        fs::write(workspace.join("Directory.Build.props"), "<Project />\n").unwrap();
        fs::write(workspace.join("pyproject.toml"), "[project]\nname='x'\n").unwrap();
        fs::write(workspace.join("Gemfile"), "source 'https://rubygems.org'\n").unwrap();
        fs::write(workspace.join("composer.json"), "{}\n").unwrap();
        fs::write(workspace.join("Package.swift"), "// swift package\n").unwrap();
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"koa":"1.0.0"}}"#).unwrap();

        let workspace_families = detect_domain_families(&workspace, None);
        assert!(workspace_families.contains(&DomainFamily::Systems));
        assert!(workspace_families.contains(&DomainFamily::JvmService));
        assert!(workspace_families.contains(&DomainFamily::DotNetService));
        assert!(workspace_families.contains(&DomainFamily::PythonService));
        assert!(workspace_families.contains(&DomainFamily::Ruby));
        assert!(workspace_families.contains(&DomainFamily::Php));
        assert!(workspace_families.contains(&DomainFamily::Mobile));
        assert!(workspace_families.contains(&DomainFamily::NodeService));

        assert_eq!(
            detect_domain_families(&workspace, Some("src/lib.rs")),
            vec![DomainFamily::Systems]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("src/Main.java")),
            vec![DomainFamily::JvmService]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("android/MainActivity.kt")),
            vec![DomainFamily::Mobile]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("src/Program.cs")),
            vec![DomainFamily::DotNetService]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("src/service.py")),
            vec![DomainFamily::PythonService]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("analytics/notebook.py")),
            vec![DomainFamily::Data]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("db/report.sql")),
            vec![DomainFamily::Data]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("app/model.rb")),
            vec![DomainFamily::Ruby]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("public/index.php")),
            vec![DomainFamily::Php]
        );
        assert_eq!(
            detect_domain_families(&workspace, Some("mobile/App.swift")),
            vec![DomainFamily::Mobile]
        );
        assert_eq!(detect_domain_families(&workspace, Some("notes.txt")), workspace_families);
    }

    #[test]
    fn detect_domain_families_covers_frontend_and_backend_javascript_frameworks() {
        let react_workspace = temp_workspace("domain-template-detect-react");
        fs::write(
            react_workspace.join("package.json"),
            r#"{"dependencies":{"react":"18.0.0","express":"5.0.0"}}"#,
        )
        .unwrap();
        let react_target = detect_domain_families(&react_workspace, Some("src/components/App.tsx"));
        assert!(react_target.contains(&DomainFamily::React));
        assert!(react_target.contains(&DomainFamily::WebUi));

        let backend_target = detect_domain_families(&react_workspace, Some("src/server/api.ts"));
        assert!(backend_target.contains(&DomainFamily::NodeService));
        assert!(!backend_target.contains(&DomainFamily::WebUi));

        let vue_workspace = temp_workspace("domain-template-detect-vue");
        fs::write(vue_workspace.join("package.json"), r#"{"dependencies":{"vue":"3.0.0"}}"#)
            .unwrap();
        let vue_target = detect_domain_families(&vue_workspace, Some("frontend/client.ts"));
        assert!(vue_target.contains(&DomainFamily::Vue));
        assert!(vue_target.contains(&DomainFamily::WebUi));

        let angular_workspace = temp_workspace("domain-template-detect-angular");
        fs::write(
            angular_workspace.join("package.json"),
            r#"{"dependencies":{"angular":"18.0.0"}}"#,
        )
        .unwrap();
        let angular_target = detect_domain_families(&angular_workspace, Some("src/app/main.ts"));
        assert!(angular_target.contains(&DomainFamily::Angular));
        assert!(angular_target.contains(&DomainFamily::WebUi));
    }

    #[test]
    fn detect_domain_families_prefers_target_and_workspace_hints() {
        let workspace = temp_workspace("domain-template-detect");
        fs::create_dir_all(workspace.join("src/components")).unwrap();
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();

        let react_target = detect_domain_families(&workspace, Some("src/components/App.tsx"));
        assert!(react_target.contains(&DomainFamily::React));
        assert!(react_target.contains(&DomainFamily::WebUi));
    }

    #[test]
    fn detect_domain_families_covers_mixed_backend_and_dotnet_workspace_hints() {
        let workspace = temp_workspace("domain-template-mixed");
        fs::create_dir_all(workspace.join("src/server")).unwrap();
        fs::write(
            workspace.join("package.json"),
            r#"{"dependencies":{"react":"18.0.0","express":"5.0.0"}}"#,
        )
        .unwrap();

        let backend_target = detect_domain_families(&workspace, Some("src/server/api.ts"));
        assert!(backend_target.contains(&DomainFamily::NodeService));

        fs::write(workspace.join("workspace.sln"), "Microsoft Visual Studio Solution File\n")
            .unwrap();
        let workspace_families = detect_domain_families(&workspace, None);
        assert!(workspace_families.contains(&DomainFamily::DotNetService));
    }

    #[test]
    fn binding_status_detects_missing_and_stale_file_inputs() {
        let workspace = temp_workspace("domain-template-binding");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("design")).unwrap();
        fs::write(workspace.join("src/app.tsx"), "export const App = () => null;\n").unwrap();

        let missing = ExternalContextBinding {
            kind: ExternalContextKind::DesignReference,
            reference: "design/missing.fig".to_string(),
            required: true,
            notes: None,
        };
        assert_eq!(
            missing.status_for_target(&workspace, Some("src/app.tsx")),
            ExternalContextStatus::Unavailable
        );

        fs::write(workspace.join("design/reference.md"), "button spec\n").unwrap();
        let stale = ExternalContextBinding {
            kind: ExternalContextKind::DesignReference,
            reference: "design/reference.md".to_string(),
            required: true,
            notes: None,
        };
        assert!(matches!(
            stale.status_for_target(&workspace, Some("src/app.tsx")),
            ExternalContextStatus::Used | ExternalContextStatus::Stale
        ));
    }

    #[test]
    fn applied_domain_context_requires_family_when_credible() {
        let context = AppliedDomainContext {
            families: Vec::new(),
            summary: "credible context".to_string(),
            credibility: AppliedDomainCredibility::Credible,
            selected_target: "src/lib.rs".to_string(),
            guidance_sources: Vec::new(),
            external_inputs: Vec::new(),
            governed_artifact_refs: Vec::new(),
            blocking_reason: None,
        };
        assert!(context.validate().is_err());
    }
}
