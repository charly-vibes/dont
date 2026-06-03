/// dont-init-modes spec alignment: init behavior, mode consistency, seed vocabulary.
mod common;

use common::{dont, init_dir};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// --- Repeated init is refused ---

#[test]
fn reinit_refused_with_already_initialised_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    // Must use a refusal code that identifies already-initialised state
    let code = v["data"]["code"].as_str().unwrap();
    assert!(
        code.contains("already") || code.contains("initialised") || code.contains("initialized"),
        "reinit must return an already-initialised code; got: {code}"
    );
}

// --- Init defaults to permissive mode ---

#[test]
fn init_without_strict_produces_permissive_mode_in_prime() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["mode"], "permissive",
        "init without --strict must produce permissive mode; got: {:?}",
        v["data"]["mode"]
    );
}

// --- Seed vocabulary starts locked ---

#[test]
fn seed_terms_are_in_locked_status_after_init() {
    // Seed vocabulary is snapshotted into seed/dont-seed.yaml during init.
    // Each seed term must start with status: locked.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let seed = fs::read_to_string(dir.path().join("seed/dont-seed.yaml"))
        .expect("seed/dont-seed.yaml must exist after init");

    for term in [
        "dont:Entity",
        "dont:Claim",
        "dont:Term",
        "dont:Evidence",
        "dont:kind_of",
        "dont:related_to",
        "dont:defined_as",
        "dont:Hypothesis",
        "dont:Retraction",
        "dont:external_ref",
    ] {
        assert!(
            seed.contains(term),
            "seed vocabulary must include {term}; seed content: {seed}"
        );
    }

    // All seed terms must start in locked status
    assert_eq!(
        seed.matches("status: locked").count(),
        10,
        "all 10 seed terms must have status: locked; seed: {seed}"
    );
}

// --- Mode change is recorded as a project event ---

#[test]
fn init_strict_records_mode_in_events() {
    let dir = TempDir::new().unwrap();
    dont()
        .args(["init", "--strict", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let events_text = fs::read_to_string(dir.path().join("events.jsonl")).unwrap();
    let has_strict_event = events_text.lines().any(|line| {
        serde_json::from_str::<Value>(line)
            .ok()
            .map(|v| v["mode"] == "strict")
            .unwrap_or(false)
    });
    assert!(
        has_strict_event,
        "init --strict must record mode=strict in events.jsonl"
    );
}

// --- Doctor --fix after fresh init is a no-op ---

#[test]
fn doctor_fix_after_fresh_init_changes_nothing() {
    // Spec: init must produce byte-identical managed files to those that doctor --fix would.
    // After a fresh init, doctor --fix must report no changes needed.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let agents_before = fs::read_to_string(dir.path().join("AGENTS.md")).ok();
    let dont_agents_before = fs::read_to_string(dir.path().join("AGENTS.md")).ok();

    let out = dont()
        .args(["doctor", "--fix", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], true,
        "doctor --fix must succeed on a fresh project"
    );

    // Files must not have changed
    let agents_after = fs::read_to_string(dir.path().join("AGENTS.md")).ok();
    let dont_agents_after = fs::read_to_string(dir.path().join("AGENTS.md")).ok();

    assert_eq!(
        agents_before, agents_after,
        "doctor --fix on fresh init must not change AGENTS.md"
    );
    assert_eq!(
        dont_agents_before, dont_agents_after,
        "doctor --fix on fresh init must not change .dont/AGENTS.md"
    );
}

// --- Permissive mode: unresolved depends-on allowed at conclude time ---

#[test]
fn permissive_mode_allows_conclude_with_unresolved_dep() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir); // permissive by default

    let out = dont()
        .args([
            "conclude",
            "claim with unresolved dep",
            "--depends-on",
            "nonexistent:term",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], true,
        "permissive mode must allow conclude with unresolved dep; got: {v:?}"
    );
}

// --- Strict mode: unresolved depends-on refused at conclude time ---

#[test]
fn strict_mode_refuses_conclude_with_unresolved_dep() {
    let dir = TempDir::new().unwrap();
    dont()
        .args(["init", "--strict", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args([
            "conclude",
            "claim with unresolved dep",
            "--depends-on",
            "nonexistent:term",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], false,
        "strict mode must refuse conclude with unresolved dep"
    );
    assert_eq!(
        v["data"]["code"], "unresolved-term-ref",
        "strict mode refusal must use code unresolved-term-ref; got: {:?}",
        v["data"]["code"]
    );
}
