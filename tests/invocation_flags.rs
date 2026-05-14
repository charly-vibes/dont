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

// --- --version --json ---

#[test]
fn version_json_emits_structured_envelope() {
    let out = dont()
        .args(["--version", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "version");
    assert!(v["data"]["cli_version"].is_string());
    assert!(v["data"]["envelope_version"].is_string());
}

#[test]
fn version_without_json_prints_plain_version() {
    let out = dont()
        .args(["--version"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    // Should contain a semver-ish string, not JSON
    assert!(!text.trim_start().starts_with('{'), "expected plain text, got JSON: {text}");
    assert!(text.contains('.'), "expected version number with dots, got: {text}");
}

// --- $DONT_AUTHOR and --author ---

#[test]
fn dont_author_env_is_reflected_in_envelope_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .env("DONT_AUTHOR", "alice")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["meta"]["author"], "alice");
}

#[test]
fn author_flag_overrides_dont_author_env() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json", "--author", "bob"])
        .env("DONT_DIR", dir.path())
        .env("DONT_AUTHOR", "alice")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["meta"]["author"], "bob");
}

#[test]
fn author_short_flag_works() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json", "-a", "carol"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["meta"]["author"], "carol");
}

// --- --direct ---

#[test]
fn direct_flag_is_accepted_without_error() {
    let dir = TempDir::new().unwrap();
    dont()
        .args(["init", "--json", "--direct"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn lock_without_project_emits_error_envelope() {
    // lock is now a real alias for forget; without a project it fails with no-project-found
    let out = dont()
        .args(["--json", "lock", "claim:123"])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["envelope_version"], "0.2");
    // Error is about missing project, not deprecation
    assert!(v["data"]["code"].as_str().unwrap_or("") != "");
}

// --- -j short flag for --json ---

#[test]
fn j_short_flag_emits_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "-j"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
}
