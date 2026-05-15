mod common;

use common::dont;
use dont::store::{
    HypothesisAssessment, HypothesisRecord, Store, StoreEvent, StoreEventKind, StoreStatus,
};
use serde_json::Value;
use tempfile::TempDir;

fn init_dir(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
}

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn define_term(dir: &TempDir, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn trust(dir: &TempDir, id: &str) {
    dont()
        .args(["trust", id, "--reason", "specific grounds for doubt", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
}

fn undoubt(dir: &TempDir, id: &str) -> Vec<u8> {
    dont()
        .args(["undoubt", id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

// --- Valid transitions ---

#[test]
fn undoubt_doubted_claim_returns_to_unverified() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim that will be doubted then undoubted");
    trust(&dir, &id);
    let out = undoubt(&dir, &id);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn undoubt_doubted_term_returns_to_unverified() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");
    trust(&dir, &id);
    let out = undoubt(&dir, &id);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn undoubt_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "tx is tracked for undoubt");
    trust(&dir, &id);
    let out = undoubt(&dir, &id);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["meta"]["tx"].is_number());
}

// --- Refusals ---

#[test]
fn undoubt_unverified_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "unverified claim cannot be undoubted");
    let out = dont()
        .args(["undoubt", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn undoubt_verified_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "verified claim cannot be undoubted");
    dont()
        .args(["flag", &id, "--evidence", "https://example.test/proof", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
    let out = dont()
        .args(["undoubt", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn undoubt_ignored_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "ignored claim cannot be undoubted");
    dont()
        .args(["ignore", &id, "--reason", "out of scope for this release", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
    let out = dont()
        .args(["undoubt", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn undoubt_entity_not_found_returns_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["undoubt", "claim:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "entity-not-found");
}

#[test]
fn undoubt_locked_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim that will be locked and then undoubt is attempted");

    // Seed the claim to meet locking prerequisites: 2+ evidence, 3+ assessed hypotheses
    let store = Store::open_dont_dir(dir.path().join(".dont")).unwrap();
    store
        .append_status_change(
            &id,
            StoreStatus::Unverified,
            StoreStatus::Verified,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String(
                    "https://source-one.example/evidence".to_string(),
                )],
            },
        )
        .unwrap();
    store
        .append_evidence_event(
            &id,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String(
                    "https://source-two.example/evidence".to_string(),
                )],
            },
        )
        .unwrap();
    let hypotheses: Vec<HypothesisRecord> = (0..3)
        .map(|idx| HypothesisRecord {
            idx,
            text: format!("hypothesis {}", idx + 1),
            assessment: HypothesisAssessment {
                supporting: vec![format!("support-{}", idx + 1)],
                refuting: vec![],
            },
        })
        .collect();
    store.set_claim_hypotheses_for_test(&id, &hypotheses).unwrap();
    drop(store);

    // Lock the claim via CLI
    dont()
        .args(["forget", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();

    // Now attempt to undoubt the locked claim — must be refused
    let out = dont()
        .args(["undoubt", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "invalid-transition",
        "undoubt on a locked claim must return invalid-transition, got: {:?}",
        v["data"]["code"]
    );
}
