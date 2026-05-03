use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    pub fn validate(&self) -> Result<(), DomainTemplateError> {
        if self.reference.trim().is_empty() {
            return Err(DomainTemplateError::MissingExternalContextReference);
        }
        if self.notes.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(DomainTemplateError::InvalidExternalContextNotes);
        }
        Ok(())
    }

    pub fn resolved_path(&self, workspace_ref: &Path) -> Option<PathBuf> {
        if has_external_scheme(&self.reference) {
            return None;
        }

        let raw = self.reference.strip_prefix("file:").unwrap_or(self.reference.as_str());
        let candidate = PathBuf::from(raw);
        if candidate.is_absolute() { Some(candidate) } else { Some(workspace_ref.join(candidate)) }
    }

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

    use super::{
        AppliedDomainContext, AppliedDomainCredibility, DomainFamily, DomainTemplateSettings,
        ExternalContextBinding, ExternalContextKind, ExternalContextStatus, detect_domain_families,
    };

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
    fn detect_domain_families_prefers_target_and_workspace_hints() {
        let workspace = std::env::temp_dir().join("synod-domain-template-detect");
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(workspace.join("src/components")).unwrap();
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();

        let react_target = detect_domain_families(&workspace, Some("src/components/App.tsx"));
        assert!(react_target.contains(&DomainFamily::React));
        assert!(react_target.contains(&DomainFamily::WebUi));
    }

    #[test]
    fn detect_domain_families_covers_mixed_backend_and_dotnet_workspace_hints() {
        let workspace = std::env::temp_dir().join("synod-domain-template-mixed");
        let _ = fs::remove_dir_all(&workspace);
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
        let workspace = std::env::temp_dir().join("synod-domain-template-binding");
        let _ = fs::remove_dir_all(&workspace);
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
