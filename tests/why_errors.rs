mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

/// `dont why` on a claim ID that does not exist must return a structured
/// "claim-not-found" error (exit 1) — the same contract as `show` and `trace`.
#[test]
fn why_unknown_claim_id_returns_claim_not_found_exit_1() {
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
    assert_eq!(
        v["data"]["code"], "claim-not-found",
        "why must return claim-not-found for missing claim id, got: {:?}",
        v["data"]["code"]
    );
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "claim-not-found error must include remediation hints"
    );
}

/// `dont why` on a term ID that does not exist must return a structured
/// "term-not-found" error (exit 1).
#[test]
fn why_unknown_term_id_returns_term_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["why", "term:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "term-not-found",
        "why must return term-not-found for missing term id, got: {:?}",
        v["data"]["code"]
    );
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "term-not-found error must include remediation hints"
    );
}

/// `dont why` on an unknown CURIE (NS:local form) must return "term-not-found"
/// and include the CURIE in the error message.
#[test]
fn why_unknown_curie_returns_term_not_found_with_curie_in_message() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["why", "WB:ZZZZ", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "term-not-found",
        "why must return term-not-found for unknown CURIE, got: {:?}",
        v["data"]["code"]
    );
    let msg = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("WB:ZZZZ"),
        "error message must include the unknown CURIE, got: {msg}"
    );
}
