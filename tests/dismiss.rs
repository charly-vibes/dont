use assert_cmd::Command;
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

fn dismiss(dir: &TempDir, id: &str, evidence: &str) -> Vec<u8> {
    dont()
        .args(["dismiss", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

// --- Valid transitions ---

#[test]
fn dismiss_unverified_claim_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the earth is round");
    let out = dismiss(&dir, &id, "https://nasa.gov/earth-shape");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn dismiss_doubted_claim_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "photosynthesis converts CO2 to O2");
    dont()
        .args(["trust", &id, "--reason", "Need to verify the chemistry", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let out = dismiss(&dir, &id, "https://acs.org/photosynthesis");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn dismiss_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "tx is tracked for mutations");
    let out = dismiss(&dir, &id, "https://example.test/tx-proof");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["meta"]["tx"].is_number());
}

#[test]
fn dismiss_evidence_appears_in_claim_view() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence is tracked");
    let ev = "https://example.test/evidence-1";
    let out = dismiss(&dir, &id, ev);
    let v: Value = serde_json::from_slice(&out).unwrap();
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(
        evidence.iter().any(|e| e.as_str() == Some(ev)),
        "evidence URI missing from claim view"
    );
}

// --- Already-verified evidence append (Phase 8) ---

#[test]
fn dismiss_already_verified_appends_evidence_without_status_change() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence accumulates over time");
    dismiss(&dir, &id, "https://example.test/ev-1");
    // Second dismiss on verified claim
    let out = dismiss(&dir, &id, "https://example.test/ev-2");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "verified");
    // Both evidence URIs should appear
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let uris: Vec<&str> = evidence.iter().filter_map(|e| e.as_str()).collect();
    assert!(uris.contains(&"https://example.test/ev-1"), "first evidence missing");
    assert!(uris.contains(&"https://example.test/ev-2"), "second evidence missing");
}

// --- Refusals ---

#[test]
fn dismiss_without_evidence_returns_no_evidence_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence-free dismiss should fail");
    let out = dont()
        .args(["dismiss", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn dismiss_claim_not_found_returns_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["dismiss", "claim:01JNONEXISTENT", "--evidence", "https://example.test", "--json"])
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
