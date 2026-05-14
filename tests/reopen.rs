mod common;

use common::{conclude_claim, dont, init_dir};
use dont::store::{
    HypothesisAssessment, HypothesisRecord, Store, StoreEvent, StoreEventKind, StoreStatus,
};
use serde_json::Value;
use tempfile::TempDir;

fn seed_verified_claim_with_evidence(dir: &TempDir, claim_id: &str, evidence: &[&str]) {
    let store = Store::open_dont_dir(dir.path()).unwrap();
    let first = evidence.first().expect("at least one evidence item");
    store
        .append_status_change(
            claim_id,
            StoreStatus::Unverified,
            StoreStatus::Verified,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String((*first).to_string())],
            },
        )
        .unwrap();
    for uri in &evidence[1..] {
        store
            .append_evidence_event(
                claim_id,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![serde_json::Value::String((*uri).to_string())],
                },
            )
            .unwrap();
    }
}

fn seed_assessed_hypotheses(dir: &TempDir, claim_id: &str, count: usize) {
    let store = Store::open_dont_dir(dir.path()).unwrap();
    let hypotheses: Vec<HypothesisRecord> = (0..count)
        .map(|idx| HypothesisRecord {
            idx,
            text: format!("hypothesis {}", idx + 1),
            assessment: HypothesisAssessment {
                supporting: vec![format!("support-{}", idx + 1)],
                refuting: vec![],
            },
        })
        .collect();
    store
        .set_claim_hypotheses_for_test(claim_id, &hypotheses)
        .unwrap();
}

fn ignore_entity(dir: &TempDir, id: &str) {
    dont()
        .args(["ignore", id, "--reason", "out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn define_term(dir: &TempDir, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

#[test]
fn reopen_ignored_claim_produces_unverified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");
    ignore_entity(&dir, &id);

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
    assert_eq!(v["data"]["id"], id);
}

#[test]
fn reopen_unverified_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn reopen_verified_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    dont()
        .args(["flag", &id, "--evidence", "https://example.com/e1", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn reopen_doubted_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    dont()
        .args(["trust", &id, "--reason", "contradicted by experiment", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn reopen_ignored_term_produces_unverified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");
    ignore_entity(&dir, &id);

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn reopen_locked_claim_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "A claim that will be permanently locked");
    seed_verified_claim_with_evidence(
        &dir,
        &id,
        &[
            "https://source-one.example/evidence",
            "https://source-two.example/evidence",
        ],
    );
    seed_assessed_hypotheses(&dir, &id, 3);

    let lock_out = dont()
        .args(["forget", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let lock_v: Value = serde_json::from_slice(&lock_out).unwrap();
    assert_eq!(lock_v["data"]["status"], "locked", "forget must produce locked status");

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn reopen_entity_not_found_returns_structured_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["reopen", "claim:NOTEXIST", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "entity-not-found");
}

#[test]
fn reopen_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");
    ignore_entity(&dir, &id);

    let output = dont()
        .args(["reopen", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert!(v["meta"]["tx"].as_i64().unwrap() > 0);
}
