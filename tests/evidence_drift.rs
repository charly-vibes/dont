mod common;

use common::dont;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

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

fn define_term(root: &Path, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
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
        .args(["flag", id, "--file", file, "--lines", lines, "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success();
}

fn dismiss_file_value(root: &Path, id: &str, file: &str, lines: &str) -> Value {
    let out = dont()
        .args(["flag", id, "--file", file, "--lines", lines, "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .output()
        .unwrap()
        .stdout;
    serde_json::from_slice(&out).unwrap()
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

fn prime(root: &Path) -> Value {
    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .output()
        .unwrap()
        .stdout;
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
fn dismiss_refuses_symlink_escape_outside_project_root() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    let outside = dir.path().join("outside");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    init_project(&root);
    std::fs::write(outside.join("secret.txt"), "secret\n").unwrap();
    std::os::unix::fs::symlink(outside.join("secret.txt"), root.join("link.txt")).unwrap();
    let id = conclude_claim(&root, "symlink should not escape project root");

    let v = dismiss_file_value(&root, &id, "link.txt", "1");

    assert_eq!(v["ok"], false, "symlink escape should be refused: {v}");
    assert_eq!(v["data"]["code"], "path-escapes-root");
}

#[test]
fn dismiss_refuses_broken_symlink_locator_before_it_can_later_escape() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    let outside_target = dir.path().join("later-secret.txt");
    std::fs::create_dir_all(&root).unwrap();
    init_project(&root);
    std::os::unix::fs::symlink(&outside_target, root.join("link.txt")).unwrap();
    let id = conclude_claim(&root, "broken symlink should not become future evidence");

    let v = dismiss_file_value(&root, &id, "link.txt", "1");

    assert_eq!(v["ok"], false, "broken symlink should be refused: {v}");
    assert_eq!(v["data"]["code"], "unreadable-evidence");
    std::fs::write(outside_target, "secret\n").unwrap();
    let shown = show(&root, &id);
    assert!(shown["data"]["evidence"].as_array().unwrap().is_empty());
}

#[test]
fn dismiss_with_excerpt_without_lines_audits_excerpt_presence() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "alpha beta gamma\n").unwrap();
    let id = conclude_claim(&root, "README mentions beta");

    let v = dont()
        .args([
            "flag",
            &id,
            "--file",
            "README.md",
            "--excerpt",
            "beta",
            "--json",
        ])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&v).unwrap();
    assert_eq!(v["data"]["evidence"][0]["audit"]["status"], "current");

    std::fs::write(root.join("README.md"), "alpha gamma\n").unwrap();
    let shown = show(&root, &id);
    assert_eq!(shown["data"]["evidence"][0]["audit"]["status"], "drifted");
}

#[test]
fn dismiss_refuses_invalid_line_spans() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "line\n").unwrap();

    for lines in ["0", "0-1", "1-0"] {
        let id = conclude_claim(&root, &format!("line span {lines} is invalid"));
        let v = dismiss_file_value(&root, &id, "README.md", lines);

        assert_eq!(v["ok"], false, "line span {lines} should be refused: {v}");
        assert_eq!(v["data"]["code"], "invalid-line-span");
    }
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
fn term_show_and_why_project_repo_locator_audit() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "original\n").unwrap();
    let id = define_term(&root, "EX:T");
    dismiss_file(&root, &id, "README.md", "1");
    std::fs::write(root.join("README.md"), "changed\n").unwrap();

    let shown = show(&root, &id);
    let show_locator = shown["data"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == "repo-file")
        .unwrap();
    assert_eq!(show_locator["audit"]["status"], "drifted");

    let explained = why(&root, &id);
    let why_locator = explained["data"]["entity"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == "repo-file")
        .unwrap();
    assert_eq!(why_locator["audit"]["status"], "drifted");
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

/// Regression guard: an anchor value containing ANSI escape sequences must not
/// pass through to terminal output.  Attackers could supply a crafted anchor
/// such as `\x1b[31mevil\x1b[0m` via `--anchor` to inject fake colour into the
/// `dont show` human-readable output.  `strip_control_chars` in the formatter
/// must remove all control characters before they reach stdout.
#[test]
fn show_human_output_strips_ansi_escape_from_anchor() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "target section content\n").unwrap();
    let id = conclude_claim(&root, "README has target section");

    // Attach a repo-file locator whose anchor contains a raw ANSI escape.
    let evil_anchor = "\x1b[31mevil\x1b[0m";
    dont()
        .args([
            "flag",
            &id,
            "--file",
            "README.md",
            "--anchor",
            evil_anchor,
            "--json",
        ])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success();

    // `dont show` in human mode (no --json flag) must not emit raw ESC bytes.
    let output = dont()
        .args(["show", &id])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(
        !stdout.contains('\x1b'),
        "human output must not contain raw ESC (ANSI injection); got: {stdout:?}"
    );
    // The sanitized anchor text should still appear (minus the escape sequences).
    // After stripping ESC the remaining visible text "evil" must be present.
    assert!(
        stdout.contains("evil"),
        "anchor text content should appear after stripping control chars; got: {stdout:?}"
    );
    // The locator display should include the '#' anchor separator.
    assert!(
        stdout.contains("README.md#"),
        "anchor should appear with '#' separator in locator display; got: {stdout:?}"
    );
}

#[test]
fn prime_counts_drifted_evidence_in_assessment_counts() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    init_project(&root);
    std::fs::write(root.join("README.md"), "original content\n").unwrap();

    let id = conclude_claim(&root, "README has original content");
    dismiss_file(&root, &id, "README.md", "1");

    // Before drift: drifted_evidence should be 0
    let v = prime(&root);
    assert_eq!(
        v["data"]["assessment_counts"]["drifted_evidence"], 0,
        "no drift yet: {v}"
    );

    // Mutate the file so the fingerprint drifts
    std::fs::write(root.join("README.md"), "changed content\n").unwrap();

    // After drift: drifted_evidence should be 1
    let v = prime(&root);
    assert_eq!(
        v["data"]["assessment_counts"]["drifted_evidence"], 1,
        "one drifted claim: {v}"
    );
}
