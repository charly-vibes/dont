mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn list_terms_envelope_kind_is_term_list() {
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
    assert_eq!(
        v["envelope_kind"], "term_list",
        "spec requires envelope_kind=term_list for term listings"
    );
}

#[test]
fn vocab_list_envelope_kind_is_term_list() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["vocab", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["envelope_kind"], "term_list",
        "spec requires envelope_kind=term_list for vocab listings"
    );
}

#[test]
fn trace_envelope_kind_is_events() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let claim_out = dont()
        .args(["conclude", "test claim for trace", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let cv: Value = serde_json::from_slice(&claim_out).unwrap();
    let id = cv["data"]["id"].as_str().unwrap().to_string();

    let out = dont()
        .args(["trace", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["envelope_kind"], "events",
        "spec requires envelope_kind=events for trace output"
    );
}

#[test]
fn success_envelope_always_carries_hints_key() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(
        v.get("hints").is_some(),
        "spec requires hints key always present on success envelopes"
    );
    assert!(v["hints"].is_array(), "hints must be an array");
}

#[test]
fn error_envelope_does_not_carry_hints() {
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
    assert!(
        v.get("hints").is_none(),
        "spec requires error envelopes do not carry hints key"
    );
}

#[test]
fn envelope_kind_error_on_failure() {
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
    assert_eq!(
        v["envelope_kind"], "error",
        "spec requires envelope_kind=error when ok=false"
    );
}
