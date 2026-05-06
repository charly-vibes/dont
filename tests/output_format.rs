use assert_cmd::Command;
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

fn conclude_json(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<serde_json::Value>(&out).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

// --- --human flag: not JSON ---

#[test]
fn list_human_emits_plain_text_not_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "water is wet");

    let out = dont()
        .args(["list", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON, got: {text}"
    );
    assert!(text.contains("water is wet"), "output should contain statement");
    assert!(text.contains("unverified"), "output should contain status");
}

#[test]
fn list_empty_human_emits_no_claims_message() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON"
    );
    assert!(
        text.contains("no claims"),
        "empty list should say 'no claims', got: {text}"
    );
}

#[test]
fn conclude_human_emits_plain_text() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "all bachelors are unmarried", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON, got: {text}"
    );
    assert!(
        text.contains("all bachelors are unmarried"),
        "output should contain statement"
    );
    assert!(
        text.contains("unverified"),
        "output should contain status"
    );
}

#[test]
fn show_human_emits_plain_text() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_json(&dir, "gravity exists");

    let out = dont()
        .args(["show", &id, "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON, got: {text}"
    );
    assert!(text.contains("gravity exists"), "output should contain statement");
    assert!(text.contains("unverified"), "output should contain status");
}

#[test]
fn init_human_emits_plain_text() {
    let dir = TempDir::new().unwrap();

    let out = dont()
        .args(["init", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON, got: {text}"
    );
    assert!(text.contains("initialized"), "output should say initialized");
}

#[test]
fn prime_human_emits_plain_text() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "some claim");

    let out = dont()
        .args(["prime", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "output must not be JSON, got: {text}"
    );
    assert!(text.contains("unverified"), "output should contain status counts");
}

// --- --json flag still works ---

#[test]
fn list_json_flag_emits_json_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.trim_start().starts_with('{'),
        "output should be JSON with --json"
    );
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claims");
}

// --- json wins when both flags are set ---

#[test]
fn json_wins_over_human_when_both_set() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["list", "--json", "--human"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.trim_start().starts_with('{'),
        "--json should win over --human, got: {text}"
    );
}
