mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// --- dont check ---

#[test]
fn check_passes_on_empty_project() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["check", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["ungrounded"], false);
    assert_eq!(v["data"]["unverified_count"], 0);
    assert_eq!(v["data"]["total_claims"], 0);
}

#[test]
fn check_fails_on_unverified_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "ungrounded claim");

    dont()
        .args(["check", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1);
}

#[test]
fn check_json_reports_unverified_count() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "first ungrounded");
    conclude_claim(&dir, "second ungrounded");

    let out = dont()
        .args(["check", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["ungrounded"], true);
    assert_eq!(v["data"]["unverified_count"], 2);
    assert_eq!(v["data"]["total_claims"], 2);
}

#[test]
fn check_passes_when_all_claims_verified() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "grounded claim");

    // Flag the claim with evidence to verify it
    dont()
        .args([
            "flag",
            &id,
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["check", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["ungrounded"], false);
    assert_eq!(v["data"]["unverified_count"], 0);
}

#[test]
fn check_human_output_shows_summary() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "ungrounded");

    // Without --json, human output is plain text; exit code signals status
    let out = dont()
        .args(["check", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("ungrounded"),
        "human output should mention ungrounded claims, got: {stdout}"
    );
}

#[test]
fn check_passes_human_on_empty_project() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["check", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("all claims grounded"),
        "human output should confirm all grounded, got: {stdout}"
    );
}
