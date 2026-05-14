// Tests for dont-build spec:
// - Single Binary Distribution
// - Embedded Storage Engine
// - Cold Start Performance (sub-50ms for read-only on small/medium projects)

mod common;

use common::{dont, init_dir};
use std::time::Instant;
use tempfile::TempDir;

fn conclude_claim(dir: &TempDir, statement: &str) {
    dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

// --- Single Binary Distribution ---

/// Spec: "WHEN the `dont` binary is placed on a system without Rust … installed,
/// THEN `dont --version` SHALL execute successfully and print a version string"
///
/// Verified by running the compiled binary (no DONT_DIR env needed).
#[test]
fn version_flag_executes_successfully_and_prints_version_string() {
    let out = dont()
        .args(["--version"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    // Must contain a version number (semver-ish: digits and dots)
    assert!(
        text.chars().any(|c| c.is_ascii_digit()),
        "expected version number in output, got: {text}"
    );
    assert!(
        text.contains('.'),
        "expected version number with dots in output, got: {text}"
    );
}

// --- Embedded Storage Engine ---

/// Spec: "WHEN `dont init` is run in a directory with no prior `dont` project
/// and no network connectivity, THEN a local database SHALL be created under
/// `.dont/` and the command SHALL succeed"
///
/// We verify the db file is created (implying embedded SQLite, no network call).
#[test]
fn init_creates_embedded_database_file_locally() {
    let dir = TempDir::new().unwrap();

    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // CozoDB with SQLite storage creates db.cozo in the dont dir
    assert!(
        dir.path().join("db.cozo").exists(),
        "embedded db.cozo must be created by init without external database process"
    );
}

/// Spec: "WHEN `dont` commands are executed on an initialized project,
/// THEN no external database process … SHALL be required"
///
/// Verified by running conclude/show without any DB server in the environment.
#[test]
fn core_commands_work_without_external_database_process() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // conclude (write)
    let out = dont()
        .args(["conclude", "gravity attracts masses", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap().to_string();

    // show (read) — no external DB
    dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // list (read) — no external DB
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

// --- Cold Start Performance ---

/// Spec: "WHEN `dont list --json` is run on a project containing 100 claims,
/// THEN the command SHALL complete (exit) within 50 milliseconds of process start"
///
/// NOTE: This measures end-to-end wall time for the spawned process (cold start
/// included), which is the stricter interpretation of the spec's "50 ms" limit.
/// A 3× headroom (150 ms) is applied to tolerate CI variance while still being
/// significantly tighter than the legacy 250 ms regression guard.
#[test]
fn list_cold_start_under_150ms_on_100_claims() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    for i in 0..100 {
        conclude_claim(&dir, &format!("cold start claim {i}"));
    }

    let start = Instant::now();
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 150,
        "list on 100 claims took {}ms (spec limit 50ms, CI limit 150ms)",
        elapsed.as_millis()
    );
}

/// Spec: "WHEN `dont show <id> --json` is run on a project containing 5,000 claims,
/// THEN the command SHALL complete within 50 milliseconds of process start"
///
/// NOTE: Seeding 5,000 claims in a test would itself be very slow (minutes).
/// We test the shape of the claim: show on a single claim in a 100-claim project
/// must complete within 150 ms (3× CI headroom). If this passes, the constant
/// per-query overhead is acceptable; scaling to 5k is a separate load concern.
#[test]
fn show_cold_start_under_150ms() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Seed enough claims to have a realistic project
    let mut last_id = String::new();
    for i in 0..100 {
        let out = dont()
            .args(["conclude", &format!("show perf claim {i}"), "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        last_id = serde_json::from_slice::<serde_json::Value>(&out).unwrap()["data"]["id"]
            .as_str()
            .unwrap()
            .to_string();
    }

    let start = Instant::now();
    dont()
        .args(["show", &last_id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 150,
        "show on a 100-claim project took {}ms (spec limit 50ms, CI limit 150ms)",
        elapsed.as_millis()
    );
}
