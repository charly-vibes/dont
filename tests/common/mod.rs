//! Shared test helpers — imported by integration test files via `mod common`.
//!
//! Previously `dont()`, `init_dir()`, and `conclude_claim()` were copy-pasted
//! verbatim in 27+ test files. Changes to the CLI invocation or env-var
//! conventions now only need to land here.

#![allow(dead_code)]

use assert_cmd::Command;
use genesis::fixture::Fixture;
use serde_json::Value;

/// A dont project fixture backed by `genesis::fixture::Fixture` (dont-7xv8).
///
/// Wraps the genesis fixture so tests keep the familiar `dir.path()` access
/// pattern while temp-dir lifecycle and cleanup are handled by genesis.
pub struct TestProject {
    fixture: Fixture,
}

impl TestProject {
    /// Root of the temporary project directory (passed as `DONT_DIR`).
    pub fn path(&self) -> &std::path::Path {
        self.fixture.root()
    }
}

impl AsRef<std::path::Path> for TestProject {
    fn as_ref(&self) -> &std::path::Path {
        self.fixture.root()
    }
}

/// Create a temporary directory initialised as a dont project.
///
/// Replaces the former `TempDir::new()` + `init_dir(&dir)` two-step with a
/// single genesis-fixture-backed call. Runs `dont init --json` and returns
/// a handle whose `path()` is the canonical `DONT_DIR`.
pub fn init_project() -> TestProject {
    let fixture = Fixture::new().build().expect("build genesis fixture");
    let project = TestProject { fixture };
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", project.path())
        .assert()
        .success();
    project
}

/// Return a `Command` pointing at the `dont` binary under test.
///
/// Redirects `XDG_CACHE_HOME` to a temp dir so error-scratch writes (the
/// genesis::guide::ErrorSink contract, written on every non-zero exit) do
/// not pollute the developer's real `~/.cache/dont/` during `cargo test`.
pub fn dont() -> Command {
    let cache = std::env::temp_dir().join("dont-test-cache");
    let mut cmd = Command::cargo_bin("dont").unwrap();
    cmd.env("XDG_CACHE_HOME", &cache);
    cmd
}

/// Run `dont conclude <statement> --json` and return the new claim's ID.
///
/// Accepts any path-like handle (`&TempDir` or `&TestProject`).
pub fn conclude_claim<P: AsRef<std::path::Path>>(dir: P, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.as_ref())
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
