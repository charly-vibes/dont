/// Tests for very long evidence excerpts and URIs.
///
/// Covers dont-03m0: "Very long evidence excerpts — no truncation bugs or panics"
///
/// Expected behaviours:
/// - Storage accepts full content without panics (no fixed-size buffers).
/// - Serialization round-trips the full excerpt without corruption.
/// - Human `show` output truncates long evidence display strings gracefully
///   with "…" so the terminal is not flooded by a 10 k-char blob.
/// - JSON `show --json` output always contains the full, untruncated excerpt.
mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

/// Helper: run `dont flag <id> --evidence <uri> --json` and assert success.
fn flag_with_evidence(dir: &TempDir, id: &str, uri: &str) -> Value {
    let out = dont()
        .args(["flag", id, "--evidence", uri, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

/// Helper: run `dont show <id> --json` and return parsed output.
fn show_json(dir: &TempDir, id: &str) -> Value {
    let out = dont()
        .args(["show", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

/// Helper: run `dont show <id>` (human output) and return stdout as string.
fn show_human(dir: &TempDir, id: &str) -> String {
    let out = dont()
        .args(["show", id])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8_lossy(&out).into_owned()
}

// ── Storage and round-trip ─────────────────────────────────────────────────

/// A very long plain-URI evidence value (10 k+ chars) must be stored and
/// retrieved without corruption.  The full string must appear verbatim in the
/// JSON output.
#[test]
fn very_long_evidence_uri_round_trips_in_json() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "long URI round-trip");

    // Build a 10 k+ character URI (starts with https:// so it looks like a URI)
    let long_uri = format!("https://example.test/evidence/{}", "x".repeat(10_000));

    let v = flag_with_evidence(&dir, &id, &long_uri);
    assert_eq!(v["ok"], true, "flag with long URI should succeed: {v}");

    // JSON show must contain the full URI, not a truncated version
    let show = show_json(&dir, &id);
    let evidence = show["data"]["evidence"].as_array().unwrap();
    let stored_uri = evidence
        .iter()
        .find_map(|e| e.as_str())
        .expect("evidence URI must be present in JSON output");

    assert_eq!(
        stored_uri.len(),
        long_uri.len(),
        "JSON output must contain the full URI without truncation"
    );
    assert_eq!(stored_uri, long_uri, "JSON evidence must round-trip exactly");
}

/// A very long `--excerpt` value (10 k+ chars) passed to `flag --file --excerpt`
/// must be stored and retrieved without corruption.
///
/// This test does not use `--file` (which requires git provenance) — it directly
/// verifies that the store accepts and round-trips large excerpt strings by
/// injecting them via the store API so we can test the storage layer in
/// isolation.  The CLI surface test is in `very_long_excerpt_accepted_by_flag`.
#[test]
fn very_long_excerpt_round_trips_via_store() {
    use dont::store::{Store, StoreEvent, StoreEventKind, StoreStatus};

    let dir = TempDir::new().unwrap();
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let store = Store::open_dont_dir(dir.path().join(".dont")).unwrap();
    let claim_result = store.append_claim("long excerpt round-trip", &[], None).unwrap();
    let claim_id = claim_result.id.clone();

    // Build a 10 k+ character excerpt (use a fixed-length string to avoid
    // floating point rounding in the repeat count).
    let long_excerpt = "A".repeat(10_001);

    let locator = serde_json::json!({
        "kind": "repo-file",
        "path": "docs/spec.md",
        "excerpt": long_excerpt
    });

    store
        .append_status_change(
            &claim_id,
            StoreStatus::Unverified,
            StoreStatus::Verified,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![locator],
            },
        )
        .unwrap();

    // Read back via store — excerpt must round-trip exactly
    let record = store.claim_by_id(&claim_id).unwrap().unwrap();
    let stored_evidence = &record.events.last().unwrap().evidence;
    let stored_excerpt = stored_evidence[0]["excerpt"]
        .as_str()
        .expect("excerpt must be present in stored evidence");

    assert_eq!(
        stored_excerpt.len(),
        long_excerpt.len(),
        "Store must preserve the full excerpt length"
    );
    assert_eq!(stored_excerpt, long_excerpt, "Store must round-trip excerpt exactly");
}

// ── No panics ──────────────────────────────────────────────────────────────

/// `show` (human output) must not panic when evidence contains a very long URI.
/// The process must exit 0 successfully.
#[test]
fn show_human_does_not_panic_on_very_long_evidence_uri() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "no panic on long URI");

    let long_uri = format!("https://example.test/{}", "y".repeat(10_000));
    flag_with_evidence(&dir, &id, &long_uri);

    // Must not panic — assert success is all we need here
    let _ = show_human(&dir, &id);
}

// ── Display truncation ─────────────────────────────────────────────────────

/// Human `show` output must truncate very long evidence URI display strings
/// to a reasonable width (≤ 200 visible characters) and append "…" so the
/// terminal is not flooded by a 10 k-char blob.
///
/// The JSON output must still contain the full URI.
#[test]
fn show_human_truncates_very_long_evidence_uri_in_display() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "display truncation for long URI");

    let long_uri = format!("https://example.test/{}", "z".repeat(10_000));
    flag_with_evidence(&dir, &id, &long_uri);

    let human = show_human(&dir, &id);

    // Each evidence line in the human output must be at most 200 chars wide
    // (the indentation is 4 chars, so the URI portion ≤ 196 chars)
    for line in human.lines() {
        let trimmed = line.trim();
        // Only check lines that look like evidence entries
        if trimmed.starts_with("https://") {
            assert!(
                line.len() <= 200,
                "human output evidence line must be ≤ 200 chars wide, got {} chars: {:?}",
                line.len(),
                &line[..line.len().min(80)]
            );
            assert!(
                trimmed.ends_with('…') || trimmed.ends_with("..."),
                "truncated evidence line must end with ellipsis, got: {:?}",
                trimmed
            );
        }
    }

    // JSON output must still contain the full URI
    let json_show = show_json(&dir, &id);
    let evidence = json_show["data"]["evidence"].as_array().unwrap();
    let stored = evidence.iter().find_map(|e| e.as_str()).unwrap();
    assert_eq!(stored.len(), long_uri.len(), "JSON must contain full URI");
}
