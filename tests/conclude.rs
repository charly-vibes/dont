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

fn conclude_in(dir: &TempDir, statement: &str) -> Vec<u8> {
    dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

// --- Successful conclude ---

#[test]
fn conclude_returns_claim_envelope_with_ok_true() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "all bachelors are unmarried");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_version"], "0.2");
    assert_eq!(v["envelope_kind"], "claim");
}

#[test]
fn conclude_creates_claim_with_unverified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "water is H2O");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn conclude_claim_has_prefixed_ulid_id() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "entropy always increases");
    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap();
    assert!(id.starts_with("claim:"), "id should have claim: prefix, got {id}");
}

#[test]
fn conclude_claim_view_has_required_arrays() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "test claim");
    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    assert!(data["derived_assessments"].is_array(), "derived_assessments");
    assert!(data["atoms"].is_array(), "atoms");
    assert!(data["hypotheses"].is_array(), "hypotheses");
    assert!(data["evidence"].is_array(), "evidence");
    assert!(data["depends_on"].is_array(), "depends_on");
    assert!(data["applicable_rules"].is_object(), "applicable_rules");
}

#[test]
fn conclude_populates_statement_in_claim_view() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let statement = "the sky is blue during clear weather";
    let out = conclude_in(&dir, statement);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["statement"], statement);
}

#[test]
fn conclude_envelope_has_tx_set_for_mutation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "mutations carry a tx");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["meta"]["tx"].is_number(), "mutation must have non-null tx");
}

#[test]
fn conclude_persists_claim_across_invocations() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "gravity attracts masses");
    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap().to_string();

    // A second invocation produces a different claim id
    let out2 = conclude_in(&dir, "gravity attracts masses");
    let v2: Value = serde_json::from_slice(&out2).unwrap();
    let id2 = v2["data"]["id"].as_str().unwrap();
    assert_ne!(id, id2, "each conclude creates a distinct claim");
}

// --- Conclude outside project ---

#[test]
fn conclude_outside_project_exits_3_with_config_missing() {
    let dir = TempDir::new().unwrap();
    // No init — DONT_DIR points to nonexistent dir
    let out = dont()
        .args(["conclude", "some claim", "--json"])
        .env("DONT_DIR", dir.path().join("nonexistent"))
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "config-missing");
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty());
    // Remediation should mention 'dont init'
    let rem_str = serde_json::to_string(remediation).unwrap();
    assert!(rem_str.contains("dont init"), "remediation must mention dont init");
}
