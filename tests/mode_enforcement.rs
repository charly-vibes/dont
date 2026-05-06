use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_permissive(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn init_strict(dir: &TempDir) {
    dont()
        .args(["init", "--strict", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn define_term(dir: &TempDir, curie: &str) {
    dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn conclude_with_resolved_depends_on_succeeds_in_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);
    define_term(&dir, "WB:P001");

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
    let deps = v["data"]["depends_on"].as_array().unwrap();
    assert!(deps.iter().any(|d| d.as_str().unwrap_or("").contains("WB:P001") || d == "WB:P001"));
}

#[test]
fn conclude_with_unresolved_depends_on_succeeds_in_permissive_mode() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
    let warnings = v["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w["rule_name"] == "unresolved-term-ref"),
        "expected unresolved-term-ref warning, got: {:?}", warnings
    );
}

#[test]
fn conclude_with_unresolved_depends_on_is_refused_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "unresolved-term-ref");
}

#[test]
fn conclude_with_resolved_depends_on_succeeds_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);
    define_term(&dir, "WB:P001");

    let output = dont()
        .args(["conclude", "The system uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn conclude_without_depends_on_succeeds_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);

    let output = dont()
        .args(["conclude", "A self-contained claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn conclude_multiple_depends_on_one_unresolved_is_refused_in_strict_mode() {
    let dir = TempDir::new().unwrap();
    init_strict(&dir);
    define_term(&dir, "WB:P001");

    let output = dont()
        .args([
            "conclude", "Uses both terms",
            "--depends-on", "WB:P001",
            "--depends-on", "WB:P002",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "unresolved-term-ref");
}

#[test]
fn conclude_carries_depends_on_in_view() {
    let dir = TempDir::new().unwrap();
    init_permissive(&dir);
    define_term(&dir, "WB:P001");

    let output = dont()
        .args(["conclude", "Uses WB:P001", "--depends-on", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    let deps = v["data"]["depends_on"].as_array().unwrap();
    assert!(!deps.is_empty(), "expected depends_on to be populated");
}
