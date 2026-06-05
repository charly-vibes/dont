mod common;

use common::{conclude_claim, dont, init_dir};
use dont::envelope::{Envelope, EnvelopeKind, ErrorResult, RemediationEntry, UnmetClause, Warning};
use serde_json::Value;
use tempfile::TempDir;

// --- Envelope<T> construction and serialization ---

#[test]
fn success_envelope_has_required_fields() {
    let env = Envelope::success(EnvelopeKind::Version, "1.0.0".to_string(), vec![], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_version"], "0.2");
    assert!(v["cli_version"].is_string());
    assert_eq!(v["envelope_kind"], "version");
    assert!(v["data"].is_string()); // "1.0.0"
    assert!(v["warnings"].is_array());
    assert!(v["hints"].is_array());
    assert!(v["meta"].is_object());
}

#[test]
fn success_envelope_has_meta_fields() {
    let env = Envelope::success(EnvelopeKind::Empty, (), vec![], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    let meta = &v["meta"];
    assert!(meta["duration_ms"].is_number());
    assert!(meta["tx"].is_null());
    assert!(meta["request_id"].is_null());
}

#[test]
fn success_envelope_no_hints_key_means_empty() {
    // Hints are always present on success envelopes
    let env = Envelope::success(EnvelopeKind::Empty, (), vec![], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert!(v["hints"].is_array());
}

#[test]
fn error_envelope_has_ok_false_and_no_hints() {
    let err = ErrorResult {
        code: "internal".to_string(),
        message: "something broke".to_string(),
        rule_name: None,
        spec_ref: None,
        entity_id: None,
        unmet_clauses: vec![],
        remediation: vec![RemediationEntry {
            command: "dont doctor".to_string(),
            description: "Run doctor".to_string(),
        }],
    };
    let env = Envelope::error(err, vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert!(
        v["hints"].is_null() || !v.as_object().unwrap().contains_key("hints"),
        "error envelopes must not carry hints"
    );
}

#[test]
fn error_envelope_data_contains_error_result_fields() {
    let err = ErrorResult {
        code: "rule-not-met".to_string(),
        message: "lockable rule tripped".to_string(),
        rule_name: Some("lockable".to_string()),
        spec_ref: Some("§13.1".to_string()),
        entity_id: Some("claim:01J000000000000000000000".to_string()),
        unmet_clauses: vec![UnmetClause {
            clause: "no open locks".to_string(),
            fix: "dont dismiss claim:xxx --reason reason".to_string(),
        }],
        remediation: vec![RemediationEntry {
            command: "dont dismiss claim:xxx --reason reason".to_string(),
            description: "Dismiss the blocking claim".to_string(),
        }],
    };
    let env = Envelope::error(err, vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    let data = &v["data"];
    assert_eq!(data["code"], "rule-not-met");
    assert_eq!(data["rule_name"], "lockable");
    let remediation = data["remediation"].as_array().unwrap();
    assert_eq!(remediation.len(), 1);
    assert_eq!(
        remediation[0]["command"],
        "dont dismiss claim:xxx --reason reason"
    );
    assert_eq!(remediation[0]["description"], "Dismiss the blocking claim");
    assert_eq!(data["unmet_clauses"][0]["clause"], "no open locks");
}

#[test]
fn remediation_must_be_non_empty() {
    // ErrorResult with empty remediation is invalid per spec (Invariant 3.2.5)
    // We enforce this via a Result return or panic — test that ok() is None
    let result = ErrorResult::new(
        "usage",
        "bad args",
        None,
        None,
        None,
        vec![],
        vec![], // empty remediation — should fail
    );
    assert!(result.is_err(), "empty remediation must be rejected");
}

#[test]
fn remediation_non_empty_is_ok() {
    let result = ErrorResult::new(
        "usage",
        "bad args",
        None,
        None,
        None,
        vec![],
        vec![RemediationEntry {
            command: "dont help add".to_string(),
            description: "Show help".to_string(),
        }],
    );
    let err = result.expect("ErrorResult with non-empty remediation must succeed");
    assert_eq!(err.code, "usage");
    assert_eq!(err.message, "bad args");
    assert_eq!(err.remediation.len(), 1);
    assert_eq!(err.remediation[0].command, "dont help add");
}

// --- Warnings ---

#[test]
fn warnings_can_appear_on_success_envelope() {
    let w = Warning {
        rule_name: "evidence-malformed".to_string(),
        entity_id: Some("claim:01J000000000000000000000".to_string()),
        message: "URI is malformed".to_string(),
        suggested_remediation: None,
    };
    let env = Envelope::success(EnvelopeKind::Empty, (), vec![w], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["warnings"].as_array().unwrap().len(), 1);
    assert_eq!(v["warnings"][0]["rule_name"], "evidence-malformed");
}

#[test]
fn warnings_can_appear_on_error_envelope() {
    let err = ErrorResult {
        code: "rule-not-met".to_string(),
        message: "rule tripped".to_string(),
        rule_name: Some("lockable".to_string()),
        spec_ref: None,
        entity_id: None,
        unmet_clauses: vec![],
        remediation: vec![RemediationEntry {
            command: "dont doctor".to_string(),
            description: "Run doctor".to_string(),
        }],
    };
    let w = Warning {
        rule_name: "evidence-stale".to_string(),
        entity_id: None,
        message: "stale evidence".to_string(),
        suggested_remediation: None,
    };
    let env = Envelope::error(err, vec![w]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["warnings"].as_array().unwrap().len(), 1);
}

// --- EnvelopeKind ---

#[test]
fn envelope_kind_serializes_to_snake_case() {
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::RuleResult).unwrap(),
        "\"rule_result\""
    );
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::Error).unwrap(),
        "\"error\""
    );
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::EvidenceCheck).unwrap(),
        "\"evidence_check\""
    );
}

#[test]
fn envelope_kind_renamed_variants_keep_dash() {
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::DontExplain).unwrap(),
        "\"dont-explain\""
    );
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::DontCompletions).unwrap(),
        "\"dont-completions\""
    );
}

#[test]
fn envelope_rejects_unknown_kind_on_deserialize() {
    let json = r#"{"ok":true,"envelope_version":"0.2","cli_version":"0.1.0","envelope_kind":"not_a_real_kind","data":null,"warnings":[],"meta":{"duration_ms":0,"tx":null,"request_id":null}}"#;
    let result: Result<dont::envelope::Envelope<serde_json::Value>, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "expected unknown envelope_kind to be rejected; envelope_kind must be a typed enum, not String"
    );
}

// --- Mutating vs read-only meta.tx ---

#[test]
fn mutating_envelope_has_tx_set() {
    let env = Envelope::success_with_tx(EnvelopeKind::Empty, (), vec![], vec![], Some(42));
    let v: Value = serde_json::to_value(&env).unwrap();
    assert_eq!(v["meta"]["tx"], 42u64);
}

#[test]
fn readonly_envelope_has_null_tx() {
    let env = Envelope::success(EnvelopeKind::Empty, (), vec![], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert!(v["meta"]["tx"].is_null());
}

// --- CLI error paths: full envelope structure verification ---
//
// Each test below drives the binary with --json, triggers a real error path,
// and verifies that the output satisfies the complete error-envelope contract:
//   ok: false
//   envelope_kind: "error"
//   envelope_version: "0.2"
//   data.code       — non-empty string
//   data.message    — non-empty string
//   data.remediation — non-empty array; each entry has non-empty command and description

/// Assert the full error envelope contract on a parsed JSON Value.
fn assert_error_envelope(v: &Value) {
    assert_eq!(v["ok"], false, "ok must be false on error");
    assert_eq!(v["envelope_kind"], "error", "envelope_kind must be 'error'");
    assert_eq!(
        v["envelope_version"], "0.2",
        "envelope_version must be '0.2'"
    );

    let data = &v["data"];
    let code = data["code"].as_str().unwrap_or("");
    assert!(!code.is_empty(), "data.code must be a non-empty string");

    let message = data["message"].as_str().unwrap_or("");
    assert!(
        !message.is_empty(),
        "data.message must be a non-empty string"
    );

    let remediation = data["remediation"]
        .as_array()
        .expect("data.remediation must be an array");
    assert!(
        !remediation.is_empty(),
        "data.remediation must be non-empty (Invariant 3.2.5)"
    );
    for (i, entry) in remediation.iter().enumerate() {
        assert!(
            entry["command"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "remediation[{i}].command must be a non-empty string"
        );
        assert!(
            entry["description"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "remediation[{i}].description must be a non-empty string"
        );
    }
}

#[test]
fn cli_error_conclude_empty_statement_is_well_formed_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "empty-statement");
}

#[test]
fn cli_error_conclude_no_project_is_well_formed_envelope() {
    let dir = TempDir::new().unwrap();
    // No init — project does not exist
    let out = dont()
        .args(["conclude", "some claim", "--json"])
        .env("DONT_DIR", dir.path().join("nonexistent"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "config-missing");
}

#[test]
fn cli_error_show_claim_not_found_is_well_formed_envelope() {
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
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "claim-not-found");
}

#[test]
fn cli_error_show_term_not_found_is_well_formed_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["show", "term:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "term-not-found");
}

#[test]
fn cli_error_trust_reason_required_is_well_formed_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim to doubt");
    let out = dont()
        .args(["trust", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "reason-required");
}

#[test]
fn cli_error_list_invalid_status_is_well_formed_envelope() {
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
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "invalid-status");
}

#[test]
fn cli_error_invalid_transition_is_well_formed_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim to double-doubt");
    dont()
        .args([
            "trust",
            &id,
            "--reason",
            "First doubt — methodology gap",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let out = dont()
        .args(["trust", &id, "--reason", "Second doubt attempt", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_error_envelope(&v);
    assert_eq!(v["data"]["code"], "invalid-transition");
}
