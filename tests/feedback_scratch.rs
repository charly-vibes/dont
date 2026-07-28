/// dont-rxf6 (genesis::guide adoption): the ErrorSink scratch contract writes
/// an error record on every non-zero exit, and `dont feedback bug
/// --from-last-error --dry-run --json` reads it back into the issue body.
///
/// Before the genesis::guide adoption, nothing wrote the scratch, so
/// `--from-last-error` always reported "No recent error found". These tests
/// pin the end-to-end loop: error → scratch → feedback body.
///
/// Each test redirects `XDG_CACHE_HOME` to its own temp dir so the scratch
/// file is isolated from other parallel tests (the shared `common::dont()`
/// helper uses a single shared cache dir).
mod common;

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

/// Build a `dont` Command with an isolated XDG_CACHE_HOME so the error
/// scratch for this test does not collide with other parallel tests.
fn dont_isolated(cache_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("dont").unwrap();
    cmd.env("XDG_CACHE_HOME", cache_dir);
    cmd
}

/// A command that exits non-zero writes an error scratch record, and
/// `feedback bug --from-last-error --dry-run --json` surfaces it under
/// "Last Error Context".
#[test]
fn feedback_from_last_error_reads_scratch_written_on_error() {
    let dir = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    common::init_dir(&dir);

    // Trigger a non-zero exit (empty statement is refused) — this writes the
    // error scratch via the ErrorSink contract in emit_error_no_exit.
    dont_isolated(cache.path())
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure();

    // The feedback command reads the last error scratch and embeds it.
    let out = dont_isolated(cache.path())
        .args([
            "feedback",
            "bug",
            "--from-last-error",
            "--dry-run",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let body = v["body"].as_str().expect("feedback body present");
    assert!(
        body.contains("### Last Error Context"),
        "body should include the last-error context section, got: {body}"
    );
    assert!(
        body.contains("Exit code: 1"),
        "body should report the non-zero exit code, got: {body}"
    );
    assert!(
        body.contains("conclude"),
        "body should name the failing command, got: {body}"
    );
}

/// When no error has been recorded, --from-last-error reports the absence
/// rather than failing — the scratch read is best-effort.
#[test]
fn feedback_from_last_error_with_no_prior_error_reports_absence() {
    let dir = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    common::init_dir(&dir);

    let out = dont_isolated(cache.path())
        .args([
            "feedback",
            "bug",
            "--from-last-error",
            "--dry-run",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let body = v["body"].as_str().expect("feedback body present");
    assert!(
        body.contains("No recent error found"),
        "body should report no recent error when scratch is empty, got: {body}"
    );
}
