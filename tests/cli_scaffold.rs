use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_tracer_bullet_commands() {
    let mut cmd = Command::cargo_bin("dont").expect("binary exists");

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("conclude"))
        .stdout(predicate::str::contains("trust"))
        .stdout(predicate::str::contains("dismiss"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn help_flags_reflect_required_reason() {
    let mut trust = Command::cargo_bin("dont").expect("binary exists");
    trust
        .args(["trust", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Reason for doubt (required)"));

    let mut ignore = Command::cargo_bin("dont").expect("binary exists");
    ignore
        .args(["ignore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Substantive reason for ignoring (required; hedge-only reasons are refused)",
        ));
}
