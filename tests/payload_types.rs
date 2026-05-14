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

// --- ClaimView required array fields always present ---
//
// Spec (ClaimView payload):
//   "All array fields SHALL always be present (possibly empty), never omitted."
//   Fields: derived_assessments[], atoms[], hypotheses[], evidence[], depends_on[]

#[test]
fn claim_view_required_array_fields_always_present() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "array fields must always be present");

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
    let obj = data.as_object().unwrap();

    for field in &["derived_assessments", "atoms", "hypotheses", "evidence", "depends_on"] {
        assert!(
            obj.contains_key(*field),
            "ClaimView must include '{}' array field (spec: dont-payload-types ClaimView — all array fields always present)",
            field
        );
        assert!(
            data[field].is_array(),
            "ClaimView field '{}' must be an array",
            field
        );
    }
}

// --- ClaimView entity_kind must be "claim" ---
//
// Spec (ClaimView payload):
//   "entity_kind: \"claim\""

#[test]
fn claim_view_entity_kind_is_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "entity kind must be claim");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["entity_kind"].as_str().unwrap(),
        "claim",
        "ClaimView entity_kind must be 'claim' (spec: dont-payload-types ClaimView)"
    );
}

// --- ClaimView must include updated_at ---
//
// Spec (ClaimView payload):
//   "created_at, updated_at" — both present (contrast with TermView which omits updated_at)

#[test]
fn claim_view_includes_updated_at() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "updated_at must be present on claims");

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
        data.as_object().unwrap().contains_key("updated_at"),
        "ClaimView must include 'updated_at' (spec: dont-payload-types ClaimView)"
    );
    assert!(
        data["updated_at"].is_string(),
        "ClaimView updated_at must be a string (RFC 3339 timestamp)"
    );
}

// --- TermView required fields ---
//
// Spec (TermView payload):
//   id, entity_kind: "term", curie, definition, kind_of[], related_to[], status,
//   derived_assessments[], confidence (float|null), provenance, created_at, applicable_rules

#[test]
fn term_view_required_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:T001", "a test term for field coverage");

    let out = dont()
        .args(["show", "WB:T001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    let obj = data.as_object().unwrap();

    assert_eq!(
        data["entity_kind"].as_str().unwrap(),
        "term",
        "TermView entity_kind must be 'term'"
    );
    for field in &["id", "curie", "definition", "status", "created_at"] {
        assert!(
            obj.contains_key(*field) && data[field].is_string(),
            "TermView must include string field '{}'",
            field
        );
    }
    for field in &["kind_of", "related_to", "derived_assessments"] {
        assert!(
            obj.contains_key(*field) && data[field].is_array(),
            "TermView must include array field '{}'",
            field
        );
    }
    assert!(
        obj.contains_key("confidence"),
        "TermView must include 'confidence' field (float|null)"
    );
    assert!(
        obj.contains_key("provenance"),
        "TermView must include 'provenance' field"
    );
    assert!(
        obj.contains_key("applicable_rules") && data["applicable_rules"].is_object(),
        "TermView must include 'applicable_rules' object"
    );
}

// --- WhyView structure ---
//
// Spec (WhyView payload):
//   entity (full ClaimView or TermView), history[], applicable_rules, remediation[]

#[test]
fn why_view_structure() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "why view must have required structure");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["envelope_kind"], "why", "why command must return envelope_kind 'why'");
    let data = &v["data"];
    let obj = data.as_object().unwrap();

    assert!(
        obj.contains_key("entity") && data["entity"].is_object(),
        "WhyView must include 'entity' object (spec: dont-payload-types WhyView)"
    );
    assert!(
        obj.contains_key("history") && data["history"].is_array(),
        "WhyView must include 'history' array"
    );
    assert!(
        obj.contains_key("applicable_rules") && data["applicable_rules"].is_object(),
        "WhyView must include 'applicable_rules' object"
    );
    assert!(
        obj.contains_key("remediation") && data["remediation"].is_array(),
        "WhyView must include 'remediation' array"
    );
    // entity must be a full ClaimView with entity_kind: "claim"
    assert_eq!(
        data["entity"]["entity_kind"].as_str().unwrap(),
        "claim",
        "WhyView entity must be a full ClaimView with entity_kind 'claim'"
    );
}

// --- WhyView history EventView shape ---
//
// Spec (EventView payload):
//   entity_id, tx, event_kind, at, author, reason (nullable), evidence_uri (nullable),
//   spawn_request_id (nullable)

#[test]
fn why_view_history_event_shape() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "event view shape in history");

    let out = dont()
        .args(["why", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let history = v["data"]["history"].as_array().unwrap();
    assert!(
        !history.is_empty(),
        "WhyView history must contain at least one event (the 'concluded' event)"
    );

    let event = &history[0];
    let event_obj = event.as_object().unwrap();
    for field in &["tx", "event_kind", "at"] {
        assert!(
            event_obj.contains_key(*field),
            "EventView entry must include field '{}' (spec: dont-payload-types EventView)",
            field
        );
    }
    // Nullable fields must be present (as null or string)
    for field in &["reason", "evidence_uri", "spawn_request_id"] {
        assert!(
            event_obj.contains_key(*field),
            "EventView entry must include nullable field '{}' (spec: dont-payload-types EventView)",
            field
        );
    }
    // event_kind for the first event on a concluded claim must be "concluded"
    assert_eq!(
        event["event_kind"].as_str().unwrap(),
        "concluded",
        "First history event for a newly concluded claim must have event_kind 'concluded'"
    );
}

// --- PrimeView top-level required fields ---
//
// Spec (PrimeView payload):
//   project, mode, status_counts, assessment_counts, rules, ontologies[], blocking[],
//   pending_spawns, harness_mode (boolean), invariants[]

#[test]
fn prime_view_required_top_level_fields() {
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
    let data = &v["data"];
    let obj = data.as_object().unwrap();

    for field in &["project", "mode"] {
        assert!(
            obj.contains_key(*field) && data[field].is_string(),
            "PrimeView must include string field '{}'",
            field
        );
    }
    for field in &["ontologies", "blocking", "invariants"] {
        assert!(
            obj.contains_key(*field) && data[field].is_array(),
            "PrimeView must include array field '{}'",
            field
        );
    }
    for field in &["status_counts", "assessment_counts", "rules"] {
        assert!(
            obj.contains_key(*field) && data[field].is_object(),
            "PrimeView must include object field '{}'",
            field
        );
    }
    assert!(
        obj.contains_key("pending_spawns") && data["pending_spawns"].is_number(),
        "PrimeView must include numeric 'pending_spawns'"
    );
    assert!(
        obj.contains_key("harness_mode") && data["harness_mode"].is_boolean(),
        "PrimeView must include boolean 'harness_mode' (spec: dont-payload-types PrimeView)"
    );
}

// --- PrimeView status_counts has all 5 persisted-status keys ---
//
// Spec (PrimeView payload):
//   "status_counts (map containing exactly the persisted-status keys
//    `unverified`, `doubted`, `verified`, `locked`, and `ignored` with integer counts)"

#[test]
fn prime_status_counts_has_all_five_keys() {
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
    let sc = &v["data"]["status_counts"];
    assert!(sc.is_object(), "status_counts must be an object");

    let obj = sc.as_object().unwrap();
    for key in &["unverified", "doubted", "verified", "locked", "ignored"] {
        assert!(
            obj.contains_key(*key),
            "status_counts must contain key '{}' (spec: dont-payload-types PrimeView)",
            key
        );
        assert!(
            sc[key].is_number(),
            "status_counts['{}'] must be an integer",
            key
        );
    }
}

// --- DoctorReport structure ---
//
// Spec (DoctorReport payload):
//   cli_version, checks[] (each with name, status, detail), summary (counts of pass/warn/fail)
//   Required check names: substrate, rules_compile, seed_snapshot, pending_spawns,
//   remediation_invariant

#[test]
fn doctor_report_structure() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["doctor", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        // doctor may exit non-zero if a check warns; don't assert success
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["envelope_kind"], "doctor",
        "doctor command must return envelope_kind 'doctor'"
    );
    let data = &v["data"];

    assert!(
        data["cli_version"].is_string(),
        "DoctorReport must include string 'cli_version'"
    );
    assert!(
        data["checks"].is_array(),
        "DoctorReport must include 'checks' array"
    );
    assert!(
        data["summary"].is_object(),
        "DoctorReport must include 'summary' object"
    );

    // Each check must have name, status, detail
    for check in data["checks"].as_array().unwrap() {
        assert!(check["name"].is_string(), "Each check must have string 'name'");
        assert!(check["status"].is_string(), "Each check must have string 'status'");
        assert!(check["detail"].is_string(), "Each check must have string 'detail'");
        let status = check["status"].as_str().unwrap();
        assert!(
            ["pass", "warn", "fail"].contains(&status),
            "Check status must be pass/warn/fail, got '{}'",
            status
        );
    }

    // Required check names must be present
    let check_names: Vec<&str> = data["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    for required in &["substrate", "rules_compile", "seed_snapshot", "pending_spawns", "remediation_invariant"] {
        assert!(
            check_names.contains(required),
            "DoctorReport checks must include '{}' (spec: dont-payload-types DoctorReport)",
            required
        );
    }

    // summary must have pass, warn, fail counts
    let summary = &data["summary"];
    for key in &["pass", "warn", "fail"] {
        assert!(
            summary[key].is_number(),
            "DoctorReport summary must include integer '{}'",
            key
        );
    }
}

// --- ConcludeInput: --confidence flag stores the value in ClaimView ---
//
// Spec (ConcludeInput schema):
//   "confidence? (float 0.0-1.0)"
// Spec (ClaimView payload):
//   "confidence (float | null; null when no LLM-authored value was provided)"
// Impl: ClaimRecord has no confidence field; build_claim_view always emits null.
// This test is RED until conclude accepts --confidence and stores it.

#[test]
fn conclude_with_confidence_stores_value_in_claim_view() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "confidence must be stored", "--confidence", "0.8", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap().to_string();

    let show_out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let sv: Value = serde_json::from_slice(&show_out).unwrap();
    let confidence = &sv["data"]["confidence"];
    assert!(
        !confidence.is_null(),
        "ClaimView confidence must not be null when --confidence was provided at conclude time \
         (spec: dont-payload-types ConcludeInput + ClaimView)"
    );
    let conf_val = confidence.as_f64().expect("confidence must be a float");
    assert!(
        (conf_val - 0.8).abs() < 1e-6,
        "ClaimView confidence must be 0.8, got {}",
        conf_val
    );
}
