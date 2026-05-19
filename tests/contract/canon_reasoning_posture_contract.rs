use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::time::{SystemTime, UNIX_EPOCH};

const BOUNDLINE_MANIFEST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
const CANON_MANIFEST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../canon/Cargo.toml");
const CANON_PROVIDER_CONTRACT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../canon/docs/integration/governed-reasoning-posture-contract.md"
);
const CANON_PROVIDER_CONTRACT_SNAPSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/061-reasoning-profile-contracts/contracts/canon-governed-reasoning-posture-contract.snapshot.md"
);
const VERSION_ALIGNMENT_BRIEF_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/061-reasoning-profile-contracts/contracts/reasoning-version-alignment-contract.md"
);
const SUPPORTED_BOUNDLINE_VERSION: &str = "0.63.0";
const SUPPORTED_BOUNDLINE_WINDOW: &str = "0.63.x";
const SUPPORTED_CANON_VERSION: &str = "0.59.0";
const SUPPORTED_CANON_WINDOW: &str = "0.59.x";
const SUPPORTED_CONTRACT_LINE: &str = "governed_reasoning_posture_v1";

fn read_text(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

fn read_text_with_fallback(
    primary_path: &str,
    fallback_path: &str,
) -> Result<String, Box<dyn Error>> {
    match fs::read_to_string(primary_path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(fs::read_to_string(fallback_path)?),
        Err(error) => Err(Box::new(error)),
    }
}

fn assert_contains(document: &str, expected: &str, context: &str) {
    assert!(document.contains(expected), "{context}: expected to find `{expected}`");
}

#[test]
fn reasoning_version_alignment_brief_declares_supported_release_pair() -> Result<(), Box<dyn Error>>
{
    let brief = read_text(VERSION_ALIGNMENT_BRIEF_PATH)?;

    assert_contains(
        &brief,
        SUPPORTED_CONTRACT_LINE,
        "reasoning alignment brief should declare the shared contract line",
    );
    assert_contains(
        &brief,
        SUPPORTED_BOUNDLINE_WINDOW,
        "reasoning alignment brief should declare the supported Boundline window",
    );
    assert_contains(
        &brief,
        SUPPORTED_CANON_WINDOW,
        "reasoning alignment brief should declare the supported Canon window",
    );

    Ok(())
}

#[test]
fn reasoning_version_alignment_brief_matches_workspace_versions() -> Result<(), Box<dyn Error>> {
    let boundline_manifest = read_text(BOUNDLINE_MANIFEST_PATH)?;
    let boundline_version_entry = format!("version = \"{SUPPORTED_BOUNDLINE_VERSION}\"");
    let canon_version_entry = format!("version = \"{SUPPORTED_CANON_VERSION}\"");
    let canon_snapshot_min_entry = format!("canon_min = \"{SUPPORTED_CANON_VERSION}\"");

    assert_contains(
        &boundline_manifest,
        boundline_version_entry.as_str(),
        "Boundline manifest should carry the planned workspace version",
    );

    match fs::read_to_string(CANON_MANIFEST_PATH) {
        Ok(canon_manifest) => assert_contains(
            &canon_manifest,
            canon_version_entry.as_str(),
            "Canon manifest should carry the planned workspace version",
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let canon_snapshot = read_text(CANON_PROVIDER_CONTRACT_SNAPSHOT_PATH)?;
            assert_contains(
                &canon_snapshot,
                canon_snapshot_min_entry.as_str(),
                "Canon snapshot should carry the planned workspace version",
            );
        }
        Err(error) => return Err(Box::new(error)),
    }

    Ok(())
}

#[test]
fn canon_reasoning_posture_contract_publishes_supported_line_and_window()
-> Result<(), Box<dyn Error>> {
    let contract = read_text_with_fallback(
        CANON_PROVIDER_CONTRACT_PATH,
        CANON_PROVIDER_CONTRACT_SNAPSHOT_PATH,
    )?;

    assert_contains(
        &contract,
        SUPPORTED_CONTRACT_LINE,
        "Canon provider contract should publish the supported contract line",
    );
    assert_contains(
        &contract,
        SUPPORTED_BOUNDLINE_WINDOW,
        "Canon provider contract should publish the supported Boundline window",
    );
    assert_contains(
        &contract,
        SUPPORTED_CANON_WINDOW,
        "Canon provider contract should publish the supported Canon window",
    );

    Ok(())
}

#[test]
fn canon_reasoning_posture_contract_snapshot_preserves_supported_line_and_window()
-> Result<(), Box<dyn Error>> {
    let contract = read_text(CANON_PROVIDER_CONTRACT_SNAPSHOT_PATH)?;

    assert_contains(
        &contract,
        SUPPORTED_CONTRACT_LINE,
        "Canon provider contract snapshot should preserve the supported contract line",
    );
    assert_contains(
        &contract,
        SUPPORTED_BOUNDLINE_WINDOW,
        "Canon provider contract snapshot should preserve the supported Boundline window",
    );
    assert_contains(
        &contract,
        SUPPORTED_CANON_WINDOW,
        "Canon provider contract snapshot should preserve the supported Canon window",
    );

    Ok(())
}

#[test]
fn read_text_with_fallback_uses_snapshot_when_primary_is_missing() -> Result<(), Box<dyn Error>> {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!("boundline-reasoning-contract-fallback-{unique}"));
    fs::create_dir_all(&root)?;

    let missing_primary = root.join("missing.md");
    let fallback = root.join("snapshot.md");
    fs::write(
        &fallback,
        format!(
            "contract_line = \"{SUPPORTED_CONTRACT_LINE}\"\nsupported_boundline_window = \"{SUPPORTED_BOUNDLINE_WINDOW}\"\nsupported_canon_window = \"{SUPPORTED_CANON_WINDOW}\"\n"
        ),
    )?;

    let text = read_text_with_fallback(
        missing_primary.to_string_lossy().as_ref(),
        fallback.to_string_lossy().as_ref(),
    )?;

    fs::remove_dir_all(&root)?;

    assert_contains(
        &text,
        SUPPORTED_CONTRACT_LINE,
        "fallback helper should return snapshot content when the primary path is missing",
    );

    Ok(())
}
