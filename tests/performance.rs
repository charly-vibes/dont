/// Performance tests: 1 000-claim graph must not degrade catastrophically.
///
/// Pass criteria (ticket dont-58qc):
///   - No panic, no hang, no OOM
///   - `list`, `show <id>`, `why <id>` each complete in < 5 seconds
///   - Scaling is linear (not exponential)
mod common;

use common::{TestProject, conclude_claim, dont, init_project};
use std::time::Instant;

const CLAIM_COUNT: usize = 1_000;
const LIMIT_SECS: u64 = 5;

/// These tests spawn the `dont` binary CLAIM_COUNT times each — ~85s total on
/// every run. They run in CI (GitHub Actions sets `CI`), but are skipped
/// locally so `cargo test` stays fast (ticket dont-a4lk / gh #35).
fn skip_outside_ci() -> bool {
    if std::env::var_os("CI").is_some() {
        return false;
    }
    eprintln!("skipping performance test outside CI (spawns {CLAIM_COUNT} processes)");
    true
}

/// Create CLAIM_COUNT claims in `dir`, return the last claim's ID.
fn populate(dir: &TestProject) -> String {
    let mut last_id = String::new();
    for i in 0..CLAIM_COUNT {
        last_id = conclude_claim(dir, &format!("performance test claim number {i}"));
    }
    last_id
}

#[test]
fn list_1000_claims_completes_within_5s() {
    if skip_outside_ci() {
        return;
    }
    let dir = init_project();
    populate(&dir);

    let start = Instant::now();
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < LIMIT_SECS,
        "dont list took {elapsed:?}, expected < {LIMIT_SECS}s"
    );
}

#[test]
fn show_in_1000_claim_graph_completes_within_5s() {
    if skip_outside_ci() {
        return;
    }
    let dir = init_project();
    let id = populate(&dir);

    let start = Instant::now();
    dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < LIMIT_SECS,
        "dont show took {elapsed:?}, expected < {LIMIT_SECS}s"
    );
}

#[test]
fn why_in_1000_claim_graph_completes_within_5s() {
    if skip_outside_ci() {
        return;
    }
    let dir = init_project();
    let id = populate(&dir);

    let start = Instant::now();
    dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < LIMIT_SECS,
        "dont why took {elapsed:?}, expected < {LIMIT_SECS}s"
    );
}
