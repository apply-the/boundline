use serde::{Deserialize, Serialize};

use crate::domain::configuration::InitTemplate;

pub const SCAFFOLD_MANIFEST_FILE_NAME: &str = "scaffold-manifest.json";
pub const SCAFFOLD_MANIFEST_VERSION: u32 = 1;
pub const INITIAL_SCAFFOLD_EPOCH: u32 = 1;

const FNV_OFFSET_BASIS_64: u64 = 0xcbf29ce484222325;
const FNV_PRIME_64: u64 = 0x100000001b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldTarget {
    Config,
    Execution,
    Assistant,
    Docs,
    Hygiene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldOwnershipMode {
    Replace,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScaffoldManifestEntry {
    pub path: String,
    pub target: ScaffoldTarget,
    pub ownership: ScaffoldOwnershipMode,
    pub fingerprint: String,
}

impl ScaffoldManifestEntry {
    pub fn new(
        path: impl Into<String>,
        target: ScaffoldTarget,
        ownership: ScaffoldOwnershipMode,
        rendered_contents: &str,
    ) -> Self {
        Self {
            path: path.into(),
            target,
            ownership,
            fingerprint: fingerprint_text(rendered_contents),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldManifest {
    pub version: u32,
    pub scaffold_epoch: u32,
    pub boundline_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_template: Option<InitTemplate>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<ScaffoldManifestEntry>,
}

impl ScaffoldManifest {
    pub fn new(
        boundline_version: impl Into<String>,
        workspace_template: Option<InitTemplate>,
        created_at_ms: u64,
        updated_at_ms: u64,
        mut entries: Vec<ScaffoldManifestEntry>,
    ) -> Self {
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        entries.dedup_by(|left, right| left.path == right.path);

        Self {
            version: SCAFFOLD_MANIFEST_VERSION,
            scaffold_epoch: INITIAL_SCAFFOLD_EPOCH,
            boundline_version: boundline_version.into(),
            workspace_template,
            created_at_ms,
            updated_at_ms,
            entries,
        }
    }

    pub fn tracks_same_state(&self, other: &Self) -> bool {
        self.version == other.version
            && self.scaffold_epoch == other.scaffold_epoch
            && self.boundline_version == other.boundline_version
            && self.workspace_template == other.workspace_template
            && self.entries == other.entries
    }
}

pub fn fingerprint_text(rendered_contents: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS_64;
    for byte in rendered_contents.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{
        ScaffoldManifest, ScaffoldManifestEntry, ScaffoldOwnershipMode, ScaffoldTarget,
        fingerprint_text,
    };
    use crate::domain::configuration::InitTemplate;

    #[test]
    fn scaffold_manifest_entry_fingerprint_is_stable() {
        let left = ScaffoldManifestEntry::new(
            ".boundline/config.toml",
            ScaffoldTarget::Config,
            ScaffoldOwnershipMode::Replace,
            "version = 1\n",
        );
        let right = ScaffoldManifestEntry::new(
            ".boundline/config.toml",
            ScaffoldTarget::Config,
            ScaffoldOwnershipMode::Replace,
            "version = 1\n",
        );

        assert_eq!(left.fingerprint, right.fingerprint);
        assert_eq!(left.fingerprint, fingerprint_text("version = 1\n"));
    }

    #[test]
    fn scaffold_manifest_sorts_entries_by_path() {
        let manifest = ScaffoldManifest::new(
            "0.64.0",
            Some(InitTemplate::Change),
            10,
            20,
            vec![
                ScaffoldManifestEntry::new(
                    "assistant/README.md",
                    ScaffoldTarget::Assistant,
                    ScaffoldOwnershipMode::Replace,
                    "shared",
                ),
                ScaffoldManifestEntry::new(
                    ".boundline/config.toml",
                    ScaffoldTarget::Config,
                    ScaffoldOwnershipMode::Replace,
                    "config",
                ),
            ],
        );

        assert_eq!(manifest.entries[0].path, ".boundline/config.toml");
        assert_eq!(manifest.entries[1].path, "assistant/README.md");
    }
}
