mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn assert_valid_envelope(bytes: &[u8], expected_kind: &str) {
    let v: Value = serde_json::from_slice(bytes).unwrap_or_else(|e| {
        panic!(
            "stdout is not valid JSON: {e}\n{}",
            String::from_utf8_lossy(bytes)
        )
    });
    assert_eq!(
        v["ok"], true,
        "envelope ok must be true for {expected_kind}"
    );
    assert_eq!(v["envelope_kind"], expected_kind, "envelope_kind mismatch");
    assert!(v["data"].is_object(), "data must be a JSON object");
    assert!(
        v["error"].is_null(),
        "error must be null on success for {expected_kind}"
    );
}

// --- data-outputting commands accept --json and emit valid envelopes ---

#[test]
fn version_json_emits_version_envelope() {
    let out = dont()
        .args(["--version", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "version");
}

#[test]
fn list_json_emits_claims_envelope() {
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
    assert_valid_envelope(&out, "claims");
}

#[test]
fn show_claim_json_emits_claim_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a testable claim");
    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "claim");
}

#[test]
fn why_json_emits_why_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim to explain");
    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "why");
}

#[test]
fn prime_json_emits_prime_envelope() {
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
    assert_valid_envelope(&out, "prime");
}

#[test]
fn trace_json_emits_trace_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim to trace");
    let out = dont()
        .args(["trace", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "events");
}

#[test]
fn rules_list_json_emits_rules_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "rule_list");
    // data is an array of rule objects
    assert!(
        v["data"].is_array(),
        "data must be a JSON array for rule_list"
    );
}

#[test]
fn explain_json_emits_explain_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["explain", "lockable", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "dont-explain");
}

#[test]
fn completions_json_emits_completions_envelope() {
    let out = dont()
        .args(["completions", "bash", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "dont-completions");
}

// --- mutation commands with --json emit valid success envelopes ---

#[test]
fn conclude_json_emits_envelope_with_id() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "test claim for json flag", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["data"]["id"].is_string(), "data.id must be present");
}

#[test]
fn init_json_emits_envelope() {
    let dir = TempDir::new().unwrap();
    let out = dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object());
}

// --- --json output is pure JSON (no prose contamination) ---

#[test]
fn list_json_stdout_is_single_json_object() {
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
    let text = String::from_utf8(out).unwrap();
    let trimmed = text.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "stdout must be a single JSON object, got: {text}"
    );
}

#[test]
fn show_term_json_emits_term_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["define", "ex:Widget", "--doc", "a widget thing", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let out = dont()
        .args(["show", "ex:Widget", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_valid_envelope(&out, "term");
}
