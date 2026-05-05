use dont::envelope::{Envelope, EnvelopeKind, ErrorResult, RemediationEntry, UnmetClause, Warning};
use serde_json::Value;

// --- Envelope<T> construction and serialization ---

#[test]
fn success_envelope_has_required_fields() {
    let env = Envelope::success("version", "1.0.0".to_string(), vec![], vec![]);
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
    let env = Envelope::success("empty", (), vec![], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    let meta = &v["meta"];
    assert!(meta["duration_ms"].is_number());
    assert!(meta["tx"].is_null());
    assert!(meta["request_id"].is_null());
}

#[test]
fn success_envelope_no_hints_key_means_empty() {
    // Hints are always present on success envelopes
    let env = Envelope::success("empty", (), vec![], vec![]);
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
    assert!(v["hints"].is_null() || !v.as_object().unwrap().contains_key("hints"),
        "error envelopes must not carry hints");
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
    assert!(!data["remediation"].as_array().unwrap().is_empty());
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
    assert!(result.is_ok());
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
    let env = Envelope::success("empty", (), vec![w], vec![]);
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
        serde_json::to_string(&EnvelopeKind::SpawnRequest).unwrap(),
        "\"spawn_request\""
    );
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::Error).unwrap(),
        "\"error\""
    );
    assert_eq!(
        serde_json::to_string(&EnvelopeKind::TermList).unwrap(),
        "\"term_list\""
    );
}

// --- Mutating vs read-only meta.tx ---

#[test]
fn mutating_envelope_has_tx_set() {
    let env = Envelope::success_with_tx("empty", (), vec![], vec![], Some(42));
    let v: Value = serde_json::to_value(&env).unwrap();
    assert_eq!(v["meta"]["tx"], 42u64);
}

#[test]
fn readonly_envelope_has_null_tx() {
    let env = Envelope::success("empty", (), vec![], vec![]);
    let v: Value = serde_json::to_value(&env).unwrap();
    assert!(v["meta"]["tx"].is_null());
}
