use assert_cmd::Command;
use predicates::prelude::*;

const COMMANDS: &[&str] = &["init", "conclude", "trust", "dismiss", "show", "list"];

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
fn tracer_bullet_subcommands_are_stubbed() {
    for command in COMMANDS {
        let mut cmd = Command::cargo_bin("dont").expect("binary exists");
        let args = match *command {
            "init" | "list" => vec![*command],
            "conclude" => vec![*command, "unsupported assertions need grounding"],
            "trust" => vec![
                *command,
                "claim:01HX05A9K8VP",
                "--reason",
                "Evidence contradicts the claim",
            ],
            "dismiss" => vec![
                *command,
                "claim:01HX05A9K8VP",
                "--evidence",
                "https://example.test/evidence",
            ],
            "show" => vec![*command, "claim:01HX05A9K8VP"],
            _ => unreachable!(),
        };

        cmd.args(args)
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "{command}: not implemented"
            )));
    }
}
