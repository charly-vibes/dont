mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn eval_export(dir: &TempDir) -> Value {
    let out = dont()
        .args(["export", "--eval", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<Value>(&out).unwrap()
}

fn eval_export_with_args(dir: &TempDir, extra: &[&str]) -> Value {
    let out = dont()
        .args(["export", "--eval", "--json"])
        .args(extra)
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();
    serde_json::from_slice::<Value>(&out.stdout).unwrap()
}

#[test]
fn eval_export_returns_eval_export_envelope_kind() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = eval_export(&dir);
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "eval_export");
}

#[test]
fn eval_export_empty_store_has_required_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = eval_export(&dir);
    let data = &v["data"];
    assert!(
        data["exported_at"].is_string(),
        "exported_at must be an RFC 3339 string"
    );
    assert!(data["scope"].is_object(), "scope must be an object");
    assert!(
        data["scope"]["since"].is_string(),
        "scope.since must be present"
    );
    assert!(
        data["scope"]["until"].is_string(),
        "scope.until must be present"
    );
    assert!(
        data["claims_by_status"].is_object(),
        "claims_by_status must be object"
    );
    assert!(
        data["events_by_kind"].is_object(),
        "events_by_kind must be object"
    );
    assert!(
        data["trust_events"].is_array(),
        "trust_events must be array"
    );
    assert!(
        data["dedup_refusals"].is_array(),
        "dedup_refusals must be array"
    );
}

#[test]
fn eval_export_empty_store_has_empty_arrays_and_maps() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = eval_export(&dir);
    let data = &v["data"];
    assert!(
        data["trust_events"].as_array().unwrap().is_empty(),
        "trust_events must be empty"
    );
    assert!(
        data["dedup_refusals"].as_array().unwrap().is_empty(),
        "dedup_refusals must be empty"
    );
    assert!(
        data["claims_by_status"].as_object().unwrap().is_empty(),
        "claims_by_status must be empty map"
    );
    assert!(
        data["events_by_kind"].as_object().unwrap().is_empty(),
        "events_by_kind must be empty map"
    );
}

#[test]
fn eval_export_claims_by_status_reflects_store() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id1 = conclude_claim(&dir, "claim one");
    let _id2 = conclude_claim(&dir, "claim two");
    // flag id1 to verify it (flag = "dont flag as concern" = verify in dont semantics)
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
    let v = eval_export(&dir);
    let by_status = &v["data"]["claims_by_status"];
    assert_eq!(by_status["unverified"], 1, "one unverified claim expected");
    assert_eq!(by_status["verified"], 1, "one verified claim expected");
}

#[test]
fn eval_export_events_by_kind_reflects_store() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "first claim");
    conclude_claim(&dir, "second claim");
    let v = eval_export(&dir);
    let by_kind = &v["data"]["events_by_kind"];
    assert_eq!(by_kind["concluded"], 2, "two conclude events expected");
}

#[test]
fn eval_export_trust_events_populated_after_flag() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the timeout is configurable");
    // flag = verify in dont semantics (doubt=false in trust_events)
    dont()
        .args([
            "flag",
            &id,
            "--evidence",
            "https://example.com/production-logs",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = eval_export(&dir);
    let trust_events = v["data"]["trust_events"].as_array().unwrap();
    assert_eq!(trust_events.len(), 1, "one trust_event expected");
    let te = &trust_events[0];
    assert!(te["event_id"].is_string(), "event_id must be string");
    assert!(
        te["target_claim_id"].is_string(),
        "target_claim_id must be string"
    );
    assert_eq!(te["target_claim_id"], id);
    assert_eq!(
        te["doubt"], false,
        "flag (verify) event should have doubt=false"
    );
    assert!(
        te["reason_excerpt"].is_string(),
        "reason_excerpt must be string"
    );
    assert!(
        te["timestamp"].is_string(),
        "timestamp must be RFC 3339 string"
    );
}

#[test]
fn eval_export_trust_event_reason_excerpt_truncated_at_120() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "some conclusion");
    let long_reason = "x".repeat(200);
    // trust = doubt in dont semantics; generates Trusted event with note=reason
    dont()
        .args(["trust", &id, "--reason", &long_reason, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = eval_export(&dir);
    let te = &v["data"]["trust_events"][0];
    let excerpt = te["reason_excerpt"].as_str().unwrap();
    assert!(
        excerpt.chars().count() <= 120,
        "reason_excerpt must be at most 120 code points; got {}",
        excerpt.chars().count()
    );
}

#[test]
fn eval_export_trust_event_has_doubt_true() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim that will be doubted");
    // trust = "dont trust" = doubt in dont semantics → doubt=true in trust_events
    dont()
        .args([
            "trust",
            &id,
            "--reason",
            "contradicted by new evidence from 2026-06-15",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let v = eval_export(&dir);
    let trust_events = v["data"]["trust_events"].as_array().unwrap();
    let doubt_events: Vec<_> = trust_events
        .iter()
        .filter(|te| te["doubt"] == true)
        .collect();
    assert!(
        !doubt_events.is_empty(),
        "trust (doubt) event must appear in trust_events with doubt=true"
    );
}

#[test]
fn eval_export_unknown_session_returns_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = eval_export_with_args(&dir, &["--session", "no-such-session-xyz"]);
    assert_eq!(v["ok"], false, "unknown session must return error");
}

#[test]
fn eval_export_inverted_time_window_returns_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = eval_export_with_args(
        &dir,
        &[
            "--since",
            "2026-01-02T00:00:00Z",
            "--until",
            "2026-01-01T00:00:00Z",
        ],
    );
    assert_eq!(v["ok"], false, "inverted time window must return error");
}

#[test]
fn eval_export_scope_session_not_present_when_no_session_flag() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let v = eval_export(&dir);
    let scope = &v["data"]["scope"];
    assert!(
        scope.get("session_id").is_none_or(|v| v.is_null()),
        "session_id must be absent or null when no --session flag given"
    );
}
