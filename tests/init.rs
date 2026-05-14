mod common;

use common::dont;
use dont::store::Store;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn init_in(dir: &TempDir) -> assert_cmd::assert::Assert {
    dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
}

// --- Successful init ---

#[test]
fn init_creates_dont_directory_with_required_entries() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    assert!(dir.path().join("db.cozo").exists(), "db.cozo");
    assert!(dir.path().join("config.toml").exists(), "config.toml");
    assert!(dir.path().join("AGENTS.md").exists(), "AGENTS.md");
    assert!(dir.path().join("seed").is_dir(), "seed/");
    assert!(dir.path().join("vocab").is_dir(), "vocab/");
    assert!(dir.path().join("rules").is_dir(), "rules/");
    assert!(dir.path().join("imports").is_dir(), "imports/");
    assert!(dir.path().join("sessions").is_dir(), "sessions/");
    assert!(dir.path().join("schemas").is_dir(), "schemas/");
}

#[test]
fn init_agents_teaches_repository_grounding_workflow() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("dont ground \"claim text\" --file README.md --lines 10-18"),
        "AGENTS.md should teach ground as the fast path for repository facts: {agents}"
    );
    assert!(
        agents.contains("underlying model is still conclude → trust → dismiss → forget"),
        "AGENTS.md should preserve the core verb model: {agents}"
    );
    assert!(
        agents.contains("dont trace <id>"),
        "AGENTS.md should recommend trace for blocker-path diagnosis: {agents}"
    );
    assert!(
        agents.contains("Prefer repository-relative file locators over opaque `file://` URIs"),
        "AGENTS.md should prefer repo-relative locators: {agents}"
    );
}

#[test]
fn init_config_toml_has_required_sections() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let config = fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(config.contains("[project]"), "[project] section");
    assert!(config.contains("[output]"), "[output] section");
    assert!(config.contains("[storage]"), "[storage] section");
}

#[test]
fn init_config_toml_defaults_to_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let config = fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(config.contains("mode = \"permissive\""));
}

#[test]
fn init_strict_sets_strict_mode() {
    let dir = TempDir::new().unwrap();

    dont()
        .arg("init")
        .arg("--strict")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let config = fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(config.contains("mode = \"strict\""));
}

#[test]
fn init_strict_mode_is_visible_in_prime() {
    let dir = TempDir::new().unwrap();
    dont()
        .arg("init")
        .arg("--strict")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .arg("prime")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["envelope_kind"], "prime");
    assert_eq!(v["data"]["mode"], "strict");
    assert_eq!(v["data"]["status_counts"]["unverified"], 0);
    assert_eq!(v["data"]["blocking"].as_array().unwrap().len(), 0);
    assert!(v["hints"]
        .as_array()
        .unwrap()
        .iter()
        .all(|hint| hint["command"] != "dont help --tutorial"));
}

#[test]
fn init_creates_locked_seed_vocabulary_snapshot() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let seed = fs::read_to_string(dir.path().join("seed/dont-seed.yaml")).unwrap();
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
        assert!(seed.contains(term), "missing seed term {term}");
    }
    assert_eq!(seed.matches("status: locked").count(), 10);
    assert!(!seed.contains("owl:Thing"));
    assert!(!seed.contains("rdfs:subClassOf"));
}

#[test]
fn init_records_initial_mode_project_event() {
    let dir = TempDir::new().unwrap();

    dont()
        .arg("init")
        .arg("--strict")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let events = fs::read_to_string(dir.path().join("events.jsonl")).unwrap();
    let event: Value = serde_json::from_str(events.lines().next().unwrap()).unwrap();
    assert_eq!(event["kind"], "project.initialized");
    assert_eq!(event["mode"], "strict");
    assert!(event["created_at"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn init_adds_dont_directory_to_gitignore() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".gitignore"), "target/\n").unwrap();

    dont()
        .arg("init")
        .arg("--json")
        .current_dir(dir.path())
        .assert()
        .success();

    let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains("target/\n"));
    assert!(gitignore.lines().any(|line| line == ".dont/"));
}

#[test]
fn init_does_not_duplicate_dont_gitignore_entry() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".gitignore"), ".dont/\n").unwrap();

    dont()
        .arg("init")
        .arg("--json")
        .current_dir(dir.path())
        .assert()
        .success();

    let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert_eq!(gitignore.lines().filter(|line| *line == ".dont/").count(), 1);
}

#[test]
fn init_outputs_success_envelope() {
    let dir = TempDir::new().unwrap();
    let output = init_in(&dir).success().get_output().stdout.clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_version"], "0.2");
    assert!(v["cli_version"].is_string());
    assert!(v["meta"].is_object());
}

// --- Already-initialized ---

#[test]
fn reinit_returns_already_initialised_error_and_exits_3() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let output = dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "already-initialised");
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty(), "remediation must be non-empty");
}

// --- Parent-directory discovery ---

#[test]
fn project_discovery_finds_dont_dir_in_parent() {
    let base = TempDir::new().unwrap();
    // Init the base (DONT_DIR points to .dont inside base)
    let dont_dir = base.path().join(".dont");
    dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", &dont_dir)
        .assert()
        .success();

    // Running conclude from a subdirectory should discover .dont/ by walking up
    let subdir = base.path().join("sub/nested");
    fs::create_dir_all(&subdir).unwrap();

    dont()
        .arg("conclude")
        .arg("test claim")
        .arg("--json")
        .current_dir(&subdir)
        // No DONT_DIR override — must discover via parent walk
        .assert()
        .success();
}

// --- Config-missing ---

#[test]
fn conclude_outside_project_returns_config_missing_and_exits_3() {
    let dir = TempDir::new().unwrap();
    // No init — no .dont/ exists

    let output = dont()
        .arg("conclude")
        .arg("a claim")
        .arg("--json")
        .env("DONT_DIR", dir.path().join("nonexistent"))
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "config-missing");
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty());
}

#[test]
fn internal_project_errors_do_not_suggest_dont_doctor() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let store = Store::open_dont_dir(dir.path()).unwrap();
    store.set_schema_version_for_test(999).unwrap();
    drop(store);

    let output = dont()
        .arg("prime")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(remediation.iter().all(|entry| entry["command"] != "dont doctor"));
}

#[test]
fn project_source_has_no_expect_calls() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/project.rs")).unwrap();
    assert!(
        !source.contains(".expect("),
        "project.rs should avoid expect() calls"
    );
}

#[test]
fn init_io_failures_name_target_path_in_structured_error() {
    let dir = TempDir::new().unwrap();
    let blocked = dir.path().join("blocked-target");
    fs::write(&blocked, "not a directory").unwrap();

    let output = dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", &blocked)
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    let message = v["data"]["message"].as_str().unwrap();
    assert!(message.contains(blocked.to_string_lossy().as_ref()), "message should name failing path: {message}");
    assert!(
        message.contains("create") || message.contains("write"),
        "message should name the failed operation: {message}"
    );
}

#[test]
fn init_treats_malformed_existing_config_as_already_initialised() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), "not valid toml = [[").unwrap();

    let output = dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["data"]["code"], "already-initialised");
}

#[test]
fn init_reports_late_gitignore_write_failures_with_path_context() {
    let root = TempDir::new().unwrap();
    fs::create_dir(root.path().join(".gitignore")).unwrap();

    let output = dont()
        .arg("init")
        .arg("--json")
        .current_dir(root.path())
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    let message = v["data"]["message"].as_str().unwrap();
    let gitignore = root.path().join(".gitignore");
    assert!(
        message.contains(gitignore.to_string_lossy().as_ref()),
        "message should name failing gitignore path: {message}"
    );
    assert!(message.contains("write"), "message should name write operation: {message}");
}

#[test]
fn prime_blocking_includes_doubted_terms() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    // Define a term and doubt it
    let term_out = dont()
        .args(["define", "EX:T001", "--doc", "a term", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let term_v: Value = serde_json::from_slice(&term_out).unwrap();
    let term_id = term_v["data"]["id"].as_str().unwrap();

    dont()
        .args(["trust", term_id, "--reason", "needs review", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    let blocking = v["data"]["blocking"].as_array().unwrap();
    assert!(
        blocking.iter().any(|b| b["id"] == term_id && b["status"] == "doubted"),
        "prime blocking[] should include doubted term {term_id}, got: {:?}",
        blocking
    );
}
