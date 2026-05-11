use assert_cmd::Command;
use predicates::prelude::*;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

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
fn completions_bash_includes_forget_not_lock() {
    let output = dont()
        .args(["completions", "bash"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let script = String::from_utf8_lossy(&output);
    assert!(
        script.contains("forget"),
        "bash completions should include 'forget' command"
    );
}
