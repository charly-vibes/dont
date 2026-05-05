use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_in(dir: &TempDir) -> assert_cmd::assert::Assert {
    dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
}

// --- Successful init ---

#[test]
fn init_creates_dont_directory_with_required_entries() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    assert!(dir.path().join("db.cozo").exists(), "db.cozo");
    assert!(dir.path().join("config.toml").exists(), "config.toml");
    assert!(dir.path().join("AGENTS.md").exists(), "AGENTS.md");
    assert!(dir.path().join("seed").is_dir(), "seed/");
    assert!(dir.path().join("vocab").is_dir(), "vocab/");
    assert!(dir.path().join("rules").is_dir(), "rules/");
    assert!(dir.path().join("imports").is_dir(), "imports/");
    assert!(dir.path().join("sessions").is_dir(), "sessions/");
    assert!(dir.path().join("schemas").is_dir(), "schemas/");
}

#[test]
fn init_config_toml_has_required_sections() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let config = fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(config.contains("[project]"), "[project] section");
    assert!(config.contains("[output]"), "[output] section");
    assert!(config.contains("[trust.hedges]"), "[trust.hedges] section");
    assert!(config.contains("[storage]"), "[storage] section");
}

#[test]
fn init_config_toml_defaults_to_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let config = fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(config.contains("mode = \"permissive\""));
}

#[test]
fn init_outputs_success_envelope() {
    let dir = TempDir::new().unwrap();
    let output = init_in(&dir).success().get_output().stdout.clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_version"], "0.2");
    assert!(v["cli_version"].is_string());
    assert!(v["meta"].is_object());
}

// --- Already-initialized ---

#[test]
fn reinit_returns_already_initialised_error_and_exits_3() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let output = dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "already-initialised");
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty(), "remediation must be non-empty");
}

// --- Parent-directory discovery ---

#[test]
fn project_discovery_finds_dont_dir_in_parent() {
    let base = TempDir::new().unwrap();
    // Init the base (DONT_DIR points to .dont inside base)
    let dont_dir = base.path().join(".dont");
    dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", &dont_dir)
        .assert()
        .success();

    // Running conclude from a subdirectory should discover .dont/ by walking up
    let subdir = base.path().join("sub/nested");
    fs::create_dir_all(&subdir).unwrap();

    dont()
        .arg("conclude")
        .arg("test claim")
        .arg("--json")
        .current_dir(&subdir)
        // No DONT_DIR override — must discover via parent walk
        .assert()
        .success();
}

// --- Config-missing ---

#[test]
fn conclude_outside_project_returns_config_missing_and_exits_3() {
    let dir = TempDir::new().unwrap();
    // No init — no .dont/ exists

    let output = dont()
        .arg("conclude")
        .arg("a claim")
        .arg("--json")
        .env("DONT_DIR", dir.path().join("nonexistent"))
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "config-missing");
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty());
}
