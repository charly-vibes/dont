// Tests for dont-cli-surface spec alignment.
//
// Spec: openspec/specs/dont-cli-surface/spec.md
// Ticket: dont-mz46
//
// Covers spec claims not already exercised by existing tests:
//   - --plain overrides CLICOLOR_FORCE
//   - completions fish generates valid `complete -c` entries
//   - completions unknown shell exits 2 with usage error
//   - -r short flag for --reason on trust, ignore
//   - -e short flag for --evidence on flag, dismiss
//   - dont help <cmd> writes to stdout and includes usage + flags

use assert_cmd::Command;
use predicates::prelude::*;
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

// ── Requirement: Colour and terminal awareness ──────────────────────────────

/// Spec scenario: "plain flag overrides CLICOLOR_FORCE"
/// WHEN both --plain and CLICOLOR_FORCE=1 are set
/// THEN output contains no ANSI colour codes
#[test]
fn plain_flag_overrides_clicolor_force() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "colour override test claim");

    let out = dont()
        .args(["list", "--plain"])
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
        "--plain must suppress ANSI even when CLICOLOR_FORCE=1, got: {text:?}"
    );
}

// ── Requirement: Shell completion generation ────────────────────────────────

/// Spec scenario: "completions generate valid shell script"
/// WHEN `dont completions fish` is invoked
/// THEN stdout contains a valid fish completion script with `complete -c` entries
#[test]
fn completions_fish_contains_complete_c_entries() {
    dont()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c dont"));
}

/// Spec scenario: "completions for unknown shell exits with usage error"
/// WHEN `dont completions <unknown-shell>` is invoked
/// THEN the process exits 2 with a usage error listing supported shell names
#[test]
fn completions_unknown_shell_exits_2() {
    dont()
        .args(["completions", "unknownshell"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("bash").or(predicate::str::contains("zsh")));
}

// ── Requirement: No short-flag conflicts ────────────────────────────────────

/// Spec: widely reused per-command short flags: -r for --reason on trust
#[test]
fn trust_short_r_flag_transitions_claim_to_doubted() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "short flag trust test");

    let out = dont()
        .args(["trust", &id, "-r", "contradicted by data", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "doubted");
}

/// Spec: widely reused per-command short flags: -r for --reason on ignore
#[test]
fn ignore_short_r_flag_transitions_claim_to_ignored() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "short flag ignore test");

    let out = dont()
        .args(["ignore", &id, "-r", "out of scope for this audit", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "ignored");
}

/// Spec: widely reused per-command short flags: -e for --evidence on flag
#[test]
fn flag_short_e_flag_transitions_claim_to_verified() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "short flag flag test");

    let out = dont()
        .args(["flag", &id, "-e", "https://example.com/evidence", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "verified");
}

/// Spec: widely reused per-command short flags: -e for --evidence on dismiss
#[test]
fn dismiss_short_e_flag_transitions_claim_to_verified() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "short flag dismiss test");

    let out = dont()
        .args(["dismiss", &id, "-e", "https://example.com/ref", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "verified");
}

// ── Requirement: Help surface structure ────────────────────────────────────

/// Spec scenario: "command help shows usage and flags"
/// WHEN `dont help <cmd>` is invoked
/// THEN the output is written to stdout and includes the command's usage line,
///      flags (including short flags), and a description
#[test]
fn help_cmd_writes_to_stdout_with_usage_and_flags() {
    let out = dont()
        .args(["help", "trust"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("Usage"),
        "help trust stdout must include 'Usage', got: {text}"
    );
    assert!(
        text.contains("--reason") || text.contains("-r"),
        "help trust stdout must include --reason / -r flag, got: {text}"
    );
}

/// Spec scenario: "bare help lists available commands"
/// WHEN `dont help` is invoked with no arguments
/// THEN the output lists all available subcommands with brief descriptions
#[test]
fn help_lists_all_core_subcommands() {
    let out = dont()
        .arg("help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    for cmd in &["conclude", "trust", "flag", "ignore", "show", "list", "ground"] {
        assert!(
            text.contains(cmd),
            "bare help must list '{cmd}' subcommand, got: {text}"
        );
    }
}
