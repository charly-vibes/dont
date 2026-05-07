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
fn trace_healthy_claim_returns_empty_blocker_paths() {
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
    assert!(v["data"]["blocker_paths"].as_array().unwrap().is_empty());
}

#[test]
fn trace_unverified_standalone_claim_returns_empty_blocker_paths() {
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
    let paths = v["data"]["blocker_paths"].as_array().unwrap();
    assert!(paths.is_empty(), "standalone unverified claim has no dep blockers");
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
    let paths = v["data"]["blocker_paths"].as_array().unwrap();
    assert_eq!(paths.len(), 1);
    let bp = &paths[0];
    assert_eq!(bp["kind"], "stale");
    assert_eq!(bp["blocking_node"], term_id.as_str());
    let path = bp["path"].as_array().unwrap();
    assert!(path.iter().any(|p| p == claim_id.as_str()));
    assert!(path.iter().any(|p| p == term_id.as_str()));
    assert!(!bp["remediation"].as_array().unwrap().is_empty());
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
    let paths = v["data"]["blocker_paths"].as_array().unwrap();
    assert_eq!(paths.len(), 1);
    let bp = &paths[0];
    assert_eq!(bp["kind"], "unresolved-term");
    assert_eq!(bp["blocking_node"], "EX:MISSING");
    assert!(!bp["remediation"].as_array().unwrap().is_empty());
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
    let paths = v["data"]["blocker_paths"].as_array().unwrap();
    assert_eq!(paths.len(), 2, "two independent blockers reported separately");
    let kinds: Vec<&str> = paths.iter().map(|p| p["kind"].as_str().unwrap()).collect();
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
    let paths = v["data"]["blocker_paths"].as_array().unwrap();
    assert_eq!(paths.len(), 1, "duplicate dep should be reported exactly once");
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
    let paths = v["data"]["blocker_paths"].as_array().unwrap();
    let remediation = paths[0]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty());
    for entry in remediation {
        let cmd = entry["command"].as_str().unwrap();
        assert!(
            cmd.starts_with("dont "),
            "remediation command should be a dont command, got: {cmd}"
        );
    }
}
