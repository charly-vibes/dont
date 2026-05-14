/// Spec-alignment tests for dont-payload-types.
///
/// Ticket: dont-uql5
/// Each test documents a behavioral claim from the spec and asserts the
/// implementation matches.  Tests in this file were written red-first.
use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_dir(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
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

fn define_term(dir: &TempDir, curie: &str, doc: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", doc, "--json"])
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

// --- Mismatch 1: ClaimView must include `confidence` and `provenance` ---
//
// Spec (ClaimView payload):
//   "confidence (float | null; null when no LLM-authored value was provided)"
//   "provenance"
// Impl: build_claim_view() emits neither field.

#[test]
fn claim_view_includes_confidence_field() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim view must carry confidence");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    // confidence must be present (null when no LLM-authored value)
    assert!(
        data.as_object().unwrap().contains_key("confidence"),
        "ClaimView must include 'confidence' field (spec: dont-payload-types ClaimView)"
    );
    // When no confidence was supplied at conclude time the value must be null
    assert!(data["confidence"].is_null(), "confidence must be null when not provided");
}

#[test]
fn claim_view_includes_provenance_field() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim view must carry provenance");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    assert!(
        data.as_object().unwrap().contains_key("provenance"),
        "ClaimView must include 'provenance' field (spec: dont-payload-types ClaimView)"
    );
}

// --- Mismatch 2: ClaimsList data must be {as_of, count, claims[]} ---
//
// Spec (ClaimsList payload):
//   "as_of (RFC 3339 timestamp), count (integer), and claims[] (array of ClaimView)"
// Impl: Envelope::success("claims", views, ...) where views is a bare Vec<Value>.

#[test]
fn claims_list_data_has_as_of_count_and_claims_array() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "first claim");
    conclude_claim(&dir, "second claim");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["envelope_kind"], "claims");
    let data = &v["data"];
    assert!(
        data.is_object(),
        "ClaimsList data must be an object {{as_of, count, claims[]}} not a bare array \
         (spec: dont-payload-types ClaimsList)"
    );
    assert!(
        data["as_of"].is_string(),
        "ClaimsList data must include 'as_of' RFC 3339 timestamp"
    );
    assert!(
        data["count"].is_number(),
        "ClaimsList data must include integer 'count'"
    );
    assert!(
        data["claims"].is_array(),
        "ClaimsList data must include 'claims' array"
    );
    let count = data["count"].as_u64().unwrap();
    let claims_len = data["claims"].as_array().unwrap().len() as u64;
    assert_eq!(
        count, claims_len,
        "ClaimsList count must equal the length of the claims array"
    );
}

// --- Mismatch 3: PrimeView assessment_counts keys must use hyphens ---
//
// Spec (PrimeView payload):
//   "assessment_counts (map containing exactly the derived-assessment keys
//    `stale`, `compromised-support`, `dangling-dependency`, and `unresolved-term`)"
// Impl: uses `compromised_support`, `dangling_dependency`, `unresolved_term` (underscores).

#[test]
fn prime_assessment_counts_uses_hyphenated_keys() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let ac = &v["data"]["assessment_counts"];
    assert!(
        ac.is_object(),
        "assessment_counts must be an object"
    );
    let obj = ac.as_object().unwrap();
    for key in &["stale", "compromised-support", "dangling-dependency", "unresolved-term"] {
        assert!(
            obj.contains_key(*key),
            "assessment_counts must contain key '{}' (spec: dont-payload-types PrimeView)",
            key
        );
    }
    // Underscore variants must NOT appear
    for bad_key in &["compromised_support", "dangling_dependency", "unresolved_term"] {
        assert!(
            !obj.contains_key(*bad_key),
            "assessment_counts must not use underscore key '{}'; use hyphens instead",
            bad_key
        );
    }
}

// --- Mismatch 4: TermView must NOT include `updated_at` ---
//
// Spec (TermView payload):
//   "TermView intentionally omits `updated_at` — term status transitions are
//    tracked through the event history (see `dont why`)."
// Impl: build_term_view() includes `updated_at`.

#[test]
fn term_view_omits_updated_at() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001", "a well-known process");

    let out = dont()
        .args(["show", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    assert!(
        !data.as_object().unwrap().contains_key("updated_at"),
        "TermView must NOT include 'updated_at' \
         (spec: dont-payload-types TermView — term transitions tracked via event history)"
    );
}
