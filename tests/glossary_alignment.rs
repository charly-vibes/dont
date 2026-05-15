/// Spec-alignment tests for dont-glossary (ticket dont-8169).
///
/// The glossary spec defines:
/// - Core four verbs: conclude, define, trust, dismiss
/// - Lifecycle verbs: lock, reopen, ignore, verify-evidence
///
/// These tests assert that the CLI exposes `dismiss` and `lock` under those
/// canonical names so that external callers, documentation, and LLM harnesses
/// can rely on the spec-defined vocabulary.
mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// ── Core four verb: dismiss ──────────────────────────────────────────────────

/// The glossary spec (Requirement: Core four verbs) names the fourth primary
/// verb as `dismiss`.  The implementation must accept `dont dismiss` at the
/// top level and use it to verify a claim with evidence.
#[test]
fn dismiss_is_a_top_level_verb_that_verifies_a_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "all bachelors are unmarried");

    let out = dont()
        .args(["dismiss", &id, "--evidence", "https://example.com/ref", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "dismiss should succeed");
    assert_eq!(
        v["data"]["status"], "verified",
        "dismiss should transition claim to verified"
    );
}

/// The spec help text should describe `dismiss` as verifying (grounding) an
/// entity, so that the glossary definition of "dismiss" is discoverable from
/// `--help`.
#[test]
fn dismiss_appears_in_top_level_help() {
    dont()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("dismiss"));
}

// ── Lifecycle verb: lock ─────────────────────────────────────────────────────

/// The glossary spec (Requirement: Lifecycle verb) names the terminal-promotion
/// lifecycle verb as `lock`.  The implementation must accept `dont lock` at the
/// top level and use it to permanently preserve a verified claim.
///
/// NOTE: The current implementation exposes this operation as `forget` and
/// actively rejects `lock`.  This test documents the required spec behaviour
/// and is expected to fail until `lock` is restored as a supported alias or
/// primary command.
#[test]
fn lock_is_a_top_level_lifecycle_verb_that_locks_a_verified_claim() {
    use dont::store::{
        HypothesisAssessment, HypothesisRecord, Store, StoreEvent, StoreEventKind, StoreStatus,
    };

    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "photons have no rest mass");

    // Seed two independent evidence sources directly (mirrors lock.rs helper)
    let store = Store::open_dont_dir(dir.path()).unwrap();
    for uri in &[
        "https://source-one.example/evidence",
        "https://source-two.example/evidence",
    ] {
        store
            .append_status_change(
                &id,
                StoreStatus::Unverified,
                StoreStatus::Verified,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![serde_json::Value::String((*uri).to_string())],
                },
            )
            .ok(); // second call may fail because status is already Verified; that's fine
        store
            .append_evidence_event(
                &id,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![serde_json::Value::String((*uri).to_string())],
                },
            )
            .ok();
    }

    // Seed three assessed hypotheses
    let hypotheses: Vec<HypothesisRecord> = (0..3)
        .map(|idx| HypothesisRecord {
            idx,
            text: format!("hypothesis {}", idx + 1),
            assessment: HypothesisAssessment {
                supporting: vec![format!("support-{}", idx + 1)],
                refuting: vec![],
            },
        })
        .collect();
    store.set_claim_hypotheses_for_test(&id, &hypotheses).unwrap();

    let out = dont()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "lock should succeed on a verified claim");
    assert_eq!(
        v["data"]["status"], "locked",
        "lock should transition claim to locked"
    );
}

/// The spec help text should list `lock` as a subcommand, so that the
/// glossary lifecycle-verb definition is discoverable from `--help`.
/// The current implementation only shows `forget`; `lock` must appear as a
/// named command, not merely as a substring of "lockable".
#[test]
fn lock_appears_as_subcommand_in_top_level_help() {
    // "  lock " — two leading spaces (clap formatting) followed by "lock" then space
    // ensures we match the command entry, not "lockable" in a description.
    dont()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::is_match(r"(?m)^\s{2,4}lock\s").unwrap());
}

// ── Core four verbs: all four appear in top-level help ───────────────────────

/// Spec (Requirement: Core four verbs): the four primary CLI verbs are
/// conclude, define, trust, and dismiss.  All four MUST appear as named
/// subcommands in `--help` output.
#[test]
fn all_core_four_verbs_appear_in_top_level_help() {
    let out = dont()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    for verb in &["conclude", "define", "trust", "dismiss"] {
        assert!(
            text.contains(verb),
            "expected core four verb '{}' in --help output",
            verb
        );
    }
}

// ── Lifecycle verbs: all four appear in top-level help ───────────────────────

/// Spec (Requirement: Lifecycle verb): the lifecycle verbs are lock, reopen,
/// ignore, and verify-evidence.  All four MUST appear as named subcommands in
/// `--help` output.
#[test]
fn all_lifecycle_verbs_appear_in_top_level_help() {
    let out = dont()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    for verb in &["lock", "reopen", "ignore", "verify-evidence"] {
        assert!(
            text.contains(verb),
            "expected lifecycle verb '{}' in --help output",
            verb
        );
    }
}

// ── Tutorial vocabulary: "core four verbs" (not "core lifecycle verbs") ──────

/// The tutorial text (returned by `dont help --tutorial`) MUST use the
/// canonical term "core four verbs" defined in the glossary spec
/// (Requirement: Core four verbs).  It MUST NOT call them "core lifecycle
/// verbs", which conflates two distinct glossary concepts.
#[test]
fn tutorial_uses_canonical_core_four_verbs_label() {
    let out = dont()
        .args(["help", "--tutorial"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("core four verbs") || text.contains("Core four verbs"),
        "tutorial should use canonical term 'core four verbs'; got:\n{}",
        &text[..text.len().min(400)]
    );
    assert!(
        !text.contains("core lifecycle verbs") && !text.contains("Core lifecycle verbs"),
        "tutorial must not conflate core four verbs with lifecycle verbs; got:\n{}",
        &text[..text.len().min(400)]
    );
}

// ── Hedge pattern: refused reasons produce glossary-canonical error code ─────

/// Spec (Requirement: Hedge pattern): vague-reason patterns fail reason quality
/// checks.  The error code MUST be `reason-not-hedge` when a hedged reason is
/// supplied to `trust`, confirming the implementation uses the glossary term.
#[test]
fn hedge_pattern_produces_reason_not_hedge_error_code() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the sky is blue");

    let out = dont()
        .args(["trust", &id, "--reason", "I think so", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "hedge reason should be refused");
    assert_eq!(
        v["data"]["code"], "reason-not-hedge",
        "error code must be 'reason-not-hedge' per glossary; got: {}",
        v["data"]["code"]
    );
}

// ── Remediation: error payloads carry actionable remediation entries ──────────

/// Spec (Requirement: Remediation): actionable recovery guidance is returned in
/// error payloads.  Every refused command MUST include a non-empty
/// `data.remediation` array so that callers can surface recovery steps.
#[test]
fn error_payload_contains_non_empty_remediation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "mars has two moons");

    // trust with a hedge is a reliable way to produce a refused command
    let out = dont()
        .args(["trust", &id, "--reason", "I think so", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    let remediation = v["data"]["remediation"].as_array().expect("remediation must be an array");
    assert!(
        !remediation.is_empty(),
        "error payload must contain at least one remediation entry per glossary spec"
    );
    // Each entry must have a `command` field
    assert!(
        remediation[0]["command"].is_string(),
        "remediation entry must have a 'command' field"
    );
}

// ── Evidence: claim JSON output carries an `evidence` field ──────────────────

/// Spec (Requirement: Evidence): evidence is the grounding material cited
/// during verification workflows.  The claim JSON output MUST expose an
/// `evidence` field so that the canonical vocabulary is visible in the data
/// model.
#[test]
fn claim_json_output_contains_evidence_field() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "water boils at 100 degrees Celsius at sea level");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(
        v["data"]["evidence"].is_array(),
        "claim JSON must include an 'evidence' field (array); got data: {}",
        v["data"]
    );
}

// ── Author string: <actor-kind>:<id> format is accepted ──────────────────────

/// Spec (Requirement: Author string): the identity format for events is
/// `<actor-kind>:<id>`.  The CLI MUST accept a value in this format via
/// `--author` or `$DONT_AUTHOR` and echo it back in `meta.author`.
#[test]
fn author_string_actor_kind_colon_id_format_is_accepted() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json", "--author", "llm:claude-sonnet"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["meta"]["author"], "llm:claude-sonnet",
        "author string in <actor-kind>:<id> format must round-trip through meta.author"
    );
}
