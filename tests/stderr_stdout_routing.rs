mod common;

use common::{conclude_claim, dont, init_dir};
use predicates::prelude::*;
use tempfile::TempDir;

// --- errors go to stderr, never to stdout ---

#[test]
fn error_on_missing_required_arg_goes_to_stderr_not_stdout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // `conclude` with no statement — clap validation error
    dont()
        .args(["conclude"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("error"));
}

#[test]
fn validation_error_without_json_goes_to_stderr_only() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // empty statement is a validation error
    dont()
        .args(["conclude", ""])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn validation_error_with_json_flag_stdout_is_json_stderr_is_empty() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .clone();

    // stdout must be valid JSON envelope
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json error output must be valid JSON on stdout");
    assert_eq!(v["ok"], false, "error envelope must have ok=false");

    // stderr must be empty (no duplicate error prose when --json is set)
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.is_empty(),
        "--json mode must not write error prose to stderr; got: {stderr:?}"
    );
}

#[test]
fn unknown_entity_error_goes_to_stderr_not_stdout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["show", "claim:nonexistent00000000000000"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("error"));
}

// --- data output goes to stdout ---

#[test]
fn successful_json_output_has_data_on_stdout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "stdout data test");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).expect("stdout must be JSON");
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object());
}

#[test]
fn list_json_output_only_on_stdout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "claim for routing test");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert!(
        stderr.is_empty(),
        "list --json must not write to stderr on success; got: {stderr:?}"
    );
    serde_json::from_slice::<serde_json::Value>(&out.stdout)
        .expect("list --json stdout must be valid JSON");
}

#[test]
fn warnings_go_to_stderr_not_into_json_stdout() {
    // Deprecation warnings emitted by `dismiss` (alias for `flag`) must not
    // contaminate stdout JSON — they go to stderr or the envelope `warnings` field.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim with deprecation path");

    let out = dont()
        .args(["dismiss", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();

    // stdout must be valid JSON regardless of the deprecation warning on stderr
    let stdout = String::from_utf8(out.stdout).unwrap();
    serde_json::from_str::<serde_json::Value>(&stdout)
        .expect("stdout must be valid JSON even when a deprecation warning is emitted to stderr");

    // the deprecation warning must be on stderr, not contaminating stdout
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("deprecated"),
        "dismiss must emit deprecation warning to stderr; got: {stderr:?}"
    );
}

// --- human output (no --json) goes to stdout ---

#[test]
fn human_success_output_goes_to_stdout_not_stderr() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "human routing check"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.is_empty(),
        "human mode success must not write to stderr; got: {stderr:?}"
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.is_empty(),
        "human mode success must write output to stdout"
    );
}

#[test]
fn init_success_output_goes_to_stdout_not_stderr() {
    let dir = TempDir::new().unwrap();

    let out = dont()
        .args(["init"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.is_empty(),
        "init success must not write to stderr; got: {stderr:?}"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.is_empty(),
        "init success must write confirmation to stdout"
    );
}
