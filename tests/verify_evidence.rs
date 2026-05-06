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

fn dismiss_claim(dir: &TempDir, id: &str, evidence: &str) {
    dont()
        .args(["dismiss", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn verify_evidence(dir: &TempDir, id: &str, mock: &str) -> Value {
    let out = dont()
        .args(["verify-evidence", id, "--json"])
        .env("DONT_DIR", dir.path())
        .env("DONT_VERIFY_EVIDENCE_MOCK", mock)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

#[test]
fn verify_evidence_reports_per_reference_results_without_changing_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence should remain checkable");
    dismiss_claim(&dir, &id, "https://example.test/evidence-1");

    let v = verify_evidence(
        &dir,
        &id,
        r#"{"https://example.test/evidence-1":{"outcome":"reachable"}}"#,
    );

    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["entity_id"], id);
    assert_eq!(v["data"]["status"], "verified");
    assert_eq!(
        v["data"]["results"][0]["uri"],
        "https://example.test/evidence-1"
    );
    assert_eq!(v["data"]["results"][0]["outcome"], "reachable");
    assert!(v["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn verify_evidence_returns_partial_results_on_timeout() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "timeouts should not abort all evidence checks");
    dismiss_claim(&dir, &id, "https://example.test/evidence-ok");
    dismiss_claim(&dir, &id, "https://example.test/evidence-timeout");

    let v = verify_evidence(
        &dir,
        &id,
        r#"{
            "https://example.test/evidence-ok":{"outcome":"reachable"},
            "https://example.test/evidence-timeout":{"outcome":"timeout","detail":"timed out after 2s"}
        }"#,
    );

    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["results"].as_array().unwrap().len(), 2);
    assert_eq!(v["data"]["results"][1]["outcome"], "timeout");
    assert!(v["warnings"].as_array().unwrap().iter().any(|warning| {
        warning["rule_name"] == "evidence-timeout"
            && warning["message"].as_str().unwrap().contains("timed out")
    }));
}

#[test]
fn verify_evidence_warns_on_malformed_or_unreachable_references() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "bad evidence should surface warnings");
    dismiss_claim(&dir, &id, "not-a-uri");
    dismiss_claim(&dir, &id, "https://example.test/offline");

    let v = verify_evidence(
        &dir,
        &id,
        r#"{
            "not-a-uri":{"outcome":"malformed","detail":"missing URI scheme"},
            "https://example.test/offline":{"outcome":"unreachable","detail":"HTTP 503"}
        }"#,
    );

    assert_eq!(v["ok"], true);
    let warnings = v["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|warning| warning["rule_name"] == "evidence-malformed")
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning["rule_name"] == "evidence-unreachable")
    );
}

#[test]
fn verify_evidence_refuses_targets_without_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claims without evidence should fail structurally");

    let out = dont()
        .args(["verify-evidence", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
}

#[test]
fn verify_evidence_refuses_unknown_target() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["verify-evidence", "claim:NOTEXIST", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "entity-not-found");
}
