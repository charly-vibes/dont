mod common;

use common::dont;
use serde_json::Value;
use std::io::Write;
use tempfile::TempDir;

fn switch_mode(dir: &TempDir, from: &str, to: &str) {
    let config_path = dir.path().join("config.toml");
    let original = std::fs::read_to_string(&config_path)
        .expect("config.toml must exist before switching mode");
    let updated = original.replace(
        &format!("mode = \"{from}\""),
        &format!("mode = \"{to}\""),
    );
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&config_path)
        .expect("config.toml must be writable");
    f.write_all(updated.as_bytes())
        .expect("write to config.toml must succeed");
}

fn init_permissive(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn init_strict(dir: &TempDir) {
    dont()
        .args(["init", "--strict", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn define_term(dir: &TempDir, curie: &str) {
    dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn conclude_with_resolved_depends_on_succeeds_in_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);

    let defined = dont()
        .args(["define", "WB:P001", "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let defined_v: Value = serde_json::from_slice(&defined).unwrap();
    let term_id = defined_v["data"]["id"].as_str().unwrap().to_string();

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
    let deps = v["data"]["depends_on"].as_array().unwrap();
    assert_eq!(deps, &vec![Value::String(term_id)]);
    assert!(v["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn conclude_with_unresolved_depends_on_succeeds_in_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
    let warnings = v["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w["rule_name"] == "unresolved-term-ref"),
        "expected unresolved-term-ref warning, got: {:?}", warnings
    );
}

#[test]
fn conclude_with_unresolved_depends_on_is_refused_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "unresolved-term-ref");
}

#[test]
fn conclude_with_resolved_depends_on_succeeds_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);
    define_term(&dir, "WB:P001");

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn conclude_without_depends_on_succeeds_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);

    let output = dont()
        .args(["conclude", "A self-contained claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn conclude_multiple_depends_on_one_unresolved_is_refused_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);
    define_term(&dir, "WB:P001");

    let output = dont()
        .args([
            "conclude", "Uses both terms",
            "--depends-on", "WB:P001",
            "--depends-on", "WB:P002",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "unresolved-term-ref");
}

#[test]
fn conclude_with_existing_term_id_depends_on_emits_no_unresolved_warning() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);

    let defined = dont()
        .args(["define", "WB:P001", "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let defined_v: Value = serde_json::from_slice(&defined).unwrap();
    let term_id = defined_v["data"]["id"].as_str().unwrap().to_string();

    let output = dont()
        .args(["conclude", "Uses an existing term id", "--depends-on", &term_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["warnings"].as_array().unwrap().is_empty());
    let deps = v["data"]["depends_on"].as_array().unwrap();
    assert_eq!(deps, &vec![Value::String(term_id)]);
}

#[test]
fn ungrounded_rule_reports_warn_severity_in_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);

    // Create a claim with an unresolved CURIE dep (succeeds in permissive)
    dont()
        .args(["conclude", "A claim with unresolved dep", "--depends-on", "NS:UNRESOLVED", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["rules", "test", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["severity"], "warn",
        "ungrounded rule should report warn severity in permissive mode");
    assert!(!v["data"]["matches"].as_array().unwrap().is_empty(),
        "should have at least one match for the unresolved CURIE dep");
}

#[test]
fn ungrounded_rule_reports_strict_severity_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);

    // In strict mode, conclude with unresolved dep is refused via pre-validation.
    // Bypass by creating a permissive project first, creating the claim, then
    // switching config to strict to simulate a mode change.
    let perm_dir = TempDir::new().unwrap();
    init_permissive(&perm_dir);
    dont()
        .args(["conclude", "A claim with unresolved dep", "--depends-on", "NS:UNRESOLVED", "--json"])
        .env("DONT_DIR", perm_dir.path())
        .assert()
        .success();

    // Overwrite the mode in config.toml to strict
    switch_mode(&perm_dir, "permissive", "strict");

    let output = dont()
        .args(["rules", "test", "ungrounded", "--json"])
        .env("DONT_DIR", perm_dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["severity"], "strict",
        "ungrounded rule should report strict severity after mode switch to strict");
    assert!(!v["data"]["matches"].as_array().unwrap().is_empty(),
        "should still show the unresolved CURIE dep as a match");
}

#[test]
fn mode_switch_from_strict_to_permissive_changes_ungrounded_severity() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);
    dont()
        .args(["conclude", "A claim with unresolved dep", "--depends-on", "NS:UNRESOLVED", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Confirm warn severity while permissive
    let output_perm = dont()
        .args(["rules", "test", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v_perm: Value = serde_json::from_slice(&output_perm).unwrap();
    assert_eq!(v_perm["data"]["severity"], "warn");

    // Switch to strict
    switch_mode(&dir, "permissive", "strict");

    // Severity should reflect new mode immediately (no restart needed)
    let output_strict = dont()
        .args(["rules", "test", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v_strict: Value = serde_json::from_slice(&output_strict).unwrap();
    assert_eq!(v_strict["data"]["severity"], "strict",
        "severity should change to strict immediately after config.toml mode switch");
}

#[test]
fn ungrounded_rule_severity_reads_persisted_mode_not_default() {
    let dir = TempDir::new().unwrap();
    // Init strict, then manually write permissive to config.toml to confirm
    // the engine reads the persisted value, not a compile-time default.
    init_strict(&dir);

    switch_mode(&dir, "strict", "permissive");

    // Create claim (now permissive, will succeed)
    dont()
        .args(["conclude", "A claim with unresolved dep", "--depends-on", "NS:UNRESOLVED", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["rules", "test", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    // Must be "warn", not "strict" — proves engine reads persisted config, not init-time default
    assert_eq!(v["data"]["severity"], "warn",
        "engine must read persisted mode from config.toml, not a hardcoded default");
    // Claim must still be queryable after the config mutation
    assert!(!v["data"]["matches"].as_array().unwrap().is_empty(),
        "claim with unresolved dep must survive the mode switch intact");
}
