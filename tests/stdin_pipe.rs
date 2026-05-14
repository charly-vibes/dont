mod common;

use common::{conclude_claim, dont, init_dir};
use dont::store::{
    HypothesisAssessment, HypothesisRecord, Store, StoreEvent, StoreEventKind, StoreStatus,
};
use serde_json::Value;
use tempfile::TempDir;

fn seed_lockable_claim(dir: &TempDir, claim_id: &str) {
    let store = Store::open_dont_dir(dir.path()).unwrap();
    store
        .append_status_change(
            claim_id,
            StoreStatus::Unverified,
            StoreStatus::Verified,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String("https://src-a.example".to_string())],
            },
        )
        .unwrap();
    store
        .append_evidence_event(
            claim_id,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String("https://src-b.example".to_string())],
            },
        )
        .unwrap();
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
    store.set_claim_hypotheses_for_test(claim_id, &hypotheses).unwrap();
}

// --- show stdin ---

#[test]
fn show_stdin_processes_two_ids_as_ndjson() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "claim one");
    let id2 = conclude_claim(&dir, "claim two");

    let input = format!("{id1}\n{id2}\n");
    let raw = dont()
        .args(["show", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(raw.status.success(), "exit code should be 0 for all-valid IDs");
    let stdout = String::from_utf8(raw.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "should emit one JSON line per entity");
    let v1: Value = serde_json::from_str(lines[0]).expect("line 0 must be valid JSON");
    let v2: Value = serde_json::from_str(lines[1]).expect("line 1 must be valid JSON");
    assert_eq!(v1["ok"], true);
    assert_eq!(v1["envelope_kind"], "claim");
    assert_eq!(v2["ok"], true);
    assert_eq!(v2["envelope_kind"], "claim");
}

#[test]
fn show_stdin_empty_lines_skipped() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "skip empty lines test");

    let input = format!("\n\n{id1}\n\n");
    let raw = dont()
        .args(["show", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(raw.status.success());
    let stdout = String::from_utf8(raw.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn show_stdin_whitespace_stripped_from_ids() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "whitespace strip test");

    let input = format!("  {id}  \r\n");
    let raw = dont()
        .args(["show", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(raw.status.success());
    let stdout = String::from_utf8(raw.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let v: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn show_stdin_invalid_id_emits_error_envelope_and_continues() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "valid after invalid");

    let input = format!("claim:notexist\n{id}\n");
    let raw = dont()
        .args(["show", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin(input)
        .output()
        .unwrap();

    // exit code should be 1 (highest severity across lines)
    assert_eq!(raw.status.code(), Some(1));

    let stdout = String::from_utf8(raw.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "should emit one envelope per input line (error + success)");
    let v1: Value = serde_json::from_str(lines[0]).unwrap();
    let v2: Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(v1["ok"], false, "first line should be error envelope");
    assert_eq!(v2["ok"], true, "second line should be success envelope");
}

#[test]
fn show_stdin_exit_code_highest_severity() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "mixed result");

    // valid then invalid → still exit 1
    let input = format!("{id}\nclaim:bogus\n");
    let raw = dont()
        .args(["show", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin(input)
        .output()
        .unwrap();

    assert_eq!(raw.status.code(), Some(1));
}

// --- stdin rejection for non-ID-sink verbs ---

#[test]
fn conclude_rejects_stdin_dash_with_usage_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let raw = dont()
        .args(["conclude", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin("some statement\n")
        .output()
        .unwrap();

    assert_eq!(raw.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&raw.stdout).unwrap();
    assert_eq!(v["ok"], false);
}

#[test]
fn ground_rejects_stdin_dash_with_usage_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let raw = dont()
        .args(["ground", "-", "--evidence", "https://example.com", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin("some statement\n")
        .output()
        .unwrap();

    assert_eq!(raw.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&raw.stdout).unwrap();
    assert_eq!(v["ok"], false);
}

// --- lock stdin ---

#[test]
fn lock_stdin_processes_multiple_ids() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "lock stdin one");
    let id2 = conclude_claim(&dir, "lock stdin two");

    // seed both claims with the full lockable gate (verified + ≥2 evidence + ≥3 hypotheses)
    seed_lockable_claim(&dir, &id1);
    seed_lockable_claim(&dir, &id2);

    let input = format!("{id1}\n{id2}\n");
    let raw = dont()
        .args(["forget", "-", "--json"])
        .env("DONT_DIR", dir.path())
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(raw.status.success());
    let stdout = String::from_utf8(raw.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in lines {
        let v: Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["data"]["status"], "locked");
    }
}
