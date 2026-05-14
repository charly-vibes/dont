// Tests for dont-agent-help spec alignment.
//
// Spec: openspec/specs/dont-agent-help/spec.md
// Ticket: dont-nolt

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_in(dir: &TempDir) -> assert_cmd::assert::Assert {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
}

// ── Requirement: Help and teaching entry points ───────────────────────────────

/// Spec scenario: "tutorial help entry point"
/// WHEN the caller runs `dont help --tutorial`
/// THEN the output is the sequenced first-session walkthrough rather than
///      per-command reference text (exit code 0, non-empty content).
#[test]
fn help_tutorial_exits_successfully_with_tutorial_content() {
    dont()
        .args(["help", "--tutorial"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tutorial").or(predicates::str::contains("Tutorial")));
}

/// Spec scenario: "help topics listing"
/// WHEN the caller runs `dont help --topics`
/// THEN the output lists the available tutorial and how-to entry points.
#[test]
fn help_topics_lists_tutorial_and_howto_entries() {
    dont()
        .args(["help", "--topics"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tutorial").or(predicates::str::contains("howto")));
}

/// Spec scenario: "how-to topic selection"
/// WHEN the caller runs `dont help --howto harness-integration`
/// THEN the output is the corresponding goal-oriented guide.
#[test]
fn help_howto_harness_integration_prints_guide() {
    dont()
        .args(["help", "--howto", "harness-integration"])
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}

// ── Requirement: Orientation prompt contract ──────────────────────────────────

/// Spec requirement: orientation text MUST instruct the LLM to read
/// `data.remediation[0].command` and run it rather than guessing reformulations.
#[test]
fn agents_md_includes_remediation_driven_recovery_guidance() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("remediation") || agents.contains("data.remediation"),
        "AGENTS.md orientation should instruct reading data.remediation[0].command on refusal: {agents}"
    );
}

/// Spec requirement: orientation text MUST require harness fulfilment of spawn requests.
#[test]
fn agents_md_includes_spawn_harness_guidance() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("spawn"),
        "AGENTS.md orientation should mention spawn harness mechanism: {agents}"
    );
}

/// Spec requirement: orientation text MUST recommend `dont suggest-term` before
/// `define`, and MUST recommend `--label "<a noun phrase>"` alongside `--doc`.
#[test]
fn agents_md_includes_suggest_term_and_label_guidance() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("suggest-term"),
        "AGENTS.md orientation should recommend dont suggest-term before define: {agents}"
    );
    assert!(
        agents.contains("--label"),
        "AGENTS.md orientation should recommend --label alongside --doc: {agents}"
    );
}

/// Spec requirement: orientation text MUST point to `dont help --tutorial` for
/// the full teaching walkthrough.
#[test]
fn agents_md_points_to_help_tutorial() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("dont help --tutorial"),
        "AGENTS.md orientation should point to 'dont help --tutorial': {agents}"
    );
}

/// Spec requirement: orientation text MUST explain permissive vs strict mode.
#[test]
fn agents_md_explains_permissive_and_strict_modes() {
    let dir = TempDir::new().unwrap();
    init_in(&dir).success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("permissive") && agents.contains("strict"),
        "AGENTS.md orientation should explain permissive and strict modes: {agents}"
    );
}

// ── Requirement: Canonical teaching artifacts ─────────────────────────────────

/// Spec scenario: "how-to corpus covers the three named operator goals"
/// WHEN the caller browses the how-to guides
/// THEN the corpus includes guides for:
///   - authoring a project-specific rule
///   - integrating `dont` into a new harness
///   - recovering a corrupted `.dont/` store
#[test]
fn help_topics_includes_three_required_howto_topics() {
    let topics_out = dont()
        .args(["help", "--topics"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&topics_out);
    assert!(
        s.contains("rule") || s.contains("authoring"),
        "topics should include a rule-authoring how-to: {s}"
    );
    assert!(
        s.contains("harness") || s.contains("integration"),
        "topics should include a harness-integration how-to: {s}"
    );
    assert!(
        s.contains("recover") || s.contains("store"),
        "topics should include a store-recovery how-to: {s}"
    );
}

// ── Requirement: Help and teaching entry points — routing ─────────────────────

/// Spec scenario: "bare help lists commands and entry points"
/// WHEN the caller runs `dont help`
/// THEN the output lists the available commands and tutorial/how-to entry points.
#[test]
fn bare_help_lists_tutorial_and_howto_entry_points() {
    let output = dont()
        .arg("help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&output);
    assert!(
        s.contains("tutorial") || s.contains("--tutorial"),
        "bare help should mention tutorial entry point: {s}"
    );
    assert!(
        s.contains("howto") || s.contains("--howto"),
        "bare help should mention how-to entry point: {s}"
    );
}

use predicates::prelude::*;
