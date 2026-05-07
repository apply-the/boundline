use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::domain::domain_templates::DomainFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HygieneFilePlan {
    pub path: &'static str,
    pub packs: Vec<HygienePatternPack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HygienePatternPack {
    pub provenance: String,
    pub patterns: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HygieneMergeResult {
    pub content: String,
    pub added_patterns: Vec<String>,
    pub preserved_custom_lines: usize,
}

pub fn plan_hygiene_defaults(
    workspace: &Path,
    domains: &BTreeSet<DomainFamily>,
) -> Vec<HygieneFilePlan> {
    let mut plans = Vec::new();

    if is_git_workspace(workspace) {
        let mut packs = vec![HygienePatternPack {
            provenance: "universal".to_string(),
            patterns: vec![".boundline/traces/", ".boundline/checkpoints/"],
        }];
        packs.extend(domain_gitignore_packs(domains));
        packs.extend(tool_gitignore_packs(workspace));
        plans.push(HygieneFilePlan { path: ".gitignore", packs });
    }

    if has_docker_cues(workspace) {
        let mut patterns = vec![".git", ".boundline/traces/", ".boundline/checkpoints/"];
        if has_node_family(domains) || workspace.join("package.json").is_file() {
            patterns.push("node_modules/");
            patterns.push("dist/");
        }
        if domains.contains(&DomainFamily::PythonService)
            || workspace.join("pyproject.toml").is_file()
        {
            patterns.push("__pycache__/");
            patterns.push(".venv/");
        }
        plans.push(HygieneFilePlan {
            path: ".dockerignore",
            packs: vec![HygienePatternPack { provenance: "tool:docker".to_string(), patterns }],
        });
    }

    if has_prettier_cues(workspace) {
        plans.push(HygieneFilePlan {
            path: ".prettierignore",
            packs: vec![HygienePatternPack {
                provenance: "tool:prettier".to_string(),
                patterns: vec![".boundline/traces/", ".boundline/checkpoints/", "dist/", "build/"],
            }],
        });
    }

    if has_legacy_eslint_ignore_cues(workspace) {
        plans.push(HygieneFilePlan {
            path: ".eslintignore",
            packs: vec![HygienePatternPack {
                provenance: "tool:eslint".to_string(),
                patterns: vec![".boundline/traces/", ".boundline/checkpoints/", "dist/", "build/"],
            }],
        });
    }

    if has_terraform_cues(workspace) {
        plans.push(HygieneFilePlan {
            path: ".terraformignore",
            packs: vec![HygienePatternPack {
                provenance: "tool:terraform".to_string(),
                patterns: vec![".terraform/", "*.tfstate", "*.tfstate.*"],
            }],
        });
    }

    if has_helm_cues(workspace) {
        plans.push(HygieneFilePlan {
            path: ".helmignore",
            packs: vec![HygienePatternPack {
                provenance: "tool:helm".to_string(),
                patterns: vec![".boundline/traces/", ".boundline/checkpoints/", "*.tgz"],
            }],
        });
    }

    plans
}

pub fn merge_hygiene_content(existing: Option<&str>, plan: &HygieneFilePlan) -> HygieneMergeResult {
    let existing = existing.unwrap_or_default();
    let mut lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    let mut normalized = lines
        .iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<BTreeSet<_>>();
    let preserved_custom_lines = normalized.len();
    let mut added_patterns = Vec::new();

    if !lines.is_empty() && lines.last().is_some_and(|line| !line.trim().is_empty()) {
        lines.push(String::new());
    }

    for pack in &plan.packs {
        let mut pack_added = Vec::new();
        for pattern in &pack.patterns {
            if normalized.insert((*pattern).to_string()) {
                pack_added.push(*pattern);
                added_patterns.push((*pattern).to_string());
            }
        }
        if !pack_added.is_empty() {
            lines.push(format!("# Boundline {} defaults", pack.provenance));
            lines.extend(pack_added.into_iter().map(str::to_string));
        }
    }

    let mut content = lines.join("\n");
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    HygieneMergeResult { content, added_patterns, preserved_custom_lines }
}

fn domain_gitignore_packs(domains: &BTreeSet<DomainFamily>) -> Vec<HygienePatternPack> {
    let mut packs = Vec::new();

    if domains.contains(&DomainFamily::Systems) {
        packs.push(HygienePatternPack {
            provenance: "domain:systems".to_string(),
            patterns: vec!["target/"],
        });
    }
    if domains.contains(&DomainFamily::PythonService) {
        packs.push(HygienePatternPack {
            provenance: "domain:python_service".to_string(),
            patterns: vec!["__pycache__/", "*.py[cod]", ".pytest_cache/", ".venv/"],
        });
    }
    if has_node_family(domains) {
        packs.push(HygienePatternPack {
            provenance: "domain:node_web".to_string(),
            patterns: vec!["node_modules/", "dist/", "build/", "coverage/"],
        });
    }
    if domains.contains(&DomainFamily::JvmService) {
        packs.push(HygienePatternPack {
            provenance: "domain:jvm_service".to_string(),
            patterns: vec!["target/", "build/", ".gradle/"],
        });
    }
    if domains.contains(&DomainFamily::DotNetService) {
        packs.push(HygienePatternPack {
            provenance: "domain:dotnet_service".to_string(),
            patterns: vec!["bin/", "obj/"],
        });
    }
    if domains.contains(&DomainFamily::Ruby) {
        packs.push(HygienePatternPack {
            provenance: "domain:ruby".to_string(),
            patterns: vec![".bundle/", "vendor/bundle/"],
        });
    }
    if domains.contains(&DomainFamily::Php) {
        packs.push(HygienePatternPack {
            provenance: "domain:php".to_string(),
            patterns: vec!["vendor/"],
        });
    }
    if domains.contains(&DomainFamily::Mobile) {
        packs.push(HygienePatternPack {
            provenance: "domain:mobile".to_string(),
            patterns: vec!["DerivedData/", ".gradle/", "build/"],
        });
    }
    if domains.contains(&DomainFamily::Data) {
        packs.push(HygienePatternPack {
            provenance: "domain:data".to_string(),
            patterns: vec![".ipynb_checkpoints/", "data/tmp/"],
        });
    }

    packs
}

fn tool_gitignore_packs(workspace: &Path) -> Vec<HygienePatternPack> {
    let mut packs = Vec::new();

    if has_kubernetes_cues(workspace) {
        packs.push(HygienePatternPack {
            provenance: "tool:kubernetes".to_string(),
            patterns: vec![".kube/", "*.secret.yaml", "*.secret.yml"],
        });
    }

    packs
}

fn has_node_family(domains: &BTreeSet<DomainFamily>) -> bool {
    domains.contains(&DomainFamily::NodeService)
        || domains.contains(&DomainFamily::WebUi)
        || domains.contains(&DomainFamily::React)
        || domains.contains(&DomainFamily::Vue)
        || domains.contains(&DomainFamily::Angular)
}

fn is_git_workspace(workspace: &Path) -> bool {
    workspace.join(".git").exists() || workspace.join(".gitignore").is_file()
}

fn has_docker_cues(workspace: &Path) -> bool {
    workspace.join("Dockerfile").is_file()
        || workspace.join(".dockerignore").is_file()
        || workspace.join("docker-compose.yml").is_file()
        || workspace.join("docker-compose.yaml").is_file()
        || workspace.join("compose.yml").is_file()
        || workspace.join("compose.yaml").is_file()
}

fn has_prettier_cues(workspace: &Path) -> bool {
    workspace.join(".prettierrc").is_file()
        || workspace.join(".prettierrc.json").is_file()
        || workspace.join("prettier.config.js").is_file()
        || workspace.join(".prettierignore").is_file()
}

fn has_legacy_eslint_ignore_cues(workspace: &Path) -> bool {
    workspace.join(".eslintrc").is_file()
        || workspace.join(".eslintrc.json").is_file()
        || workspace.join(".eslintignore").is_file()
}

fn has_terraform_cues(workspace: &Path) -> bool {
    workspace.join(".terraformignore").is_file() || contains_top_level_extension(workspace, "tf")
}

fn has_helm_cues(workspace: &Path) -> bool {
    workspace.join("Chart.yaml").is_file()
        || workspace.join(".helmignore").is_file()
        || workspace.join("charts").is_dir()
}

fn has_kubernetes_cues(workspace: &Path) -> bool {
    workspace.join("kustomization.yaml").is_file()
        || workspace.join("kustomization.yml").is_file()
        || workspace.join("k8s").is_dir()
        || workspace.join("kubernetes").is_dir()
}

fn contains_top_level_extension(workspace: &Path, extension: &str) -> bool {
    fs::read_dir(workspace).ok().into_iter().flatten().filter_map(Result::ok).any(|entry| {
        entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(extension))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;

    use uuid::Uuid;

    use super::{
        DomainFamily, HygieneFilePlan, HygienePatternPack, merge_hygiene_content,
        plan_hygiene_defaults,
    };

    #[test]
    fn plans_universal_and_selected_domain_gitignore_defaults() {
        let workspace = std::env::temp_dir().join(format!("boundline-hygiene-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".git")).unwrap();

        let domains = [DomainFamily::React].into_iter().collect();
        let plans = plan_hygiene_defaults(&workspace, &domains);
        let gitignore = plans.iter().find(|plan| plan.path == ".gitignore").unwrap();

        assert!(gitignore.packs.iter().any(|pack| pack.provenance == "universal"));
        assert!(gitignore.packs.iter().any(|pack| pack.provenance == "domain:node_web"));
    }

    #[test]
    fn plans_legacy_eslintignore_defaults_when_legacy_cues_are_present() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-eslint-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join(".eslintrc.json"), "{}\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());
        let eslintignore = plans.iter().find(|plan| plan.path == ".eslintignore").unwrap();

        assert!(eslintignore.packs.iter().any(|pack| pack.provenance == "tool:eslint"));
    }

    #[test]
    fn skips_eslintignore_for_flat_config_without_legacy_ignore_usage() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-eslint-flat-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("eslint.config.js"), "export default [];\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());

        assert!(plans.iter().all(|plan| plan.path != ".eslintignore"));
    }

    #[test]
    fn plans_kubernetes_gitignore_defaults_when_kustomize_cues_are_present() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-kubernetes-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".git")).unwrap();
        fs::write(workspace.join("kustomization.yaml"), "resources: []\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());
        let gitignore = plans.iter().find(|plan| plan.path == ".gitignore").unwrap();
        let kubernetes =
            gitignore.packs.iter().find(|pack| pack.provenance == "tool:kubernetes").unwrap();

        assert!(kubernetes.patterns.contains(&".kube/"));
        assert!(kubernetes.patterns.contains(&"*.secret.yaml"));
    }

    #[test]
    fn merge_preserves_existing_lines_and_adds_missing_patterns_once() {
        let plan = HygieneFilePlan {
            path: ".gitignore",
            packs: vec![HygienePatternPack {
                provenance: "universal".to_string(),
                patterns: vec![".boundline/traces/", "node_modules/"],
            }],
        };

        let merged = merge_hygiene_content(Some("node_modules/\ncustom/\n"), &plan);

        assert!(merged.content.contains("custom/"));
        assert_eq!(merged.content.matches("node_modules/").count(), 1);
        assert!(merged.content.contains(".boundline/traces/"));
        assert_eq!(merged.added_patterns, vec![".boundline/traces/"]);
    }

    #[test]
    fn plans_docker_defaults_with_node_and_python_cues() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-docker-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("Dockerfile"), "FROM python:3.12\n").unwrap();
        fs::write(workspace.join("package.json"), "{}\n").unwrap();
        fs::write(workspace.join("pyproject.toml"), "[project]\nname=\"x\"\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());
        let dockerignore = plans.iter().find(|plan| plan.path == ".dockerignore").unwrap();
        let pack = dockerignore.packs.iter().find(|p| p.provenance == "tool:docker").unwrap();

        assert!(pack.patterns.contains(&"node_modules/"));
        assert!(pack.patterns.contains(&"__pycache__/"));
        assert!(pack.patterns.contains(&".venv/"));
    }

    #[test]
    fn plans_prettier_defaults_when_prettierrc_is_present() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-prettier-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join(".prettierrc"), "{}\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());
        let prettierignore = plans.iter().find(|plan| plan.path == ".prettierignore").unwrap();
        let pack = prettierignore.packs.iter().find(|p| p.provenance == "tool:prettier").unwrap();

        assert!(pack.patterns.contains(&"dist/"));
        assert!(pack.patterns.contains(&"build/"));
    }

    #[test]
    fn plans_terraform_defaults_when_tf_files_are_present() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-terraform-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("main.tf"), "# terraform\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());
        let terraformignore = plans.iter().find(|plan| plan.path == ".terraformignore").unwrap();
        let pack = terraformignore.packs.iter().find(|p| p.provenance == "tool:terraform").unwrap();

        assert!(pack.patterns.contains(&".terraform/"));
        assert!(pack.patterns.contains(&"*.tfstate"));
    }

    #[test]
    fn plans_helm_defaults_when_chart_yaml_is_present() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-helm-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("Chart.yaml"), "apiVersion: v2\nname: myapp\n").unwrap();

        let plans = plan_hygiene_defaults(&workspace, &BTreeSet::new());
        let helmignore = plans.iter().find(|plan| plan.path == ".helmignore").unwrap();
        let pack = helmignore.packs.iter().find(|p| p.provenance == "tool:helm").unwrap();

        assert!(pack.patterns.contains(&"*.tgz"));
    }

    #[test]
    fn plans_remaining_domain_gitignore_packs() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-hygiene-domains-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".git")).unwrap();

        let domains = [
            DomainFamily::Systems,
            DomainFamily::PythonService,
            DomainFamily::JvmService,
            DomainFamily::DotNetService,
            DomainFamily::Ruby,
            DomainFamily::Php,
            DomainFamily::Mobile,
            DomainFamily::Data,
        ]
        .into_iter()
        .collect();

        let plans = plan_hygiene_defaults(&workspace, &domains);
        let gitignore = plans.iter().find(|plan| plan.path == ".gitignore").unwrap();

        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:systems"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:python_service"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:jvm_service"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:dotnet_service"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:ruby"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:php"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:mobile"));
        assert!(gitignore.packs.iter().any(|p| p.provenance == "domain:data"));

        let jvm = gitignore.packs.iter().find(|p| p.provenance == "domain:jvm_service").unwrap();
        assert!(jvm.patterns.contains(&".gradle/"));
        let dotnet =
            gitignore.packs.iter().find(|p| p.provenance == "domain:dotnet_service").unwrap();
        assert!(dotnet.patterns.contains(&"obj/"));
        let ruby = gitignore.packs.iter().find(|p| p.provenance == "domain:ruby").unwrap();
        assert!(ruby.patterns.contains(&".bundle/"));
        let data = gitignore.packs.iter().find(|p| p.provenance == "domain:data").unwrap();
        assert!(data.patterns.contains(&".ipynb_checkpoints/"));
    }
}
