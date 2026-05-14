//! Shared test helpers — imported by integration test files via `mod common`.
//!
//! Previously `dont()`, `init_dir()`, and `conclude_claim()` were copy-pasted
//! verbatim in 27+ test files. Changes to the CLI invocation or env-var
//! conventions now only need to land here.

#![allow(dead_code)]

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// Return a `Command` pointing at the `dont` binary under test.
pub fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

/// Initialise a temporary directory as a dont project.
///
/// Sets `DONT_DIR` to `dir.path()` (the temp dir itself), which is the
/// canonical convention used by the vast majority of integration tests.
pub fn init_dir(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

/// Run `dont conclude <statement> --json` and return the new claim's ID.
pub fn conclude_claim(dir: &TempDir, statement: &str) -> String {
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
