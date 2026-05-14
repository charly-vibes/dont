/// Spec-alignment tests for dont-glossary (ticket dont-8169).
///
/// The glossary spec defines:
/// - Core four verbs: conclude, define, trust, dismiss
/// - Lifecycle verbs: lock, reopen, ignore, verify-evidence
///
/// These tests assert that the CLI exposes `dismiss` and `lock` under those
/// canonical names so that external callers, documentation, and LLM harnesses
/// can rely on the spec-defined vocabulary.
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

use predicates;
