mod common;

use common::{conclude_claim as conclude_json, dont, init_dir};
use tempfile::TempDir;

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

// --- default output (no flags) is human-readable ---

#[test]
fn list_default_emits_human_not_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "water is wet");

    let out = dont()
        .args(["list"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "default output must not be JSON, got: {text}"
    );
    assert!(text.contains("water is wet"), "output should contain statement");
}

#[test]
fn conclude_default_emits_human_not_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "the sky is blue"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "default output must not be JSON, got: {text}"
    );
    assert!(text.contains("the sky is blue"), "output should contain statement");
}

// --- --plain flag disables ANSI; color env vars respected ---

#[test]
fn plain_flag_emits_no_ansi_escape_codes() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "all swans are white");

    let out = dont()
        .args(["list", "--plain"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.contains('\x1b'),
        "--plain output must not contain ANSI escape codes, got: {text:?}"
    );
    assert!(text.contains("all swans are white"), "output should contain statement");
}

#[test]
fn no_color_env_disables_ansi() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "entropy increases");

    let out = dont()
        .args(["list"])
        .env("DONT_DIR", dir.path())
        .env("NO_COLOR", "1")
        .env("CLICOLOR_FORCE", "0")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.contains('\x1b'),
        "NO_COLOR=1 must disable ANSI escape codes, got: {text:?}"
    );
}

#[test]
fn clicolor_force_enables_ansi() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "constants are constant");

    let out = dont()
        .args(["list"])
        .env("DONT_DIR", dir.path())
        .env("CLICOLOR_FORCE", "1")
        .env("NO_COLOR", "")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains('\x1b'),
        "CLICOLOR_FORCE=1 must enable ANSI escape codes, got: {text:?}"
    );
}

#[test]
fn no_color_flag_disables_ansi_even_when_force_is_set() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "forced colour can be disabled");

    let out = dont()
        .args(["list", "--no-color"])
        .env("DONT_DIR", dir.path())
        .env("CLICOLOR_FORCE", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.contains('\x1b'),
        "--no-color must suppress ANSI even when CLICOLOR_FORCE=1, got: {text:?}"
    );
}

#[test]
fn color_flag_enables_ansi_even_when_no_color_env_is_set() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_json(&dir, "flag overrides no_color env");

    let out = dont()
        .args(["list", "--color"])
        .env("DONT_DIR", dir.path())
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains('\x1b'),
        "--color must force ANSI even when NO_COLOR=1, got: {text:?}"
    );
}

// --- evidence rendering (dont-08l) ---

#[test]
fn show_human_renders_atoms_section() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_json(&dir, "claim with atoms");

    dont()
        .args(["atom", "define", &id, "--text", "sub-check alpha", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id])
        .env("DONT_DIR", dir.path())
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("atoms:"),
        "human show must include 'atoms:' section, got: {text}"
    );
    assert!(
        text.contains("sub-check alpha"),
        "human show must include atom text, got: {text}"
    );
}

#[test]
fn show_human_renders_hypotheses_section() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_json(&dir, "claim with hypotheses");

    dont()
        .args(["hypothesis", "add", &id, "--text", "maybe it's the moon", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id])
        .env("DONT_DIR", dir.path())
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("hypotheses:"),
        "human show must include 'hypotheses:' section, got: {text}"
    );
    assert!(
        text.contains("maybe it's the moon"),
        "human show must include hypothesis text, got: {text}"
    );
}

#[test]
fn show_json_includes_atom_and_hypothesis_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_json(&dir, "claim for json fields check");

    dont()
        .args(["atom", "define", &id, "--text", "an atom", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    dont()
        .args(["hypothesis", "add", &id, "--text", "a hypothesis", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let atoms = v["data"]["atoms"].as_array().expect("data.atoms must be array");
    assert_eq!(atoms.len(), 1);
    assert_eq!(atoms[0]["idx"], 0);
    assert_eq!(atoms[0]["text"], "an atom");
    assert_eq!(atoms[0]["status"], "unverified");

    let hyps = v["data"]["hypotheses"].as_array().expect("data.hypotheses must be array");
    assert_eq!(hyps.len(), 1);
    assert_eq!(hyps[0]["idx"], 0);
    assert_eq!(hyps[0]["text"], "a hypothesis");
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
