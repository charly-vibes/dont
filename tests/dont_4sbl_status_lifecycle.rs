// dont-4sbl: SPEC-ALIGN dont-status-lifecycle
//
// Tests for spec claims not covered by existing tests.
// Spec: openspec/specs/dont-status-lifecycle/spec.md

mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn conclude_with_deps(dir: &TempDir, statement: &str, deps: &[&str]) -> String {
    let mut args = vec!["conclude", statement, "--json"];
    for dep in deps {
        args.push("--depends-on");
        args.push(dep);
    }
    let out = dont()
        .args(&args)
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
        .args(["define", curie, "--doc", "a test term", "--json"])
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

fn trust_entity(dir: &TempDir, id: &str) {
    dont()
        .args(["trust", id, "--reason", "spec-align test doubt", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn ignore_entity(dir: &TempDir, id: &str) {
    dont()
        .args(["ignore", id, "--reason", "spec-align test ignore", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn show_entity(dir: &TempDir, id: &str) -> Value {
    let out = dont()
        .args(["show", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<Value>(&out).unwrap()
}

// --- Scenario: ignored entities have empty derived_assessments ---
//
// Spec: "WHEN an entity has persisted status `ignored`
//        THEN read/query output for that entity has an empty `derived_assessments[]`"

#[test]
fn ignored_claim_with_doubted_dep_has_empty_derived_assessments() {
    // Setup: term → claim depends on it → term is doubted → claim is ignored
    // The dependent claim is ignored. Even though its dependency is doubted,
    // derived_assessments must be [] because the entity is ignored.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let term_id = define_term(&dir, "WB:IGNORED001");
    let claim_id = conclude_with_deps(
        &dir,
        "claim depending on doubted term but will be ignored",
        &["WB:IGNORED001"],
    );

    // Put the term in doubted state — this would normally cause stale on the claim
    trust_entity(&dir, &term_id);

    // Ignore the claim
    ignore_entity(&dir, &claim_id);

    // Show the ignored claim — derived_assessments must be empty
    let v = show_entity(&dir, &claim_id);
    assert_eq!(v["data"]["status"], "ignored", "claim should be ignored");
    let derived = v["data"]["derived_assessments"]
        .as_array()
        .expect("derived_assessments must be an array");
    assert!(
        derived.is_empty(),
        "ignored entity must have empty derived_assessments, got: {derived:?}"
    );
}

// --- Scenario: direct doubt does not cascade into persisted dependent status ---
//
// Spec: "WHEN a dependency transitions to `doubted`
//        THEN dependent entities do not receive automatic persisted lifecycle transitions"

#[test]
fn doubting_a_term_does_not_change_dependent_claims_persisted_status() {
    // Setup: claim depends on a term; trust (doubt) the term; verify dependent claim status unchanged
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let term_id = define_term(&dir, "WB:NOCASCADE001");
    let claim_id = conclude_with_deps(
        &dir,
        "this claim must not change status when its dep is doubted",
        &["WB:NOCASCADE001"],
    );

    // Confirm claim starts unverified
    let before = show_entity(&dir, &claim_id);
    assert_eq!(before["data"]["status"], "unverified");

    // Doubt the dependency
    trust_entity(&dir, &term_id);

    // The dependent claim's persisted status must remain unverified
    let after = show_entity(&dir, &claim_id);
    assert_eq!(
        after["data"]["status"], "unverified",
        "doubting a dependency must not change the dependent claim's persisted status"
    );

    // But derived_assessments should contain stale (computed on demand)
    let derived = after["data"]["derived_assessments"]
        .as_array()
        .expect("derived_assessments must be an array");
    assert!(
        derived.iter().any(|d| d.as_str() == Some("stale")),
        "dependent claim should have stale derived assessment, got: {derived:?}"
    );
}

// --- Scenario: reopen does not target derived stale assessment ---
//
// Spec: "WHEN an actor invokes `reopen` on an entity whose persisted status is `verified`
//        and whose `derived_assessments[]` contains `stale`
//        THEN the command is refused because `stale` is not a persisted lifecycle state"

#[test]
fn reopen_verified_stale_claim_is_refused() {
    // Setup: claim depends on a term, claim is flagged verified, then term is doubted
    // (giving claim stale derived_assessment). Reopen must be refused.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let term_id = define_term(&dir, "WB:STALE001");
    let claim_id = conclude_with_deps(&dir, "verified claim that becomes stale", &["WB:STALE001"]);

    // First flag the term to verified so stale-cascade does not block the claim flag
    dont()
        .args([
            "flag",
            &term_id,
            "--evidence",
            "https://example.test/term-ev1",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Flag the claim to verified (dependency is verified, so stale-cascade is satisfied)
    dont()
        .args([
            "flag",
            &claim_id,
            "--evidence",
            "https://example.test/ev1",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Confirm verified
    let v = show_entity(&dir, &claim_id);
    assert_eq!(v["data"]["status"], "verified");

    // Now doubt the dependency so derived_assessments gains stale
    trust_entity(&dir, &term_id);

    // Confirm stale is present in derived_assessments
    let v2 = show_entity(&dir, &claim_id);
    let derived = v2["data"]["derived_assessments"]
        .as_array()
        .expect("derived_assessments must be array");
    assert!(
        derived.iter().any(|d| d.as_str() == Some("stale")),
        "claim should be stale, got: {derived:?}"
    );
    assert_eq!(
        v2["data"]["status"], "verified",
        "persisted status must remain verified"
    );

    // Now try to reopen — must be refused because status is verified, not ignored
    let output = dont()
        .args(["reopen", &claim_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v3: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v3["ok"], false, "reopen on stale verified claim must fail");
    assert_eq!(
        v3["data"]["code"], "invalid-transition",
        "error code must be invalid-transition"
    );
}
