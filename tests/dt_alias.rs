// Tests for dont-kxg: dt binary alias with positive-framing vocabulary.
//
// Single binary detects argv[0]. When invoked as 'dt':
//   record  ≡ conclude, challenge ≡ trust, lock ≡ forget
// Wrong-vocabulary calls are refused with helpful suggestions.

mod common;

use assert_cmd::Command;
use common::{conclude_claim, dont, init_dir};
use dont::store::{
    HypothesisAssessment, HypothesisRecord, Status, Store, StoreEvent, StoreEventKind,
};
use predicates::prelude::*;
use tempfile::TempDir;

/// Seed a claim into a state that satisfies the lockable gate (verified + 3 assessed hypotheses + 2 independent evidence sources).
fn seed_lockable_claim(dir: &TempDir) -> String {
    let id = conclude_claim(dir, "claim that will be locked via dt");
    let store = Store::open_dont_dir(dir.path()).unwrap();

    // Verified with 2 independent evidence sources
    store
        .append_status_change(
            &id,
            Status::Unverified,
            Status::Verified,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String(
                    "https://source1.example.com/evidence".to_string(),
                )],
            },
        )
        .unwrap();
    store
        .append_evidence_event(
            &id,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String(
                    "https://source2.example.com/evidence".to_string(),
                )],
            },
        )
        .unwrap();

    // 3 assessed hypotheses
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
    store
        .set_claim_hypotheses_for_test(&id, &hypotheses)
        .unwrap();

    id
}

fn dt() -> Command {
    // Redirect XDG_CACHE_HOME so dt's error-scratch writes don't pollute the
    // developer's real ~/.cache/dont/ during cargo test (see tests/common/mod.rs).
    let cache = std::env::temp_dir().join("dont-test-cache");
    let mut cmd = Command::cargo_bin("dt").unwrap();
    cmd.env("XDG_CACHE_HOME", &cache);
    cmd
}

// ── dt record (≡ dont conclude) ────────────────────────────────────────────

#[test]
fn dt_record_creates_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dt()
        .args(["record", "the sky is blue", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "dt record should succeed");
    assert_eq!(
        v["data"]["status"], "unverified",
        "new claim starts unverified"
    );
}

// ── dt challenge (≡ dont trust) ────────────────────────────────────────────

#[test]
fn dt_challenge_registers_doubt() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim to challenge");

    let out = dt()
        .args(["challenge", &id, "--reason", "needs source", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "doubted");
}

// ── dt lock (≡ dont forget) ────────────────────────────────────────────────

#[test]
fn dt_lock_locks_verified_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = seed_lockable_claim(&dir);

    let out = dt()
        .args(["lock", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "locked");
}

// ── shared commands work via dt ────────────────────────────────────────────

#[test]
fn dt_show_works_as_shared_command() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "shared command claim");

    dt().args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&id));
}

// ── cross-vocab: dt rejects dont-exclusive vocabulary ──────────────────────

/// "unknown command 'conclude' for dt. Did you mean 'dt record'?"
#[test]
fn dt_conclude_is_rejected_with_record_suggestion() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dt().args(["conclude", "some claim"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("record"));
}

/// "unknown command 'trust' for dt. Did you mean 'dt challenge'?"
#[test]
fn dt_trust_is_rejected_with_challenge_suggestion() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dt().args(["trust", "claim:abc", "--reason", "x"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("challenge"));
}

/// "unknown command 'forget' for dt. Did you mean 'dt lock'?"
#[test]
fn dt_forget_is_rejected_with_lock_suggestion() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dt().args(["forget", "claim:abc"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("lock"));
}

// ── cross-vocab: dont rejects dt-exclusive vocabulary ──────────────────────

/// "unknown command 'record' for dont. Did you mean 'dont conclude'?"
#[test]
fn dont_record_is_rejected_with_conclude_suggestion() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["record", "some claim"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("conclude"));
}

/// "unknown command 'challenge' for dont. Did you mean 'dont trust'?"
#[test]
fn dont_challenge_is_rejected_with_trust_suggestion() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["challenge", "claim:abc", "--reason", "x"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("trust"));
}

/// "unknown command 'lock' for dont. Did you mean 'dont forget'?"
#[test]
fn dont_lock_is_rejected_with_forget_suggestion() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["lock", "claim:abc"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("forget"));
}
