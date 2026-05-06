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

fn define_term(dir: &TempDir, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

#[test]
fn ignore_unverified_claim_produces_ignored_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    let output = dont()
        .args(["ignore", &id, "--reason", "out of scope for this project", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "ignored");
    assert_eq!(v["data"]["id"], id);
}

#[test]
fn ignore_requires_reason() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    let output = dont()
        .args(["ignore", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "reason-required");
}

#[test]
fn ignore_hedge_only_reason_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    let output = dont()
        .args(["ignore", &id, "--reason", "maybe not relevant", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "reason-not-hedge");
}

#[test]
fn ignore_already_ignored_claim_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    dont()
        .args(["ignore", &id, "--reason", "out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["ignore", &id, "--reason", "still out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "invalid-transition");
}

#[test]
fn ignore_verified_claim_produces_ignored_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    dont()
        .args(["dismiss", &id, "--evidence", "https://example.com/evidence", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["ignore", &id, "--reason", "superseded by a better claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "ignored");
}

#[test]
fn ignore_doubted_claim_produces_ignored_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    dont()
        .args(["trust", &id, "--reason", "contradicted by observation", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["ignore", &id, "--reason", "domain out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "ignored");
}

#[test]
fn ignore_unverified_term_produces_ignored_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");

    let output = dont()
        .args(["ignore", &id, "--reason", "deprecated concept", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "ignored");
}

#[test]
fn ignore_entity_not_found_returns_structured_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["ignore", "claim:NOTEXIST", "--reason", "out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "entity-not-found");
}

#[test]
fn ignore_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Gravity causes apples to fall");

    let output = dont()
        .args(["ignore", &id, "--reason", "out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert!(v["meta"]["tx"].as_i64().unwrap() > 0);
}

#[test]
fn prime_status_counts_includes_ignored() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let claim_id = conclude_claim(&dir, "The sky is blue");
    let term_id = define_term(&dir, "proj:Concept");

    dont()
        .args(["ignore", &claim_id, "--reason", "out of scope", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    dont()
        .args(["ignore", &term_id, "--reason", "deprecated concept", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["data"]["status_counts"]["ignored"], 2);
    assert_eq!(v["data"]["status_counts"]["unverified"], 0);
    assert_eq!(v["data"]["status_counts"]["locked"], 0);
}
