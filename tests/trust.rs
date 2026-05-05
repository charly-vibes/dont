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
        .args(["dismiss", &id, "--evidence", "https://nist.gov", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    // Now trust the verified claim
    let out = trust(&dir, &id, "altitude matters — needs qualification");
    let v: Value = serde_json::from_slice(&out).unwrap();
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
        .args(["trust", "claim:01JNONEXISTENT", "--reason", "test", "--json"])
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
        .args(["trust", &id, "--reason", "MAYBE this contradicts X", "--json"])
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
    let out = trust(&dir, &id, "Peer review found a methodological flaw in the cited study");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "doubted");
}
