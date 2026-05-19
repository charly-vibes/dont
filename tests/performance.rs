/// Performance tests: 1 000-claim graph must not degrade catastrophically.
///
/// Pass criteria (ticket dont-58qc):
///   - No panic, no hang, no OOM
///   - `list`, `show <id>`, `why <id>` each complete in < 5 seconds
///   - Scaling is linear (not exponential)
mod common;

use common::{conclude_claim, dont, init_dir};
use std::time::Instant;
use tempfile::TempDir;

const CLAIM_COUNT: usize = 1_000;
const LIMIT_SECS: u64 = 5;

/// Create CLAIM_COUNT claims in `dir`, return the last claim's ID.
fn populate(dir: &TempDir) -> String {
    let mut last_id = String::new();
    for i in 0..CLAIM_COUNT {
        last_id = conclude_claim(dir, &format!("performance test claim number {i}"));
    }
    last_id
}

#[test]
fn list_1000_claims_completes_within_5s() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
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
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
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
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
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
