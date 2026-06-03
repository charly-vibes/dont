mod common;

use common::dont;
use predicates::prelude::*;

#[test]
fn help_uses_consistent_lowercase_positional_names() {
    dont()
        .args(["show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Usage: dont show [OPTIONS] <entity-id>",
        ));

    dont()
        .args(["define", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Usage: dont define [OPTIONS] [curie]",
        ));

    dont()
        .args(["atom", "dismiss", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Usage: dont atom dismiss [OPTIONS] <claim-id> <idx>",
        ));

    dont()
        .args(["import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Usage: dont import [OPTIONS] <adapter> [arg]...",
        ));
}

#[test]
fn extra_positional_args_produce_actionable_usage_errors() {
    dont()
        .args(["show", "claim:abc", "extra"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "unexpected argument 'extra' found",
        ))
        .stderr(predicate::str::contains(
            "Usage: dont show [OPTIONS] <entity-id>",
        ))
        .stderr(predicate::str::contains(
            "For more information, try '--help'",
        ));

    dont()
        .args([
            "atom",
            "define",
            "claim:abc",
            "extra",
            "--text",
            "check source",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "unexpected argument 'extra' found",
        ))
        .stderr(predicate::str::contains(
            "Usage: dont atom define [OPTIONS] --text <TEXT> <claim-id>",
        ));
}
