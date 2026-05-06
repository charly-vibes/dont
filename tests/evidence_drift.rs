use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_project(root: &Path) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success();
}

fn conclude_claim(root: &Path, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", root.join(".dont"))
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

fn dismiss_file(root: &Path, id: &str, file: &str, lines: &str) {
    dont()
        .args(["dismiss", id, "--file", file, "--lines", lines, "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success();
}

fn show(root: &Path, id: &str) -> Value {
    let out = dont()
        .args(["show", id, "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

fn verify_evidence(root: &Path, id: &str) -> Value {
    let out = dont()
        .args(["verify-evidence", id, "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

fn why(root: &Path, id: &str) -> Value {
    let out = dont()
        .args(["why", id, "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

#[test]
fn show_projects_current_repo_locator_excerpt_and_fingerprint_audit() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "title\nsource line\nmore\n").unwrap();
    let id = conclude_claim(&root, "README has a source line");

    dismiss_file(&root, &id, "README.md", "2");

    let v = show(&root, &id);
    let locator = v["data"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == "repo-file")
        .unwrap();
    assert_eq!(locator["path"], "README.md");
    assert_eq!(locator["line_start"], 2);
    assert_eq!(locator["line_end"], 2);
    assert_eq!(locator["excerpt"], "source line");
    assert!(
        locator["fingerprint"]
            .as_str()
            .unwrap()
            .starts_with("fnv1a64:")
    );
    assert_eq!(locator["audit"]["status"], "current");
}

#[test]
fn show_reports_fingerprint_mismatch_as_drift_without_status_mutation() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "title\noriginal\nmore\n").unwrap();
    let id = conclude_claim(&root, "README has original text");
    dismiss_file(&root, &id, "README.md", "2");

    std::fs::write(root.join("README.md"), "title\nchanged\nmore\n").unwrap();

    let v = show(&root, &id);
    let locator = v["data"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == "repo-file")
        .unwrap();
    assert_eq!(v["data"]["status"], "verified");
    assert_eq!(locator["audit"]["status"], "drifted");
    assert!(
        locator["audit"]["detail"]
            .as_str()
            .unwrap()
            .contains("fingerprint")
    );
}

#[test]
fn why_projects_same_repo_locator_audit_contract_as_show() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "title\noriginal\nmore\n").unwrap();
    let id = conclude_claim(&root, "README has original text");
    dismiss_file(&root, &id, "README.md", "2");
    std::fs::write(root.join("README.md"), "title\nchanged\nmore\n").unwrap();

    let v = why(&root, &id);

    assert_eq!(v["envelope_kind"], "why");
    let locator = v["data"]["entity"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == "repo-file")
        .unwrap();
    assert_eq!(locator["audit"]["status"], "drifted");
}

#[test]
fn verify_evidence_reports_missing_line_span_for_repo_locator() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "one\ntwo\nthree\n").unwrap();
    let id = conclude_claim(&root, "README has a third line");
    dismiss_file(&root, &id, "README.md", "3");

    std::fs::write(root.join("README.md"), "one\ntwo\n").unwrap();

    let v = verify_evidence(&root, &id);
    assert_eq!(v["data"]["status"], "verified");
    let result = &v["data"]["results"][0];
    assert_eq!(result["outcome"], "unresolved");
    assert_eq!(result["locator"]["path"], "README.md");
    assert!(result["detail"].as_str().unwrap().contains("line span"));
}
