/// dont-errors spec alignment: error envelope structure, code boundaries,
/// remediation invariant, unmet_clauses, and rule_name usage.
mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// --- Structured error envelope ---

#[test]
fn error_envelope_has_required_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "error envelope must have ok=false");
    assert_eq!(
        v["envelope_kind"], "error",
        "error envelope_kind must be 'error'"
    );
    let data = &v["data"];
    assert!(data["code"].is_string(), "data.code must be a string");
    assert!(data["message"].is_string(), "data.message must be a string");
    assert!(
        data["remediation"].is_array(),
        "data.remediation must be an array"
    );
}

// --- Remediation invariant: every error must have at least one remediation ---

#[test]
fn validation_error_has_non_empty_remediation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(
        !remediation.is_empty(),
        "error envelope must contain at least one remediation entry; got empty array"
    );
}

#[test]
fn not_found_error_has_non_empty_remediation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["show", "claim:nonexistent00000000000000", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(
        !remediation.is_empty(),
        "not-found error must contain at least one remediation entry"
    );
}

#[test]
fn remediation_entries_have_command_and_description() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["show", "claim:nonexistent00000000000000", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let remediation = v["data"]["remediation"].as_array().unwrap();
    for entry in remediation {
        assert!(
            entry["command"].is_string(),
            "each remediation entry must have a 'command' field; got: {entry}"
        );
        assert!(
            entry["description"].is_string(),
            "each remediation entry must have a 'description' field; got: {entry}"
        );
    }
}

// --- Verb-level validators have dedicated codes and null rule_name ---

#[test]
fn no_evidence_error_has_dedicated_code_and_null_rule_name() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim for evidence check");

    let out = dont()
        .args(["flag", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["code"], "no-evidence",
        "flag without evidence must use code 'no-evidence', not 'rule-not-met'"
    );
    assert!(
        v["data"]["rule_name"].is_null(),
        "verb-level validator must have null rule_name; got: {:?}",
        v["data"]["rule_name"]
    );
}

#[test]
fn reason_required_error_has_dedicated_code_and_null_rule_name() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim for reason check");

    // trust without --reason should require a reason
    let out = dont()
        .args(["trust", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["code"], "reason-required",
        "trust without --reason must use code 'reason-required', not 'rule-not-met'"
    );
    assert!(
        v["data"]["rule_name"].is_null(),
        "verb-level validator must have null rule_name; got: {:?}",
        v["data"]["rule_name"]
    );
}

// --- rule-not-met always carries rule_name ---

#[test]
fn rule_not_met_error_has_non_null_rule_name() {
    // Flag a claim that depends on an unverified claim — the stale-cascade
    // dependency gate fires and returns rule-not-met with a non-null rule_name.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let dep_id = conclude_claim(&dir, "unverified dependency");

    // Conclude claim B with a dependency on dep_id (still unverified).
    let b_out = dont()
        .args([
            "conclude",
            "claim b depends on a",
            "--depends-on",
            &dep_id,
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let b: Value = serde_json::from_slice(&b_out).unwrap();
    let b_id = b["data"]["id"].as_str().unwrap().to_string();

    // Flag claim B — dep_id is unverified, so the stale-cascade rule fires.
    let out = dont()
        .args(["flag", &b_id, "--evidence", "https://example.com", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["code"], "rule-not-met",
        "dependency gate refusal must use code 'rule-not-met'; got: {:?}",
        v["data"]["code"]
    );
    assert!(
        !v["data"]["rule_name"].is_null(),
        "rule-not-met must have a non-null rule_name; got: {:?}",
        v["data"]["rule_name"]
    );
    let rule_name = v["data"]["rule_name"].as_str().unwrap();
    assert!(
        !rule_name.is_empty(),
        "rule_name must not be empty on rule-not-met; got: {rule_name:?}"
    );
}
