mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// --- dont list on empty project ---

/// `dont list` on a fresh project (JSON) must return a valid envelope with an
/// empty claims array, count=0, and exit 0.  No panics, no error.
#[test]
fn list_empty_project_json_returns_valid_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claims");
    assert_eq!(
        v["data"]["claims"].as_array().unwrap().len(),
        0,
        "empty project must have zero claims"
    );
    assert_eq!(
        v["data"]["count"], 0,
        "count field must be 0 for empty project"
    );
}

/// `dont list` on a fresh project (human) must print a message that includes an
/// actionable suggestion so users know how to get started.
#[test]
fn list_empty_project_human_suggests_conclude() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON"
    );
    assert!(
        text.contains("dont conclude"),
        "empty list must suggest 'dont conclude' to the user, got: {text}"
    );
}

// --- dont list --kind terms on empty project ---

/// `dont list --kind terms` on a fresh project (JSON) must return a valid
/// envelope with an empty data array and exit 0.  No panics, no error.
#[test]
fn list_empty_terms_json_returns_valid_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--kind", "terms", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "terms");
    assert_eq!(
        v["data"].as_array().unwrap().len(),
        0,
        "empty project must have zero terms"
    );
}

/// `dont list --kind terms` on a fresh project (human) must print a message
/// that includes an actionable suggestion to define terms.
#[test]
fn list_empty_terms_human_suggests_define() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--kind", "terms", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON"
    );
    assert!(
        text.contains("dont define"),
        "empty terms list must suggest 'dont define' to the user, got: {text}"
    );
}

// --- dont show on empty project (no entity) ---

/// `dont show` with any entity ID on a fresh project must return a structured
/// error with a remediation hint — not a panic.
#[test]
fn show_on_empty_project_returns_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["show", "claim:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "not-found error must include remediation hints"
    );
}

// --- dont trace on empty project (no entity) ---

/// `dont trace` with any entity ID on a fresh project must return a structured
/// error — not a panic.
#[test]
fn trace_on_empty_project_returns_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["trace", "claim:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "not-found error must include remediation hints"
    );
}

// --- dont why on empty project (no entity) ---

/// `dont why` with any entity ID on a fresh project must return a structured
/// error — not a panic.
#[test]
fn why_on_empty_project_returns_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["why", "claim:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "not-found error must include remediation hints"
    );
}
