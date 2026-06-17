mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn list_claims(dir: &TempDir) -> Vec<Value> {
    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["claims"].as_array().cloned().unwrap_or_default()
}

#[test]
fn no_persist_conclude_returns_success_without_writing() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "an ephemeral claim", "--no-persist", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "no-persist conclude should return ok=true");
    assert_eq!(v["envelope_kind"], "claim");
    // The store should remain empty
    let claims = list_claims(&dir);
    assert!(
        claims.is_empty(),
        "no-persist must not write to store; found {} claims",
        claims.len()
    );
}

#[test]
fn no_persist_conclude_duplicate_returns_error_without_writing() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    // First, conclude a claim normally
    conclude_claim(&dir, "the cache expires after 60 seconds");
    // Now try to conclude the same text with --no-persist
    let out = dont()
        .args([
            "conclude",
            "the cache expires after 60 seconds",
            "--no-persist",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        v["ok"], false,
        "duplicate conclude with --no-persist should return error"
    );
    let code = v["data"]["code"].as_str().unwrap();
    assert_eq!(
        code, "duplicate-refused",
        "error code must be duplicate-refused; got: {code}"
    );
    // Still only one claim in the store
    let claims = list_claims(&dir);
    assert_eq!(
        claims.len(),
        1,
        "no-persist must not add to store; found {} claims",
        claims.len()
    );
}

#[test]
fn no_persist_trust_with_hedge_returns_error_without_writing() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the payment gateway is stable");
    let out = dont()
        .args([
            "trust",
            &id,
            "--reason",
            "maybe it works",
            "--no-persist",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        v["ok"], false,
        "hedge rejection should propagate through --no-persist"
    );
    let code = v["data"]["code"].as_str().unwrap();
    assert!(
        code.contains("hedge"),
        "error code should mention hedge; got: {code}"
    );
    // Status should remain unverified
    let show_out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show: Value = serde_json::from_slice(&show_out).unwrap();
    assert_eq!(
        show["data"]["status"], "unverified",
        "status must not change after no-persist hedge rejection"
    );
}

#[test]
fn no_persist_on_readonly_command_is_noop() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "a claim to list");
    // list with --no-persist should behave identically to list without it
    let with_flag: Value = serde_json::from_slice(
        &dont()
            .args(["list", "--no-persist", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();
    let without_flag: Value = serde_json::from_slice(
        &dont()
            .args(["list", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();
    assert_eq!(
        with_flag["data"]["claims"].as_array().unwrap().len(),
        without_flag["data"]["claims"].as_array().unwrap().len(),
        "--no-persist on list must not change output"
    );
}

#[test]
fn no_persist_trust_valid_returns_success_without_writing() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the retry logic uses exponential backoff");
    let out = dont()
        .args([
            "trust",
            &id,
            "--reason",
            "confirmed in production logs from 2026-06-01",
            "--no-persist",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], true,
        "valid trust with --no-persist should return ok=true"
    );
    // Status must remain unverified
    let show_out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show: Value = serde_json::from_slice(&show_out).unwrap();
    assert_eq!(
        show["data"]["status"], "unverified",
        "no-persist trust must not change claim status in store"
    );
}

#[test]
fn no_persist_flag_is_accepted_on_all_subcommands_without_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    // --no-persist must not cause an unrecognised flag error on read-only commands
    let read_only_cmds: &[&[&str]] = &[
        &["list", "--no-persist", "--json"],
        &["doctor", "--no-persist", "--json"],
    ];
    for args in read_only_cmds {
        let out = dont()
            .args(*args)
            .env("DONT_DIR", dir.path())
            .output()
            .unwrap();
        // Must not be a flag-parse failure (exit 2 is clap's usage error code)
        assert_ne!(
            out.status.code(),
            Some(2),
            "--no-persist must be accepted on {:?} without parse error",
            args
        );
    }
}

#[test]
fn dont_no_persist_env_var_makes_writes_ephemeral() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "env var driven ephemeral claim", "--json"])
        .env("DONT_DIR", dir.path())
        .env("DONT_NO_PERSIST", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], true,
        "DONT_NO_PERSIST=1 conclude should return ok=true"
    );
    // No claim should be written
    let claims = list_claims(&dir);
    assert!(
        claims.is_empty(),
        "DONT_NO_PERSIST=1 must not write to store; found {} claims",
        claims.len()
    );
}

#[test]
fn dont_no_persist_env_var_0_is_treated_as_unset() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "a persisted claim", "--json"])
        .env("DONT_DIR", dir.path())
        .env("DONT_NO_PERSIST", "0")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    // Claim SHOULD be written when DONT_NO_PERSIST=0
    let claims = list_claims(&dir);
    assert_eq!(
        claims.len(),
        1,
        "DONT_NO_PERSIST=0 must not suppress writes"
    );
}

#[test]
fn no_persist_response_has_ephemeral_true() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args([
            "conclude",
            "ephemeral claim for envelope check",
            "--no-persist",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ephemeral"], true,
        "envelope.ephemeral must be true when --no-persist is used"
    );
}

#[test]
fn normal_response_has_no_ephemeral_field() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "a normal persisted claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(
        v.get("ephemeral").is_none() || v["ephemeral"].is_null(),
        "envelope.ephemeral must be absent on normal (non-ephemeral) responses"
    );
}
