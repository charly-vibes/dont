mod common;

use common::{conclude_claim, dont, init_dir};
use dont::store::{
    HypothesisAssessment, HypothesisRecord, Status, Store, StoreEvent, StoreEventKind,
};
use serde_json::Value;
use tempfile::TempDir;

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

fn trust(dir: &TempDir, id: &str, reason: &str) -> Vec<u8> {
    dont()
        .args(["trust", id, "--reason", reason, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

// --- Valid transitions ---

#[test]
fn trust_unverified_claim_produces_doubted_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "rust is memory safe");
    let out = trust(&dir, &id, "I have not verified the proofs myself");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["status"], "doubted");
}

#[test]
fn trust_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the earth orbits the sun");
    let out = trust(&dir, &id, "need independent confirmation");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["meta"]["tx"].is_number());
}

#[test]
fn trust_verified_claim_produces_doubted_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "water boils at 100C at sea level");
    // Dismiss first to reach verified
    dont()
        .args(["flag", &id, "--evidence", "https://nist.gov", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    // Now trust the verified claim
    let out = trust(&dir, &id, "altitude matters — needs qualification");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "doubted");
}

#[test]
fn trust_unverified_term_produces_doubted_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");
    let out = trust(&dir, &id, "definition conflicts with observed usage");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["status"], "doubted");
}

// --- Refusals ---

#[test]
fn trust_without_reason_returns_reason_required_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim needing reason");
    let out = dont()
        .args(["trust", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "reason-required");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn trust_already_doubted_returns_invalid_transition_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim to doubt twice");
    trust(&dir, &id, "first doubt");
    let out = dont()
        .args(["trust", &id, "--reason", "second doubt", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn trust_claim_not_found_returns_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args([
            "trust",
            "claim:01JNONEXISTENT",
            "--reason",
            "test",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
}

// --- Hedge refusals ---

#[test]
fn trust_with_hedge_i_think_returns_reason_not_hedge() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "hedged claim");
    let out = dont()
        .args(["trust", &id, "--reason", "I think this is wrong", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "reason-not-hedge");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn trust_hedge_check_is_case_insensitive() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "another hedged claim");
    let out = dont()
        .args([
            "trust",
            &id,
            "--reason",
            "MAYBE this contradicts X",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["code"], "reason-not-hedge");
}

#[test]
fn trust_with_non_hedge_reason_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim with solid reason");
    let out = trust(
        &dir,
        &id,
        "Peer review found a methodological flaw in the cited study",
    );
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "doubted");
}

// --- Locked-entity transition refusals ---

fn seed_verified_claim_with_evidence(dir: &TempDir, claim_id: &str, evidence: &[&str]) {
    let store = Store::open_dont_dir(dir.path()).unwrap();
    let first = evidence.first().expect("at least one evidence item");
    store
        .append_status_change(
            claim_id,
            Status::Unverified,
            Status::Verified,
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

fn lock_claim(dir: &TempDir, id: &str) {
    seed_verified_claim_with_evidence(
        dir,
        id,
        &[
            "https://source-one.example/evidence",
            "https://source-two.example/evidence",
        ],
    );
    seed_assessed_hypotheses(dir, id, 3);
    dont()
        .args(["forget", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn trust_locked_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "A locked claim that cannot be doubted");
    lock_claim(&dir, &id);

    let out = dont()
        .args(["trust", &id, "--reason", "Found a flaw", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
    assert!(
        !v["data"]["message"].as_str().unwrap_or("").is_empty()
            || !v["data"]["remediation"]
                .as_array()
                .unwrap_or(&vec![])
                .is_empty(),
        "response should contain transition context: {v}"
    );
}

#[test]
fn trust_ignored_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "An ignored claim that cannot be doubted");

    dont()
        .args(["ignore", &id, "--reason", "out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args([
            "trust",
            &id,
            "--reason",
            "But actually it matters",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

// Spec (Requirement: Rule outcomes and error taxonomy boundary):
// "Verb-level validators such as reason-required, reason-not-hedge, ... MUST use their own
//  dedicated error codes and MUST set rule_name to null."
#[test]
fn trust_reason_not_hedge_has_null_rule_name() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim for rule_name null check");
    let out = dont()
        .args(["trust", &id, "--reason", "I think this is wrong", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["code"], "reason-not-hedge");
    // Verb-level validators MUST have rule_name: null (not a rule-layer error).
    assert!(
        v["data"]["rule_name"].is_null(),
        "reason-not-hedge must have rule_name: null, got: {}",
        v["data"]["rule_name"]
    );
}

// Spec (Requirement: Rule outcomes and error taxonomy boundary):
// "Verb-level validators such as reason-required ... MUST set rule_name to null."
#[test]
fn trust_reason_required_has_null_rule_name() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim for reason-required null check");
    let out = dont()
        .args(["trust", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["code"], "reason-required");
    // Verb-level validators MUST have rule_name: null (not a rule-layer error).
    assert!(
        v["data"]["rule_name"].is_null(),
        "reason-required must have rule_name: null, got: {}",
        v["data"]["rule_name"]
    );
}
