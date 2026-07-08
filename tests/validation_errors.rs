/// dont-pyl: Input validation error messages — consistent format and actionability.
///
/// Consistency rule: error messages must name (1) the field/argument that failed,
/// (2) the reason it failed, and (3) the expected format or valid values.
/// Message format: `<field>: <reason>; expected <format>`
/// (displayed on stderr as `error: <field>: <reason>; expected <format>`)
mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn init() -> TempDir {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dir
}

fn json_message(stdout: &[u8]) -> String {
    let v: Value = serde_json::from_slice(stdout).unwrap();
    v["data"]["message"].as_str().unwrap_or("").to_string()
}

// ── conclude ──────────────────────────────────────────────────────────────────

/// Empty statement: error message must name "statement" as the failed field.
#[test]
fn conclude_empty_statement_names_field() {
    let dir = init();
    let out = dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("statement:"),
        "message must start with 'statement:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

/// Shell metacharacter in statement: error message must name "statement".
#[test]
fn conclude_metachar_statement_names_field() {
    let dir = init();
    let out = dont()
        .args(["conclude", "bad|pipe", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("statement:"),
        "message must start with 'statement:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

/// Human-readable stderr for empty statement must contain "error: statement:".
#[test]
fn conclude_empty_statement_stderr_contains_field() {
    let dir = init();
    dont()
        .args(["conclude", ""])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .stderr(predicates::str::contains("error: statement:"));
}

// ── define ────────────────────────────────────────────────────────────────────

/// Missing --doc: error message must name "--doc" as the failed field.
#[test]
fn define_missing_doc_names_field() {
    let dir = init();
    let out = dont()
        .args(["define", "ex:Foo", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("--doc:"),
        "message must start with '--doc:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

/// Human-readable stderr for missing --doc must contain "error: --doc:".
#[test]
fn define_missing_doc_stderr_contains_field() {
    let dir = init();
    dont()
        .args(["define", "ex:Foo"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .stderr(predicates::str::contains("error: --doc:"));
}

// ── trust ─────────────────────────────────────────────────────────────────────

/// Missing --reason on trust: error message must name "--reason".
#[test]
fn trust_missing_reason_names_field() {
    let dir = init();
    // conclude a claim first so the entity exists
    let claim_out = dont()
        .args(["conclude", "the earth is round", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&claim_out).unwrap();
    let id = v["data"]["id"].as_str().unwrap();

    let out = dont()
        .args(["trust", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("--reason:"),
        "message must start with '--reason:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

// ── ignore ────────────────────────────────────────────────────────────────────

/// Missing --reason on ignore: error message must name "--reason".
#[test]
fn ignore_missing_reason_names_field() {
    let dir = init();
    let claim_out = dont()
        .args(["conclude", "the earth is round", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&claim_out).unwrap();
    let id = v["data"]["id"].as_str().unwrap();

    let out = dont()
        .args(["ignore", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("--reason:"),
        "message must start with '--reason:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

// ── flag ──────────────────────────────────────────────────────────────────────

/// Missing --evidence on flag: error message must name "--evidence".
#[test]
fn flag_missing_evidence_names_field() {
    let dir = init();
    let claim_out = dont()
        .args(["conclude", "a claim to flag", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&claim_out).unwrap();
    let id = v["data"]["id"].as_str().unwrap();

    let out = dont()
        .args(["flag", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("--evidence:"),
        "message must start with '--evidence:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

// ── ground ────────────────────────────────────────────────────────────────────

/// Empty statement on ground: error message must name "statement".
#[test]
fn ground_empty_statement_names_field() {
    let dir = init();
    let out = dont()
        .args(["ground", "", "--evidence", "https://example.com", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("statement:"),
        "message must start with 'statement:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

/// Missing --evidence on ground: error message must name "--evidence".
#[test]
fn ground_missing_evidence_names_field() {
    let dir = init();
    let out = dont()
        .args(["ground", "valid statement", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("--evidence:"),
        "message must start with '--evidence:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}

// ── validate_claim_statement (path traversal) ──────────────────────────────

/// Path traversal in statement: error message must name "statement".
#[test]
fn conclude_path_traversal_statement_names_field() {
    let dir = init();
    let out = dont()
        .args(["conclude", "../bad/path", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let msg = json_message(&out);
    assert!(
        msg.starts_with("statement:"),
        "message must start with 'statement:'; got: {msg:?}"
    );
    assert!(
        msg.contains("expected"),
        "message must contain 'expected'; got: {msg:?}"
    );
}
