mod common;

use common::{conclude_claim, dont, init_dir};
use predicates::prelude::*;
use tempfile::TempDir;

// --- --quiet suppresses confirmatory output ---

#[test]
fn conclude_quiet_produces_no_stdout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["conclude", "quiet test claim", "--quiet", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn init_quiet_produces_no_stdout() {
    let dir = TempDir::new().unwrap();

    dont()
        .args(["init", "--quiet", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn flag_quiet_produces_no_stdout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "quiet flag test");

    dont()
        .args([
            "flag",
            &id,
            "--evidence",
            "https://example.com",
            "--quiet",
            "--human",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// --- --quiet does not suppress errors ---

#[test]
fn quiet_does_not_suppress_validation_errors() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["conclude", "", "--quiet", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn quiet_does_not_suppress_unknown_entity_errors() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args([
            "show",
            "claim:nonexistent00000000000000",
            "--quiet",
            "--human",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// --- --quiet with --json: --json wins, data still emitted ---

#[test]
fn quiet_with_json_still_emits_json_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "conclude",
            "quiet json test",
            "--quiet",
            "--json",
            "--human",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // --json takes precedence even with --human: full envelope must appear on stdout
    let v: serde_json::Value =
        serde_json::from_slice(&out).expect("--json must emit valid JSON even with --quiet");
    assert_eq!(v["ok"], true);
}

// --- --quiet is documented in --help ---

#[test]
fn conclude_help_documents_quiet_flag() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["conclude", "--help"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("--quiet"));
}

#[test]
fn init_help_documents_quiet_flag() {
    let dir = TempDir::new().unwrap();

    dont()
        .args(["init", "--help"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("--quiet"));
}

// --- list --quiet: data still appears (--quiet only suppresses confirmations) ---

#[test]
fn list_quiet_still_shows_claims() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "claim to list quietly");

    let out = dont()
        .args(["list", "--quiet"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("claim to list quietly"),
        "--quiet must not suppress data output (list), got: {text:?}"
    );
}
