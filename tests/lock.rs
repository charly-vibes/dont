use assert_cmd::Command;
use dont::store::{HypothesisAssessment, HypothesisRecord, Store};
use serde_json::Value;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_dir(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
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
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn dismiss_claim(dir: &TempDir, id: &str, evidence: &str) {
    dont()
        .args(["dismiss", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
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
    store.set_claim_hypotheses_for_test(claim_id, &hypotheses).unwrap();
}

#[test]
fn lock_verified_claim_with_sufficient_hypotheses_and_evidence_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Independent evidence converges on this claim");
    dismiss_claim(&dir, &id, "https://source-one.example/evidence");
    dismiss_claim(&dir, &id, "https://source-two.example/evidence");
    seed_assessed_hypotheses(&dir, &id, 3);

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "locked");
    assert_eq!(v["data"]["id"], id);
    assert_eq!(v["data"]["hypotheses"].as_array().unwrap().len(), 3);
}

#[test]
fn lock_unverified_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "This claim is not verified yet");

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-verified");
}

#[test]
fn lock_already_locked_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "This claim will be locked once");
    dismiss_claim(&dir, &id, "https://source-one.example/evidence");
    dismiss_claim(&dir, &id, "https://source-two.example/evidence");
    seed_assessed_hypotheses(&dir, &id, 3);

    dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-locked");
}

#[test]
fn lock_claim_with_too_few_hypotheses_is_refused_by_lockable_gate() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "This claim lacks enough hypotheses");
    dismiss_claim(&dir, &id, "https://source-one.example/evidence");
    dismiss_claim(&dir, &id, "https://source-two.example/evidence");
    seed_assessed_hypotheses(&dir, &id, 2);

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-met");
    assert_eq!(v["data"]["rule_name"], "lockable");
    assert!(v["data"]["unmet_clauses"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["clause"].as_str().unwrap().contains("hypotheses")));
}

#[test]
fn lock_claim_with_too_little_evidence_is_refused_by_lockable_gate() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "This claim lacks enough evidence");
    dismiss_claim(&dir, &id, "https://source-one.example/evidence");
    seed_assessed_hypotheses(&dir, &id, 3);

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-met");
    assert_eq!(v["data"]["rule_name"], "lockable");
    assert!(v["data"]["unmet_clauses"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["clause"].as_str().unwrap().contains("evidence")));
}

#[test]
fn lock_claim_with_unverified_term_dependency_is_refused_by_lockable_gate() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001");
    let out = dont()
        .args([
            "conclude",
            "This claim depends on WB:P001",
            "--depends-on",
            "WB:P001",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap().to_string();
    dismiss_claim(&dir, &id, "https://source-one.example/evidence");
    dismiss_claim(&dir, &id, "https://source-two.example/evidence");
    seed_assessed_hypotheses(&dir, &id, 3);

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-met");
    assert_eq!(v["data"]["rule_name"], "lockable");
    assert!(v["data"]["unmet_clauses"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["clause"].as_str().unwrap().contains("stale")));
}

#[test]
fn lock_term_is_refused_as_wrong_entity_kind() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");

    let output = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "wrong-entity-kind");
}
