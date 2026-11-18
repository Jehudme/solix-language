//! Integration tests for `solix wiki` offline documentation portal resolution and CLI behavior.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn solix_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_solix"))
}

fn create_temp_workspace(prefix: &str) -> PathBuf {
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temp_dir = std::env::temp_dir().join(format!(
        "solix_wiki_test_{prefix}_{}_{unique_id}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);
    assert!(fs::create_dir_all(&temp_dir).is_ok());
    temp_dir
}

#[test]
fn test_wiki_with_custom_dir_no_open_search_and_export() {
    let temp_root = create_temp_workspace("custom_dir");
    let custom_wiki_dir = temp_root.join("custom_site");
    let assets_dir = custom_wiki_dir.join("assets");
    assert!(fs::create_dir_all(&assets_dir).is_ok());

    assert!(
        fs::write(
            custom_wiki_dir.join("index.html"),
            "<!DOCTYPE html><html><body>Solix Custom Offline Docs</body></html>"
        )
        .is_ok()
    );
    assert!(
        fs::write(
            assets_dir.join("search_index.js"),
            "window.SOLIX_DOCS_INDEX = [{\"title\":\"Index\",\"path\":\"index.html\"}];"
        )
        .is_ok()
    );

    let path_output = Command::new(solix_bin())
        .args(["wiki", "--no-open", "--path"])
        .env("SOLIX_WIKI_DIR", &custom_wiki_dir)
        .env("SOLIX_HOME", temp_root.join("solix_home"))
        .output();
    assert!(path_output.is_ok());
    let path_result = path_output.expect("command output should succeed");
    assert!(
        path_result.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&path_result.stderr)
    );
    let printed_path = String::from_utf8_lossy(&path_result.stdout)
        .trim()
        .to_string();
    assert!(
        printed_path.ends_with("index.html"),
        "expected printed path to end with index.html, got: {printed_path}"
    );

    let export_dir = temp_root.join("exported");
    let search_output = Command::new(solix_bin())
        .args(["wiki", "weak refs", "--no-open", "--output"])
        .arg(&export_dir)
        .env("SOLIX_WIKI_DIR", &custom_wiki_dir)
        .env("SOLIX_HOME", temp_root.join("solix_home"))
        .output();
    assert!(search_output.is_ok());
    let search_result = search_output.expect("export command should succeed");
    assert!(
        search_result.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&search_result.stderr)
    );
    let stdout_text = String::from_utf8_lossy(&search_result.stdout);
    assert!(
        stdout_text.contains("q=weak%20refs"),
        "expected encoded search query URI in stdout, got: {stdout_text}"
    );
    assert!(export_dir.join("index.html").is_file());
    assert!(export_dir.join("assets").join("search_index.js").is_file());

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_wiki_embedded_materialization_when_staged() {
    let staged_index = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("docs")
        .join("index.html");
    if staged_index.is_file() {
        assert!(
            solix_toolchain::assets::has_embedded_wiki_docs(),
            "assets/docs/index.html is staged on disk, so EMBEDDED_WIKI_DOCS must not be empty"
        );
    }

    if !solix_toolchain::assets::has_embedded_wiki_docs() {
        return;
    }

    let temp_root = create_temp_workspace("embedded");
    let isolated_home = temp_root.join("isolated_home");
    let output = Command::new(solix_bin())
        .args(["wiki", "--no-open", "--path"])
        .current_dir(&temp_root)
        .env("SOLIX_HOME", &isolated_home)
        .env("SOLIX_FORCE_EMBEDDED_ASSETS", "1")
        .env_remove("SOLIX_WIKI_DIR")
        .output();
    assert!(output.is_ok());
    let result = output.expect("embedded wiki command should succeed");
    assert!(
        result.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let printed_path = PathBuf::from(String::from_utf8_lossy(&result.stdout).trim());
    assert!(
        printed_path.is_file(),
        "expected materialized index.html at {}",
        printed_path.display()
    );
    let expected_version_dir = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert!(
        printed_path
            .components()
            .any(|component| component.as_os_str() == expected_version_dir.as_str()),
        "expected versioned cache folder '{expected_version_dir}' in {}",
        printed_path.display()
    );

    let materialized_dir = printed_path.parent().expect("index.html has parent");
    assert!(
        materialized_dir
            .join("assets")
            .join("search_index.js")
            .is_file()
    );
    assert!(materialized_dir.join("assets").join("style.css").is_file());
    assert!(materialized_dir.join("assets").join("app.js").is_file());
    assert!(
        !materialized_dir.join("internals").exists(),
        "internal governance/milestone docs must never be bundled in end-user solix wiki"
    );

    let _ = fs::remove_dir_all(&temp_root);
}
