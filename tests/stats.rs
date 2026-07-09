mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn stats(dir: &TempDir) -> Value {
    let out = dont()
        .args(["stats", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<Value>(&out).unwrap()
}

fn stats_with_args(dir: &TempDir, extra: &[&str]) -> Value {
    let out = dont()
        .args(["stats", "--json"])
        .args(extra)
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();
    serde_json::from_slice::<Value>(&out.stdout).unwrap()
}

#[test]
fn stats_returns_stats_envelope_kind() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = stats(&dir);
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "stats");
}

#[test]
fn stats_empty_store_has_zero_counts_and_null_rate() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = stats(&dir);
    let data = &v["data"];
    assert_eq!(
        data["idle_skill"], true,
        "idle_skill must be true on empty store"
    );
    assert_eq!(data["dedup_refusal_count"], 0);
    assert_eq!(data["caught_contradiction_count"], 0);
    assert!(
        data["verb_counts"].as_object().unwrap().is_empty(),
        "verb_counts must be empty map"
    );
    assert!(
        data["claim_verification_rate"].is_null(),
        "claim_verification_rate must be null when no claims"
    );
}

#[test]
fn stats_after_conclude_shows_verb_count() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "the cache expires after 60 seconds");
    let v = stats(&dir);
    let data = &v["data"];
    assert_eq!(data["verb_counts"]["conclude"], 1);
    assert_eq!(data["idle_skill"], false);
}

#[test]
fn stats_verb_counts_omits_absent_verbs() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "the service starts on port 8080");
    let v = stats(&dir);
    let data = &v["data"];
    // only conclude should appear; trust/flag/lock/etc absent
    let vc = data["verb_counts"].as_object().unwrap();
    assert!(
        !vc.contains_key("trust"),
        "trust must be absent when no trust events occurred"
    );
    assert_eq!(data["verb_counts"]["conclude"], 1);
}

#[test]
fn stats_claim_verification_rate_is_correct_ratio() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "alpha claim");
    let _id2 = conclude_claim(&dir, "beta claim");
    let _id3 = conclude_claim(&dir, "gamma claim");
    // flag id1 to verify it (flag = "dont flag as a concern" = verify in dont semantics)
    dont()
        .args([
            "flag",
            &id1,
            "--evidence",
            "https://example.com/prod-logs",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = stats(&dir);
    let rate = v["data"]["claim_verification_rate"].as_f64().unwrap();
    // 1 of 3 claims verified = 0.333...
    assert!(
        (rate - 1.0 / 3.0).abs() < 0.01,
        "expected ~0.333, got {rate}"
    );
}

#[test]
fn stats_claim_verification_rate_null_when_no_claims() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = stats(&dir);
    assert!(v["data"]["claim_verification_rate"].is_null());
}

#[test]
fn stats_inverted_time_window_returns_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = stats_with_args(
        &dir,
        &[
            "--since",
            "2026-01-02T00:00:00Z",
            "--until",
            "2026-01-01T00:00:00Z",
        ],
    );
    assert_eq!(v["ok"], false, "inverted time window must return error");
    let code = v["data"]["code"].as_str().unwrap();
    assert!(
        code.contains("since") || code.contains("window") || code.contains("inverted"),
        "error code should reference --since/--until; got: {code}"
    );
}

#[test]
fn stats_unknown_session_returns_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = stats_with_args(&dir, &["--session", "nonexistent-session-id-xyz"]);
    assert_eq!(v["ok"], false, "unknown session must return error");
}

#[test]
fn stats_idle_skill_true_when_no_writes_in_scope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    // Run a read-only command (list) - should not affect idle_skill
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = stats(&dir);
    assert_eq!(
        v["data"]["idle_skill"], true,
        "idle_skill must be true when only read-only commands ran"
    );
}

#[test]
fn stats_caught_contradiction_count_increments_for_doubted_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    // Create claim X and conclude Y with X as a dependency (X is evidence for Y)
    let x_id = conclude_claim(&dir, "claim X that will be doubted");
    // conclude Y with --depends-on x_id to establish X as evidence for Y
    dont()
        .args([
            "conclude",
            "claim Y that depends on X",
            "--depends-on",
            &x_id,
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    // Now trust (= doubt in dont semantics) claim X; this is the contradiction event
    dont()
        .args([
            "trust",
            &x_id,
            "--reason",
            "turns out this was wrong, evidence from experiment 2026-06-15",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = stats(&dir);
    // caught_contradiction_count should be >= 1
    let count = v["data"]["caught_contradiction_count"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        count >= 1,
        "caught_contradiction_count should be at least 1; got {count}"
    );
}

#[test]
fn stats_caught_contradiction_count_zero_when_doubted_claim_not_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let x_id = conclude_claim(
        &dir,
        "isolated claim that is doubted but not used as evidence",
    );
    // trust = doubt in dont semantics; X has no depends_on back-references
    dont()
        .args([
            "trust",
            &x_id,
            "--reason",
            "was wrong, see experiment 2026-06-15",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = stats(&dir);
    assert_eq!(
        v["data"]["caught_contradiction_count"], 0,
        "caught_contradiction_count must be 0 when doubted claim was never used as evidence"
    );
}

#[test]
fn stats_default_scope_since_is_today_not_epoch() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = stats(&dir);
    let since = v["data"]["scope"]["since"].as_str().unwrap();
    // Should start with today's date (2026-...) not epoch (1970-...)
    assert!(
        !since.starts_with("1970-"),
        "default scope.since must be today's midnight, not epoch; got: {since}"
    );
    // Should end with T00:00:00Z (midnight)
    assert!(
        since.ends_with("T00:00:00Z"),
        "default scope.since must be midnight UTC; got: {since}"
    );
}

/// Ignored claims must be excluded from total claim counts and verification rate.
#[test]
fn stats_excludes_ignored_claims_from_counts() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim to be ignored");
    // Ignore the claim
    dont()
        .args(["ignore", &id, "--reason", "stray claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = stats(&dir);
    let data = &v["data"];
    // The claim_verification_rate should be null (no non-ignored claims)
    assert!(
        data["claim_verification_rate"].is_null(),
        "claim_verification_rate must be null when all claims are ignored; got: {:?}",
        data["claim_verification_rate"]
    );
}
