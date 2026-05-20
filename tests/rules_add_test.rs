mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// --- rules add ---

#[test]
fn rules_add_valid_dl_file_copies_into_rules_dir() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("my-rule.dl");
    std::fs::write(
        &rule_file,
        r#"?[entity_id, detail] <- [["entity-1", "found"]]"#,
    )
    .unwrap();

    dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    assert!(
        dir.path().join("rules").join("my-rule.dl").exists(),
        "rule file should be copied to .dont/rules/"
    );
}

#[test]
fn rules_add_returns_empty_envelope_on_success() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("new-rule.dl");
    std::fs::write(
        &rule_file,
        r#"?[entity_id, detail] <- [["e", "d"]]"#,
    )
    .unwrap();

    let out = dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "empty");
}

#[test]
fn rules_add_success_hints_include_severity_guidance() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("my-custom-rule.dl");
    std::fs::write(
        &rule_file,
        r#"?[entity_id, detail] <- [["e", "d"]]"#,
    )
    .unwrap();

    let out = dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let hints = v["hints"].as_array().expect("hints array must be present");
    let hint_commands: Vec<&str> = hints
        .iter()
        .filter_map(|h| h["command"].as_str())
        .collect();
    // At least one hint must mention severity / config to guide the operator.
    let has_severity_hint = hint_commands.iter().any(|c| c.contains("warn") || c.contains("strict") || c.contains("severity"));
    assert!(
        has_severity_hint,
        "rules add should hint the operator about severity configuration; got: {:?}",
        hint_commands
    );
}

#[test]
fn rules_add_makes_rule_appear_in_rules_list() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("added-rule.dl");
    std::fs::write(
        &rule_file,
        r#"?[entity_id, detail] <- [["e", "d"]]"#,
    )
    .unwrap();

    dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let rules = v["data"].as_array().unwrap();
    let names: Vec<&str> = rules.iter().filter_map(|r| r["name"].as_str()).collect();
    assert!(names.contains(&"added-rule"), "added rule should appear in list");
}

#[test]
fn rules_add_invalid_dl_emits_compile_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("bad-rule.dl");
    std::fs::write(&rule_file, "this is not valid datalog !!!@#$").unwrap();

    let out = dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-compile-error");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn rules_add_nonexistent_file_emits_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "add", "/nonexistent/path/rule.dl", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn rules_add_shipped_rule_name_emits_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("ungrounded.dl");
    std::fs::write(&rule_file, r#"?[entity_id, detail] <- [["e", "d"]]"#).unwrap();

    let out = dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "cannot-shadow-shipped-rule");
}

#[test]
fn rules_add_duplicate_name_without_force_emits_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("my-rule.dl");
    std::fs::write(&rule_file, r#"?[entity_id, detail] <- [["e", "d"]]"#).unwrap();

    // First add succeeds.
    dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Second add without --force fails.
    let out = dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-already-exists");
}

#[test]
fn rules_add_duplicate_name_with_force_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rule_file = dir.path().join("my-rule.dl");
    std::fs::write(&rule_file, r#"?[entity_id, detail] <- [["e", "d"]]"#).unwrap();

    dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Update the source and re-add with --force.
    std::fs::write(&rule_file, r#"?[entity_id, detail] <- [["e2", "d2"]]"#).unwrap();

    dont()
        .args(["rules", "add", rule_file.to_str().unwrap(), "--force", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Verify the updated content is present.
    let dest = dir.path().join("rules").join("my-rule.dl");
    let content = std::fs::read_to_string(&dest).unwrap();
    assert!(content.contains("e2"), "rule file should contain updated content");
}

// --- rules test ---

#[test]
fn rules_test_shipped_rule_returns_rule_result_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "test", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "rule_result");
    assert_eq!(v["data"]["rule_name"], "ungrounded");
    assert!(
        matches!(v["data"]["severity"].as_str(), Some("warn") | Some("strict")),
        "severity must be present"
    );
    assert!(v["data"]["matches"].is_array());
}

#[test]
fn rules_test_custom_rule_that_fires_returns_matches() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Write a rule directly to the rules dir (simulating a pre-installed custom rule).
    let rules_dir = dir.path().join("rules");
    std::fs::write(
        rules_dir.join("always-fires.dl"),
        r#"?[entity_id, detail] <- [["test-entity", "test violation"]]"#,
    )
    .unwrap();

    let out = dont()
        .args(["rules", "test", "always-fires", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "rule_result");
    let matches = v["data"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["entity_id"], "test-entity");
    assert_eq!(matches[0]["detail"], "test violation");
}

#[test]
fn rules_test_does_not_modify_store_state() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let rules_dir = dir.path().join("rules");
    std::fs::write(
        rules_dir.join("always-fires.dl"),
        r#"?[entity_id, detail] <- [["x", "y"]]"#,
    )
    .unwrap();

    // Run test twice; store state should be identical (no tx advance visible via list).
    let out1 = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    dont()
        .args(["rules", "test", "always-fires", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out2 = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v1: Value = serde_json::from_slice(&out1).unwrap();
    let v2: Value = serde_json::from_slice(&out2).unwrap();
    let claims1 = v1["data"]["claims"].as_array().or_else(|| v1["data"].as_array()).unwrap();
    let claims2 = v2["data"]["claims"].as_array().or_else(|| v2["data"].as_array()).unwrap();
    assert_eq!(
        claims1.len(),
        claims2.len(),
        "rules test must not alter claim count"
    );
}

#[test]
fn rules_test_unknown_rule_returns_rule_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "test", "nonexistent-rule", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-found");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}
