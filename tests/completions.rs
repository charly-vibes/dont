mod common;

use common::dont;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn completions_bash_produces_nonempty_output() {
    dont()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_bash_includes_conclude_for_tab_completion() {
    dont()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conclude"));
}

#[test]
fn completions_zsh_produces_nonempty_output() {
    dont()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_produces_nonempty_output() {
    dont()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_includes_conclude() {
    dont()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conclude"));
}

#[test]
fn completions_powershell_produces_nonempty_output() {
    dont()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_elvish_produces_nonempty_output() {
    dont()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_bash_includes_both_canonical_commands_and_aliases() {
    let output = dont()
        .args(["completions", "bash"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let script = String::from_utf8_lossy(&output);
    for name in ["flag", "dismiss", "forget", "lock"] {
        assert!(
            script.contains(name),
            "bash completions should include '{name}' command"
        );
    }
}

#[test]
fn completions_bash_json_returns_envelope_with_script() {
    let output = dont()
        .args(["completions", "bash", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["shell"], "bash");
    let script = v["data"]["script"].as_str().unwrap();
    assert!(
        script.contains("conclude"),
        "script should include conclude"
    );
}

#[test]
fn completions_json_envelope_is_parseable_for_all_shells() {
    for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = dont()
            .args(["completions", shell, "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(v["ok"], true, "shell {shell} should return ok envelope");
        assert_eq!(v["data"]["shell"], *shell);
        assert!(
            v["data"]["script"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "shell {shell} script should be non-empty"
        );
    }
}

#[test]
fn completions_json_envelope_kind_is_dont_completions_for_all_shells() {
    for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = dont()
            .args(["completions", shell, "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(
            v["envelope_kind"], "dont-completions",
            "shell {shell} should emit envelope_kind=dont-completions"
        );
        assert!(
            v["error"].is_null(),
            "shell {shell} error must be null on success"
        );
    }
}

#[test]
fn completions_plain_output_is_not_json_when_no_json_flag() {
    for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = dont()
            .args(["completions", shell, "--human"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let text = String::from_utf8_lossy(&output);
        assert!(
            serde_json::from_slice::<Value>(&output).is_err(),
            "shell {shell} plain output should not be JSON, got: {text:.80}"
        );
    }
}

#[test]
fn completions_json_script_contains_subcommands_for_all_shells() {
    for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = dont()
            .args(["completions", shell, "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&output).unwrap();
        let script = v["data"]["script"].as_str().unwrap_or("");
        assert!(
            script.contains("conclude"),
            "shell {shell} JSON script should contain 'conclude'"
        );
    }
}
