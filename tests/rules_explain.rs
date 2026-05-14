mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn explain_ungrounded_returns_why_explain_payload() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["explain", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["rule_name"], "ungrounded");
    assert!(
        v["data"]["explanation"].as_str().unwrap().len() > 20,
        "explanation should be non-trivial prose"
    );
}

#[test]
fn explain_returns_severity_for_shipped_rule() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["explain", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let severity = v["data"]["severity"].as_str().unwrap();
    assert!(
        severity == "warn" || severity == "strict",
        "severity must be warn or strict, got: {severity}"
    );
}

#[test]
fn explain_unknown_rule_emits_not_found_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["explain", "nonexistent-rule", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-found");
}

#[test]
fn explain_works_without_json_flag() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["explain", "ungrounded"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}
