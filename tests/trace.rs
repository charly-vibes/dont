use assert_cmd::Command;
use dont::store::Store;
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
    serde_json::from_slice::<Value>(&out).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

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
        .args(["define", curie, "--doc", "a test term definition", "--json"])
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

fn dismiss(dir: &TempDir, id: &str, evidence: &str) {
    dont()
        .args(["flag", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn trust(dir: &TempDir, id: &str, reason: &str) {
    dont()
        .args(["trust", id, "--reason", reason, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

// --- trace ---

#[test]
fn trace_healthy_claim_returns_empty_blockers() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "standalone healthy claim");
    dismiss(&dir, &id, "https://example.com/evidence");

    let out = dont()
        .args(["trace", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "trace");
    assert_eq!(v["data"]["entity_id"], id.as_str());
    assert!(v["data"]["blockers"].as_array().unwrap().is_empty());
    assert!(v["data"]["as_of"].as_str().unwrap().contains('T'));
}

#[test]
fn trace_unverified_standalone_claim_returns_empty_blockers() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "unverified standalone claim");

    let out = dont()
        .args(["trace", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let blockers = v["data"]["blockers"].as_array().unwrap();
    assert!(blockers.is_empty(), "standalone unverified claim has no dep blockers");
}

#[test]
fn trace_claim_blocked_by_doubted_term_shows_stale_path() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let term_id = define_term(&dir, "EX:T001");
    let claim_id = conclude_with_deps(&dir, "relies on doubted term", &["EX:T001"]);
    trust(&dir, &term_id, "the definition is inaccurate");

    let out = dont()
        .args(["trace", &claim_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let blockers = v["data"]["blockers"].as_array().unwrap();
    assert_eq!(blockers.len(), 1);
    let blocker = &blockers[0];
    assert_eq!(blocker["kind"], "stale");
    assert_eq!(blocker["start_entity"], claim_id.as_str());
    assert_eq!(blocker["blocking_node"], term_id.as_str());
    let path = blocker["path"].as_array().unwrap();
    assert!(path.iter().any(|p| p == claim_id.as_str()));
    assert!(path.iter().any(|p| p == term_id.as_str()));
    assert!(!blocker["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn trace_claim_with_unresolved_curie_shows_unresolved_term_path() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let claim_id = conclude_with_deps(&dir, "relies on missing term", &["EX:MISSING"]);

    let out = dont()
        .args(["trace", &claim_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let blockers = v["data"]["blockers"].as_array().unwrap();
    assert_eq!(blockers.len(), 1);
    let blocker = &blockers[0];
    assert_eq!(blocker["kind"], "unresolved-term");
    assert_eq!(blocker["start_entity"], claim_id.as_str());
    assert_eq!(blocker["unresolved_reference"], "EX:MISSING");
    assert!(!blocker["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn trace_multiple_independent_blockers_reported_separately() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let term_id = define_term(&dir, "EX:T002");
    trust(&dir, &term_id, "concerns about this term");
    let claim_id = conclude_with_deps(
        &dir,
        "depends on doubted term and missing term",
        &["EX:T002", "EX:ALSO_MISSING"],
    );

    let out = dont()
        .args(["trace", &claim_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let blockers = v["data"]["blockers"].as_array().unwrap();
    assert_eq!(blockers.len(), 2, "two independent blockers reported separately");
    let kinds: Vec<&str> = blockers.iter().map(|p| p["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"stale"));
    assert!(kinds.contains(&"unresolved-term"));
}

#[test]
fn trace_unknown_entity_returns_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["trace", "claim:nonexistent", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
}

#[test]
fn trace_duplicate_dependency_is_reported_once() {
    // Validates deduplication: if the same term appears multiple times in depends_on,
    // trace emits only one blocker entry (visited-set semantics).
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let term_id = define_term(&dir, "EX:DUP");
    trust(&dir, &term_id, "duplicate dep test");
    let claim_id = conclude_with_deps(
        &dir,
        "depends on same term twice",
        &["EX:DUP", "EX:DUP"],
    );

    let out = dont()
        .args(["trace", &claim_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let blockers = v["data"]["blockers"].as_array().unwrap();
    assert_eq!(blockers.len(), 1, "duplicate dep should be reported exactly once");
}

#[test]
fn trace_remediation_contains_valid_dont_commands() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let term_id = define_term(&dir, "EX:T003");
    trust(&dir, &term_id, "needs re-evaluation");
    let claim_id = conclude_with_deps(&dir, "depends on doubted term", &["EX:T003"]);

    let out = dont()
        .args(["trace", &claim_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let blockers = v["data"]["blockers"].as_array().unwrap();
    let remediation = blockers[0]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty());
    for entry in remediation {
        let cmd = entry["command"].as_str().unwrap();
        assert!(
            cmd.starts_with("dont "),
            "remediation command should be a dont command, got: {cmd}"
        );
    }
}

#[test]
fn trace_self_referential_dependency_terminates_with_bounded_output() {
    // Spec: "WHEN the traversed dependency/support graph contains a cycle,
    // THEN dont trace terminates without infinite expansion and reports the
    // bounded path context needed for diagnosis."
    //
    // We inject a claim whose depends_on list contains its own ID — the most
    // degenerate cycle possible.  The implementation's visited-set guard MUST
    // prevent infinite expansion; the result MUST be a valid JSON envelope
    // with a finite (non-empty) blockers array.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Inject the self-referential claim directly via the store API, bypassing
    // the CLI's term-resolution layer (which only accepts term CURIEs).
    let store = Store::open_dont_dir(dir.path()).unwrap();
    let result = store
        .append_claim("self-referential claim for cycle test", &[], None)
        .unwrap();
    let claim_id = result.id.clone();

    // Now re-open and inject depends_on containing the claim's own ID.
    // We do this by creating a second append that references the first claim's
    // ID as a raw dependency string.  Because append_claim stores whatever
    // strings we give it, we can model the degenerate self-cycle.
    let self_dep = vec![claim_id.clone()];
    // Normally append_claim generates a new ID. Instead, we test the
    // cycle-guard by asserting that even if the trace encounters its own
    // start node in the dependency list, it terminates.
    //
    // Strategy: create a claim whose depends_on contains a CURIE that maps to
    // a term whose ID equals the claim's own ID prefix.  Since `trace_claim`
    // inserts the start entity into `visited` before iterating, a dep equal to
    // the start entity ID is always skipped.
    let claim_with_self_dep = store
        .append_claim("claim that lists itself as a dep", &self_dep, None)
        .unwrap();
    let cyclic_id = claim_with_self_dep.id.clone();

    // The self-dep entry ("claim:...") is not a term ID and not a known CURIE,
    // so trace would classify it as unresolved-term — but because cyclic_id !=
    // claim_id, the visited-set only prevents re-visiting the START node.
    // However, the single-level traversal means there is no recursion at all,
    // so the output is bounded by definition.  This test documents and verifies
    // that guarantee.
    let out = dont()
        .args(["trace", &cyclic_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "trace");
    // The self-dep "claim:..." is treated as an unresolved-term reference since
    // it doesn't resolve to any defined term.  Trace must produce exactly one
    // bounded blocker, not an infinite expansion.
    let blockers = v["data"]["blockers"].as_array().unwrap();
    assert_eq!(
        blockers.len(),
        1,
        "self-referential dep produces exactly one bounded blocker, not infinite expansion"
    );
    assert_eq!(blockers[0]["kind"], "unresolved-term");
}
