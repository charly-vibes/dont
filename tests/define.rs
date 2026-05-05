use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_dir(dir: &TempDir) {
    dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn define_creates_unverified_term() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args([
            "define",
            "WB:P001",
            "--doc",
            "Process by which X becomes Y",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert!(v["data"]["id"].as_str().unwrap().starts_with("term:"));
    assert_eq!(v["data"]["entity_kind"], "term");
    assert_eq!(v["data"]["curie"], "WB:P001");
    assert_eq!(v["data"]["definition"], "Process by which X becomes Y");
    assert_eq!(v["data"]["status"], "unverified");
    assert!(v["meta"]["tx"].is_number());
}

#[test]
fn define_missing_curie_returns_structured_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "--doc", "A definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "curie-required");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_missing_doc_returns_structured_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "doc-required");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn defined_term_can_be_shown_after_reopen() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "Process by which X", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    let id = v["data"]["id"].as_str().unwrap();

    let output = dont()
        .args(["show", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["curie"], "WB:P001");
}
