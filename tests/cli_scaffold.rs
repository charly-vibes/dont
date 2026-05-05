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
