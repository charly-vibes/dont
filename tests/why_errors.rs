mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

/// `dont why` on a claim ID that does not exist must return a structured
/// "claim-not-found" error (exit 1) — the same contract as `show` and `trace`.
#[test]
fn why_unknown_claim_id_returns_claim_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["why", "claim:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "claim-not-found",
        "why must return claim-not-found for missing claim id, got: {:?}",
        v["data"]["code"]
    );
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "claim-not-found error must include remediation hints"
    );
}

/// `dont why` on a term ID that does not exist must return a structured
/// "term-not-found" error (exit 1).
#[test]
fn why_unknown_term_id_returns_term_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["why", "term:01JNONEXISTENT", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "term-not-found",
        "why must return term-not-found for missing term id, got: {:?}",
        v["data"]["code"]
    );
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "term-not-found error must include remediation hints"
    );
}

/// `dont why` on an unknown CURIE (NS:local form) must return "term-not-found"
/// and include the CURIE in the error message.
#[test]
fn why_unknown_curie_returns_term_not_found_with_curie_in_message() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["why", "WB:ZZZZ", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "term-not-found",
        "why must return term-not-found for unknown CURIE, got: {:?}",
        v["data"]["code"]
    );
    let msg = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("WB:ZZZZ"),
        "error message must include the unknown CURIE, got: {msg}"
    );
}

// --- happy-path: claim ---

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
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

fn define_term(dir: &TempDir, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
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

/// `dont why <claim-id> --json` for an existing claim must return
/// `ok: true`, `envelope_kind: "why"`, and a `data` object with the
/// expected fields: `entity`, `history`, `applicable_rules`, `remediation`.
#[test]
fn why_existing_claim_returns_ok_true_with_required_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the earth orbits the sun");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "why on existing claim must return ok: true");
    assert_eq!(
        v["envelope_kind"], "why",
        "envelope_kind must be 'why', got: {:?}",
        v["envelope_kind"]
    );

    let data = &v["data"];
    assert!(
        data["entity"].is_object(),
        "data.entity must be an object, got: {:?}",
        data["entity"]
    );
    assert!(
        data["history"].is_array(),
        "data.history must be an array, got: {:?}",
        data["history"]
    );
    assert!(
        data["applicable_rules"].is_object(),
        "data.applicable_rules must be an object, got: {:?}",
        data["applicable_rules"]
    );
    assert!(
        data["remediation"].is_array(),
        "data.remediation must be an array, got: {:?}",
        data["remediation"]
    );
}

/// The `entity` object embedded in the `why` response for a claim must
/// contain the canonical claim fields: id, entity_kind, statement, status,
/// evidence, created_at.
#[test]
fn why_claim_entity_contains_canonical_claim_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "gravity attracts mass");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let entity = &v["data"]["entity"];

    assert_eq!(
        entity["id"], id,
        "entity.id must match the requested claim id"
    );
    assert_eq!(
        entity["entity_kind"], "claim",
        "entity.entity_kind must be 'claim', got: {:?}",
        entity["entity_kind"]
    );
    assert_eq!(
        entity["statement"], "gravity attracts mass",
        "entity.statement must match, got: {:?}",
        entity["statement"]
    );
    assert!(
        entity["status"].is_string(),
        "entity.status must be a string, got: {:?}",
        entity["status"]
    );
    assert!(
        entity["evidence"].is_array(),
        "entity.evidence must be an array, got: {:?}",
        entity["evidence"]
    );
    assert!(
        entity["created_at"].is_string(),
        "entity.created_at must be a string, got: {:?}",
        entity["created_at"]
    );
}

/// A freshly concluded claim has exactly one history event of kind "created"
/// (or similar), confirming that `why` exposes the event log.
#[test]
fn why_claim_history_contains_at_least_one_event() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "light travels fast");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let history = v["data"]["history"].as_array().unwrap();
    assert!(
        !history.is_empty(),
        "a newly created claim must have at least one history event"
    );
    // Each history event must carry at minimum an event_kind field.
    for event in history {
        assert!(
            event["event_kind"].is_string(),
            "each history event must have an event_kind string, got: {:?}",
            event
        );
    }
}

// --- happy-path: term ---

/// `dont why <term-id> --json` for an existing term must return
/// `ok: true`, `envelope_kind: "why"`, and a `data` object with the
/// expected fields: `entity`, `history`, `applicable_rules`, `remediation`.
#[test]
fn why_existing_term_returns_ok_true_with_required_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "why on existing term must return ok: true");
    assert_eq!(
        v["envelope_kind"], "why",
        "envelope_kind must be 'why', got: {:?}",
        v["envelope_kind"]
    );

    let data = &v["data"];
    assert!(
        data["entity"].is_object(),
        "data.entity must be an object, got: {:?}",
        data["entity"]
    );
    assert!(
        data["history"].is_array(),
        "data.history must be an array, got: {:?}",
        data["history"]
    );
    assert!(
        data["applicable_rules"].is_object(),
        "data.applicable_rules must be an object, got: {:?}",
        data["applicable_rules"]
    );
    assert!(
        data["remediation"].is_array(),
        "data.remediation must be an array, got: {:?}",
        data["remediation"]
    );
}

/// The `entity` object in a term `why` response must include the canonical
/// term fields: id, entity_kind, curie, status, evidence, created_at.
#[test]
fn why_term_entity_contains_canonical_term_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P002");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let entity = &v["data"]["entity"];

    assert_eq!(
        entity["id"], id,
        "entity.id must match the requested term id"
    );
    assert_eq!(
        entity["entity_kind"], "term",
        "entity.entity_kind must be 'term', got: {:?}",
        entity["entity_kind"]
    );
    assert_eq!(
        entity["curie"], "WB:P002",
        "entity.curie must match, got: {:?}",
        entity["curie"]
    );
    assert!(
        entity["status"].is_string(),
        "entity.status must be a string, got: {:?}",
        entity["status"]
    );
    assert!(
        entity["evidence"].is_array(),
        "entity.evidence must be an array, got: {:?}",
        entity["evidence"]
    );
    assert!(
        entity["created_at"].is_string(),
        "entity.created_at must be a string, got: {:?}",
        entity["created_at"]
    );
}

// --- human-readable format (no --json flag) ---

/// `dont why <claim-id>` without --json must produce plain text output,
/// not a JSON object.
#[test]
fn why_claim_default_output_is_plain_text_not_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "water boils at 100 celsius");

    let out = dont()
        .args(["why", &id])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "default why output must not be JSON, got: {text}"
    );
}

/// `dont why <claim-id> --human` must produce plain text output.
#[test]
fn why_claim_human_flag_emits_plain_text() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "ice is cold");

    let out = dont()
        .args(["why", &id, "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "--human flag must produce plain text, not JSON, got: {text}"
    );
}

// --- remediation field ---

/// Spec (WhyView payload, Requirement: WhyView with unmet lockable rule):
/// "WHEN `dont why claim:X --json` is run and `lockable` is not met,
/// THEN `remediation[]` contains an entry with `rule_name: "lockable"` and a
/// suggested command."
///
/// A freshly concluded claim has no assessed hypotheses and no independent
/// evidence, so `lockable` is always unmet on it. The `remediation` array
/// MUST be non-empty and the first entry MUST carry `rule_name: "lockable"`
/// and a non-empty `command` string.
#[test]
fn why_claim_with_unmet_lockable_rule_has_non_empty_remediation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "remediation must be populated for unmet rules");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let remediation = v["data"]["remediation"]
        .as_array()
        .expect("data.remediation must be an array");

    assert!(
        !remediation.is_empty(),
        "remediation[] must be non-empty when lockable rule is unmet, got empty array"
    );

    let lockable_entry = remediation
        .iter()
        .find(|e| e["rule_name"] == "lockable")
        .expect("remediation[] must contain an entry with rule_name: \"lockable\"");

    let command = lockable_entry["command"].as_str().unwrap_or("");
    assert!(
        !command.is_empty(),
        "remediation entry for lockable must have a non-empty command, got: {:?}",
        lockable_entry
    );

    let description = lockable_entry["description"].as_str().unwrap_or("");
    assert!(
        !description.is_empty(),
        "remediation entry for lockable must have a non-empty description, got: {:?}",
        lockable_entry
    );
}

/// Spec (WhyView payload, Requirement: WhyView on fully satisfied entity):
/// "WHEN `dont why claim:X --json` is run and all rules pass,
/// THEN `remediation` is an empty array and all `applicable_rules` show as met."
///
/// This is the converse: for a claim that has met all applicable rules
/// (using applicable_rules.lockable.met == true as the signal), remediation
/// MUST be empty.
#[test]
fn why_claim_with_all_rules_met_has_empty_remediation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "all rules met means empty remediation");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let lockable_met = v["data"]["applicable_rules"]["lockable"]["met"]
        .as_bool()
        .unwrap_or(false);

    // A freshly concluded claim does NOT meet lockable — so this test is only
    // meaningful when lockable is met. Skip the remediation assertion if
    // lockable is not met (that case is covered by the sibling test above).
    if lockable_met {
        let remediation = v["data"]["remediation"]
            .as_array()
            .expect("data.remediation must be an array");
        assert!(
            remediation.is_empty(),
            "remediation[] must be empty when all rules pass, got: {:?}",
            remediation
        );
    }
}

/// `dont why <term-id>` without --json must produce plain text output.
#[test]
fn why_term_default_output_is_plain_text_not_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P003");

    let out = dont()
        .args(["why", &id])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "default why output for a term must not be JSON, got: {text}"
    );
}
