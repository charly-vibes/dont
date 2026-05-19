mod common;

use common::dont;
use dont::store::{
    HypothesisAssessment, HypothesisRecord, Store, StoreEvent, StoreEventKind, Status,
};
use serde_json::Value;
use tempfile::TempDir;

fn init_dir(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
}

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn flag(dir: &TempDir, id: &str, evidence: &str) -> Vec<u8> {
    dont()
        .args(["flag", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

fn define_term(dir: &TempDir, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn flag_file(dir: &TempDir, id: &str, extra_args: &[&str]) -> Vec<u8> {
    let mut args = vec!["flag", id, "--json"];
    args.extend_from_slice(extra_args);
    dont()
        .args(&args)
        .env("DONT_DIR", dir.path().join(".dont"))
        .output()
        .unwrap()
        .stdout
}

// --- Valid transitions ---

#[test]
fn flag_unverified_claim_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the earth is round");
    let out = flag(&dir, &id, "https://nasa.gov/earth-shape");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn flag_doubted_claim_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "photosynthesis converts CO2 to O2");
    dont()
        .args(["trust", &id, "--reason", "Need to verify the chemistry", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
    let out = flag(&dir, &id, "https://acs.org/photosynthesis");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn flag_unverified_term_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");
    let out = flag(&dir, &id, "https://example.test/term-evidence");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn flag_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "tx is tracked for mutations");
    let out = flag(&dir, &id, "https://example.test/tx-proof");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["meta"]["tx"].is_number());
}

#[test]
fn flag_evidence_appears_in_claim_view() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence is tracked");
    let ev = "https://example.test/evidence-1";
    let out = flag(&dir, &id, ev);
    let v: Value = serde_json::from_slice(&out).unwrap();
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(
        evidence.iter().any(|e| e.as_str() == Some(ev)),
        "evidence URI missing from claim view"
    );
}

// --- Already-verified evidence append ---

#[test]
fn flag_already_verified_appends_evidence_without_status_change() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence accumulates over time");
    flag(&dir, &id, "https://example.test/ev-1");
    // Second flag on verified claim
    let out = flag(&dir, &id, "https://example.test/ev-2");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "verified");
    // Both evidence URIs should appear
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let uris: Vec<&str> = evidence.iter().filter_map(|e| e.as_str()).collect();
    assert!(uris.contains(&"https://example.test/ev-1"), "first evidence missing");
    assert!(uris.contains(&"https://example.test/ev-2"), "second evidence missing");
}

// --- Refusals ---

#[test]
fn flag_without_evidence_returns_no_evidence_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence-free flag should fail");
    let out = dont()
        .args(["flag", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn flag_refuses_claims_with_unverified_term_dependencies() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001");

    let out = dont()
        .args([
            "conclude",
            "Uses WB:P001",
            "--depends-on",
            "WB:P001",
            "--json",
        ])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap().to_string();

    let output = dont()
        .args(["flag", &id, "--evidence", "https://example.test/proof", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-met");
    assert_eq!(v["data"]["rule_name"], "stale-cascade");

    let shown = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shown_v: Value = serde_json::from_slice(&shown).unwrap();
    assert_eq!(shown_v["data"]["status"], "unverified");
}

#[test]
fn flag_claim_not_found_returns_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["flag", "claim:01JNONEXISTENT", "--evidence", "https://example.test", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
}

// --- Repository evidence locators ---

#[test]
fn flag_file_locator_stored_as_structured_entry() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::write(dir.path().join("README.md"), "API docs\n").unwrap();
    let id = conclude_claim(&dir, "README documents the API");
    let out = flag_file(&dir, &id, &["--file", "README.md"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "flag --file should succeed: {v}");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).expect("should have a structured locator entry");
    assert_eq!(locator["kind"], "repo-file");
    assert_eq!(locator["path"], "README.md");
}

#[test]
fn flag_file_locator_with_line_span() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let spec = (1..=18)
        .map(|n| format!("line {n}\n"))
        .collect::<String>();
    std::fs::write(dir.path().join("spec.md"), spec).unwrap();
    let id = conclude_claim(&dir, "spec defines the contract on lines 10-18");
    let out = flag_file(&dir, &id, &["--file", "spec.md", "--lines", "10-18"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "flag --file --lines should succeed: {v}");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).unwrap();
    assert_eq!(locator["kind"], "repo-file");
    assert_eq!(locator["path"], "spec.md");
    assert_eq!(locator["line_start"], 10);
    assert_eq!(locator["line_end"], 18);
}

#[test]
fn flag_file_locator_with_anchor() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::create_dir(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("docs/api.md"), "# authentication\n").unwrap();
    let id = conclude_claim(&dir, "section heading anchors the claim");
    let out = flag_file(&dir, &id, &["--file", "docs/api.md", "--anchor", "authentication"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).unwrap();
    assert_eq!(locator["anchor"], "authentication");
}

#[test]
fn flag_file_locator_escape_via_dotdot_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim needing evidence");
    let out = flag_file(&dir, &id, &["--file", "../../etc/passwd"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "path escape should be refused: {v}");
    assert_eq!(v["data"]["code"], "path-escapes-root");
}

#[test]
fn flag_file_locator_absolute_path_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim needing evidence");
    let out = flag_file(&dir, &id, &["--file", "/etc/passwd"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "absolute path should be refused: {v}");
    assert_eq!(v["data"]["code"], "path-not-relative");
}

#[test]
fn flag_file_and_uri_evidence_combined() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::write(dir.path().join("README.md"), "proof\n").unwrap();
    let id = conclude_claim(&dir, "claim grounded in both repo and URI evidence");
    let out = flag_file(
        &dir,
        &id,
        &["--file", "README.md", "--evidence", "https://example.test/proof"],
    );
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(evidence.iter().any(|e| e.as_str() == Some("https://example.test/proof")), "URI evidence missing");
    assert!(evidence.iter().any(|e| e.is_object()), "locator entry missing");
}

#[test]
fn flag_no_evidence_without_file_flag_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "must provide evidence");
    let out = flag_file(&dir, &id, &[]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
}

// --- Git provenance helpers ---

fn init_git_repo(dir: &TempDir) {
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            // Unset hook-injected vars so git operates on this temp repo,
            // not on the surrounding worktree (e.g. during prek/lefthook runs).
            // GIT_WORK_TREE without GIT_DIR is also fatal to git.
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap();
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
}

fn git_add_commit(dir: &TempDir, file: &str) {
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap();
    };
    run(&["add", file]);
    run(&["commit", "-m", "add test file"]);
}

fn git_stage(dir: &TempDir, file: &str) {
    std::process::Command::new("git")
        .args(["add", file])
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
}

// --- Git provenance tests ---

#[test]
fn flag_file_locator_includes_commit_ref() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    init_git_repo(&dir);
    std::fs::write(dir.path().join("README.md"), "API docs\n").unwrap();
    git_add_commit(&dir, "README.md");
    let id = conclude_claim(&dir, "README documents the API");
    let out = flag_file(&dir, &id, &["--file", "README.md"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "flag --file should succeed for clean committed file: {v}");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).expect("should have locator");
    let commit_ref = locator["commit_ref"].as_str().expect("commit_ref must be present for git repo");
    assert!(commit_ref.starts_with("git:"), "commit_ref should start with 'git:': {commit_ref}");
    assert_eq!(commit_ref.len(), 44, "commit_ref should be 'git:' + 40-char SHA: {commit_ref}");
}

#[test]
fn flag_file_untracked_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    init_git_repo(&dir);
    // initial commit so HEAD exists
    std::fs::write(dir.path().join("initial.md"), "seed\n").unwrap();
    git_add_commit(&dir, "initial.md");
    // untracked file
    std::fs::write(dir.path().join("untracked.md"), "not tracked\n").unwrap();
    let id = conclude_claim(&dir, "claim needing file evidence");
    let out = flag_file(&dir, &id, &["--file", "untracked.md"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "untracked file should be rejected: {v}");
    assert_eq!(v["data"]["code"], "untracked-file");
}

#[test]
fn flag_file_staged_not_committed_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    init_git_repo(&dir);
    // initial commit so HEAD exists
    std::fs::write(dir.path().join("initial.md"), "seed\n").unwrap();
    git_add_commit(&dir, "initial.md");
    // staged but not committed
    std::fs::write(dir.path().join("staged.md"), "staged content\n").unwrap();
    git_stage(&dir, "staged.md");
    let id = conclude_claim(&dir, "claim needing file evidence");
    let out = flag_file(&dir, &id, &["--file", "staged.md"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "staged-not-committed file should be rejected: {v}");
    assert_eq!(v["data"]["code"], "staged-not-committed");
}

#[test]
fn flag_file_dirty_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    init_git_repo(&dir);
    std::fs::write(dir.path().join("dirty.md"), "original content\n").unwrap();
    git_add_commit(&dir, "dirty.md");
    // modify without staging
    std::fs::write(dir.path().join("dirty.md"), "modified content\n").unwrap();
    let id = conclude_claim(&dir, "claim needing file evidence");
    let out = flag_file(&dir, &id, &["--file", "dirty.md"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "dirty file should be rejected: {v}");
    assert_eq!(v["data"]["code"], "dirty-file");
}

// --- Locked-entity transition refusals ---

fn seed_verified_claim_with_evidence_in_dont_dir(dir: &TempDir, claim_id: &str, evidence: &[&str]) {
    let store = Store::open_dont_dir(dir.path().join(".dont")).unwrap();
    let first = evidence.first().expect("at least one evidence item");
    store
        .append_status_change(
            claim_id,
            Status::Unverified,
            Status::Verified,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![serde_json::Value::String((*first).to_string())],
            },
        )
        .unwrap();
    for uri in &evidence[1..] {
        store
            .append_evidence_event(
                claim_id,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![serde_json::Value::String((*uri).to_string())],
                },
            )
            .unwrap();
    }
}

fn seed_assessed_hypotheses_in_dont_dir(dir: &TempDir, claim_id: &str, count: usize) {
    let store = Store::open_dont_dir(dir.path().join(".dont")).unwrap();
    let hypotheses: Vec<HypothesisRecord> = (0..count)
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

#[test]
fn flag_locked_claim_is_refused_with_invalid_transition() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "A claim that will be permanently locked");

    seed_verified_claim_with_evidence_in_dont_dir(
        &dir,
        &id,
        &[
            "https://source-one.example/evidence",
            "https://source-two.example/evidence",
        ],
    );
    seed_assessed_hypotheses_in_dont_dir(&dir, &id, 3);

    dont()
        .args(["forget", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();

    // Now try to flag the locked claim — must be refused
    let out = dont()
        .args(["flag", &id, "--evidence", "https://example.test/new-proof", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "flag on locked claim must be refused: {v}");
    assert_eq!(v["data"]["code"], "invalid-transition",
        "error code must be invalid-transition: {v}");
}

// --- Spec: deprecated alias emits warning ---
// Spec (Requirement: flag (formerly dismiss)):
//   "WHEN `dont dismiss` is invoked
//    THEN a deprecation warning is emitted to stderr suggesting `flag` instead
//    AND the command proceeds normally
//    AND the deprecation warning goes to stderr regardless of whether --json is set"

#[test]
fn dismiss_alias_emits_deprecation_warning_to_stderr() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim verified via deprecated dismiss alias");

    let output = dont()
        .args(["dismiss", &id, "--evidence", "https://example.test/dep-proof", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .clone();

    // Command must succeed (proceed normally)
    let v: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["ok"], true, "dismiss must proceed normally: {v}");
    assert_eq!(v["data"]["status"], "verified", "dismiss must verify the claim: {v}");

    // Deprecation warning MUST go to stderr
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("deprecated") || stderr.contains("flag"),
        "dismiss must emit a deprecation warning to stderr suggesting 'flag'; got stderr: {:?}",
        stderr
    );
}

#[test]
fn dismiss_alias_emits_deprecation_warning_to_stderr_with_json_flag() {
    // The spec says the warning goes to stderr regardless of whether --json is set.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim for deprecation-warning-with-json test");

    let output = dont()
        .args(["dismiss", &id, "--evidence", "https://example.test/dep-proof2", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .clone();

    // Even with --json, stdout must be valid JSON and stderr must have the warning
    let v: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["ok"], true, "dismiss --json must succeed");

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("deprecated") || stderr.contains("flag"),
        "dismiss --json must still emit deprecation warning to stderr; got: {:?}",
        stderr
    );
}

#[test]
fn dismiss_deprecation_warning_does_not_appear_on_stdout() {
    // The deprecation warning must go ONLY to stderr, not contaminate stdout JSON.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim for stdout-clean deprecation check");

    let output = dont()
        .args(["dismiss", &id, "--evidence", "https://example.test/dep-proof3", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .clone();

    // stdout must be parseable JSON without the warning text
    let v: Value = serde_json::from_slice(&output.stdout)
        .expect("stdout must be valid JSON even when dismiss emits a deprecation warning");
    assert_eq!(v["ok"], true);
}

// --- Malformed evidence URI validation ---

/// A bare string with no scheme is not a valid evidence locator.
/// `dont flag` must reject it at input time with a structured error,
/// not silently store it for later failure.
#[test]
fn flag_malformed_evidence_uri_no_scheme_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim that should never be verified with garbage evidence");

    let out = dont()
        .args(["flag", &id, "--evidence", "not-a-valid-locator", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "malformed evidence URI must be rejected: {v}");
    assert_eq!(
        v["data"]["code"], "malformed-evidence-uri",
        "error code must be malformed-evidence-uri, got: {:?}",
        v["data"]["code"]
    );
    // Error message must mention the offending string so the user can act on it.
    let msg = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("not-a-valid-locator"),
        "error message must include the offending locator, got: {msg}"
    );
    // Must include remediation hints.
    assert!(
        !v["data"]["remediation"].as_array().unwrap().is_empty(),
        "malformed-evidence-uri must include remediation hints"
    );
}

/// The claim must remain unverified when a malformed URI is rejected.
#[test]
fn flag_malformed_evidence_uri_does_not_change_claim_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "status should not change on malformed evidence");

    dont()
        .args(["flag", &id, "--evidence", "garbage-no-scheme", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1);

    // Claim must still be unverified.
    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["status"], "unverified",
        "claim status must remain unverified after malformed-evidence rejection: {v}"
    );
}

// --- Multi-file evidence accumulation ---

/// Two sequential `--file` invocations on different files accumulate both
/// structured locators under `dont show --json`.
#[test]
fn flag_multi_file_evidence_both_appear_in_show() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::write(dir.path().join("docs.md"), "documentation content\n").unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/impl.rs"), "fn main() {}\n").unwrap();
    let id = conclude_claim(&dir, "claim supported by two separate source files");

    // First file evidence
    let out1 = flag_file(&dir, &id, &["--file", "docs.md"]);
    let v1: Value = serde_json::from_slice(&out1).unwrap();
    assert_eq!(v1["ok"], true, "first --file flag should succeed: {v1}");

    // Second file evidence — different path
    let out2 = flag_file(&dir, &id, &["--file", "src/impl.rs"]);
    let v2: Value = serde_json::from_slice(&out2).unwrap();
    assert_eq!(v2["ok"], true, "second --file flag should succeed: {v2}");

    // Verify via `dont show --json` that both locators are present
    let show_out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let sv: Value = serde_json::from_slice(&show_out).unwrap();
    assert_eq!(sv["ok"], true, "show must succeed: {sv}");
    let evidence = sv["data"]["evidence"].as_array().unwrap();
    assert_eq!(
        evidence.iter().filter(|e| e.is_object()).count(),
        2,
        "show must expose exactly two repo-file locators; got: {evidence:?}"
    );
    let paths: Vec<&str> = evidence
        .iter()
        .filter_map(|e| e["path"].as_str())
        .collect();
    assert!(paths.contains(&"docs.md"), "docs.md locator missing from show: {paths:?}");
    assert!(paths.contains(&"src/impl.rs"), "src/impl.rs locator missing from show: {paths:?}");
}

/// `dont why --json` exposes both file locators in `data.entity.evidence`
/// when two different files have been flagged against the same claim.
#[test]
fn flag_multi_file_evidence_both_appear_in_why() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::write(dir.path().join("spec.md"), "spec content\n").unwrap();
    std::fs::write(dir.path().join("test.rs"), "// test\n").unwrap();
    let id = conclude_claim(&dir, "claim verifiable from spec and test file");

    flag_file(&dir, &id, &["--file", "spec.md"]);
    flag_file(&dir, &id, &["--file", "test.rs"]);

    let why_out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let wv: Value = serde_json::from_slice(&why_out).unwrap();
    assert_eq!(wv["ok"], true, "why must succeed: {wv}");

    // `data.entity.evidence` mirrors the claim view
    let evidence = wv["data"]["entity"]["evidence"].as_array().unwrap();
    assert_eq!(
        evidence.iter().filter(|e| e.is_object()).count(),
        2,
        "why must expose exactly two repo-file locators; got: {evidence:?}"
    );
    let paths: Vec<&str> = evidence
        .iter()
        .filter_map(|e| e["path"].as_str())
        .collect();
    assert!(paths.contains(&"spec.md"), "spec.md locator missing from why: {paths:?}");
    assert!(paths.contains(&"test.rs"), "test.rs locator missing from why: {paths:?}");

    // `data.history` must contain two flagged events
    let history = wv["data"]["history"].as_array().unwrap();
    let flagged_count = history.iter().filter(|h| h["event_kind"] == "flagged").count();
    assert_eq!(flagged_count, 2, "why history must show two flagged events; got: {history:?}");
}

/// A `file:` scheme URI is not an accepted evidence scheme; it must be rejected.
#[test]
fn flag_file_scheme_uri_is_rejected_as_malformed() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "file URI should be rejected as malformed");

    let out = dont()
        .args(["flag", &id, "--evidence", "file:///etc/passwd", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "file: URI must be rejected as malformed: {v}");
    assert_eq!(
        v["data"]["code"], "malformed-evidence-uri",
        "got: {:?}",
        v["data"]["code"]
    );
}
