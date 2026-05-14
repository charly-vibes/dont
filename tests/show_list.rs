mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn define_term(dir: &TempDir, curie: &str, doc: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", doc, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<Value>(&out).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

// --- show ---

#[test]
fn show_returns_claim_view_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the moon orbits the earth");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["id"], id.as_str());
    assert_eq!(v["data"]["status"], "unverified");
    assert_eq!(v["data"]["statement"], "the moon orbits the earth");
}

#[test]
fn show_claim_view_has_required_arrays_and_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "required arrays test");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    assert!(data["derived_assessments"].is_array());
    assert!(data["atoms"].is_array());
    assert!(data["hypotheses"].is_array());
    assert!(data["evidence"].is_array());
    assert!(data["depends_on"].is_array());
    assert!(data["applicable_rules"].is_object());
    assert!(data["created_at"].is_string());
    assert!(data["updated_at"].is_string());
}

#[test]
fn show_reflects_current_status_after_trust() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim to be trusted");
    dont()
        .args(["trust", &id, "--reason", "Source has conflicts of interest", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "doubted");
}

#[test]
fn show_evidence_reflects_dismiss_history() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim with evidence");
    let ev = "https://example.test/proof";
    dont()
        .args(["flag", &id, "--evidence", ev, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(evidence.iter().any(|e| e.as_str() == Some(ev)));
}

#[test]
fn show_nonexistent_id_returns_claim_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["show", "claim:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

// --- show: CURIE resolution ---

#[test]
fn show_by_curie_resolves_to_term() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001", "a process by which X becomes Y");

    let out = dont()
        .args(["show", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["curie"], "WB:P001");
    assert_eq!(v["data"]["entity_kind"], "term");
}

#[test]
fn show_unknown_curie_returns_term_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["show", "WB:ZZZZ", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-not-found");
    let msg = v["data"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("WB:ZZZZ"), "expected curie in error: {msg}");
    assert!(msg.contains("no term with curie"), "expected curie phrasing: {msg}");
}

// --- list ---

#[test]
fn list_returns_claims_envelope_kind() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "first claim");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claims");
    assert!(v["data"]["claims"].is_array());
}

#[test]
fn list_returns_all_concluded_claims() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "alpha claim");
    let id2 = conclude_claim(&dir, "beta claim");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let claims = v["data"]["claims"].as_array().unwrap();
    let ids: Vec<&str> = claims.iter().filter_map(|c| c["id"].as_str()).collect();
    assert!(ids.contains(&id1.as_str()), "alpha claim missing");
    assert!(ids.contains(&id2.as_str()), "beta claim missing");
}

#[test]
fn list_claims_sorted_by_created_at_descending() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "earliest claim");
    let id2 = conclude_claim(&dir, "latest claim");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let claims = v["data"]["claims"].as_array().unwrap();
    assert!(claims.len() >= 2);
    // Most recent first
    let ids: Vec<&str> = claims.iter().filter_map(|c| c["id"].as_str()).collect();
    let pos1 = ids.iter().position(|&i| i == id1.as_str()).unwrap();
    let pos2 = ids.iter().position(|&i| i == id2.as_str()).unwrap();
    assert!(pos2 < pos1, "latest claim should appear before earliest");
}

#[test]
fn list_empty_project_returns_empty_array() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claims");
    assert_eq!(v["data"]["claims"].as_array().unwrap().len(), 0);
}

#[test]
fn list_status_filter_returns_only_matching_claims() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let unverified = conclude_claim(&dir, "still unverified");
    let doubted = conclude_claim(&dir, "will be doubted");
    let verified = conclude_claim(&dir, "will be verified");
    let ignored = conclude_claim(&dir, "will be ignored");

    dont()
        .args(["trust", &doubted, "--reason", "Conflicts with source evidence", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    dont()
        .args(["flag", &verified, "--evidence", "https://example.test/proof", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    dont()
        .args(["ignore", &ignored, "--reason", "Out of scope for this project", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    for (status, expected_id) in [
        ("unverified", unverified.as_str()),
        ("doubted", doubted.as_str()),
        ("verified", verified.as_str()),
        ("ignored", ignored.as_str()),
    ] {
        let out = dont()
            .args(["list", "--status", status, "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&out).unwrap();
        let claims = v["data"]["claims"].as_array().unwrap();
        assert_eq!(claims.len(), 1, "status={status}");
        assert_eq!(claims[0]["id"], expected_id, "status={status}");
        assert_eq!(claims[0]["status"], status, "status={status}");
    }
}

#[test]
fn list_invalid_status_returns_validation_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--status", "pending", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-status");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn list_kind_terms_returns_defined_terms() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let term_id = define_term(&dir, "WB:P001", "a process by which X becomes Y");
    conclude_claim(&dir, "claims should not appear in term listings");

    let out = dont()
        .args(["list", "--kind", "terms", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["envelope_kind"], "terms");
    let terms = v["data"].as_array().unwrap();
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0]["id"], term_id);
    assert_eq!(terms[0]["entity_kind"], "term");
    assert_eq!(terms[0]["curie"], "WB:P001");
    assert_eq!(terms[0]["status"], "unverified");
}

#[test]
fn list_kind_claims_retains_current_behavior() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001", "a process by which X becomes Y");
    let claim_id = conclude_claim(&dir, "claims remain the default listing kind");

    let out = dont()
        .args(["list", "--kind", "claims", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["envelope_kind"], "claims");
    let claims = v["data"]["claims"].as_array().unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0]["id"], claim_id);
    assert_eq!(claims[0]["entity_kind"], "claim");
}

#[test]
fn list_default_claims_emits_hint_when_terms_exist() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001", "a process by which X becomes Y");
    conclude_claim(&dir, "default listing still shows claims");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["envelope_kind"], "claims");
    assert!(v["hints"].as_array().unwrap().iter().any(|hint| {
        hint["command"] == "dont list --kind terms"
    }));
}

#[test]
fn list_kind_terms_supports_status_filter() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let unverified = define_term(&dir, "WB:P001", "a process by which X becomes Y");
    let ignored = define_term(&dir, "WB:P002", "a process by which Y becomes Z");
    dont()
        .args(["ignore", &ignored, "--reason", "Out of scope for this project", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["list", "--kind", "terms", "--status", "ignored", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let terms = v["data"].as_array().unwrap();
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0]["id"], ignored);
    assert_eq!(terms[0]["status"], "ignored");
    assert_ne!(terms[0]["id"], unverified);
}

#[test]
fn list_invalid_kind_returns_validation_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--kind", "events", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-kind");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}
