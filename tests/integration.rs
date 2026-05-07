use assert_cmd::Command;
use serde_json::Value;
use std::time::Instant;
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

fn assert_envelope_conformance(v: &Value, expect_ok: bool) {
    assert!(v.get("ok").is_some(), "missing ok");
    assert!(v.get("envelope_version").is_some(), "missing envelope_version");
    assert_eq!(v["envelope_version"], "0.2", "envelope_version must be 0.2");
    assert!(v.get("cli_version").is_some(), "missing cli_version");
    assert!(v.get("envelope_kind").is_some(), "missing envelope_kind");
    assert!(v.get("data").is_some(), "missing data");
    assert!(v["warnings"].is_array(), "warnings must be array");
    assert!(v["meta"].is_object(), "meta must be object");
    assert!(v["meta"]["duration_ms"].is_number(), "meta.duration_ms must be number");
    assert!(v["meta"]["request_id"].is_null(), "meta.request_id must be null for tracer");

    if expect_ok {
        assert_eq!(v["ok"], true);
        assert!(v["hints"].is_array(), "success envelope must have hints array");
    } else {
        assert_eq!(v["ok"], false);
        assert!(
            v.as_object().is_none_or(|m| !m.contains_key("hints")),
            "error envelope must not carry hints"
        );
        let remediation = v["data"]["remediation"].as_array().unwrap();
        assert!(!remediation.is_empty(), "remediation must be non-empty on error");
        for entry in remediation {
            assert!(entry["command"].is_string(), "remediation entry must have command string");
            assert!(entry["description"].is_string(), "remediation entry must have description string");
        }
    }
}

// --- Multi-step workflows ---

#[test]
fn workflow_conclude_trust_show() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "bacteria cause cholera");
    dont()
        .args(["trust", &id, "--reason", "Germ theory needs independent replication", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "doubted");
    assert_eq!(v["data"]["id"], id.as_str());
}

#[test]
fn workflow_conclude_dismiss_show() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the speed of light is constant");
    dont()
        .args(["flag", &id, "--evidence", "https://nist.gov/codata", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(evidence.iter().any(|e| e.as_str() == Some("https://nist.gov/codata")));
}

#[test]
fn workflow_full_transition_cycle() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    // unverified → verified → doubted → verified
    let id = conclude_claim(&dir, "plate tectonics drives continental drift");
    dont()
        .args(["flag", &id, "--evidence", "https://usgs.gov/tectonics", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    dont()
        .args(["trust", &id, "--reason", "Recent dataset contradicts the velocity estimates", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let out = dont()
        .args(["flag", &id, "--evidence", "https://usgs.gov/corrected", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(evidence.len() >= 2, "both evidence URIs should accumulate");
}

// --- Envelope conformance ---

#[test]
fn envelope_conformance_success_commands() {
    let dir = TempDir::new().unwrap();

    // init
    let out = dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_envelope_conformance(&serde_json::from_slice::<Value>(&out).unwrap(), true);

    // conclude
    let out = dont()
        .args(["conclude", "test claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_envelope_conformance(&v, true);
    assert!(v["meta"]["tx"].is_number(), "conclude must have tx");
    let id = v["data"]["id"].as_str().unwrap().to_string();

    // trust
    let out = dont()
        .args(["trust", &id, "--reason", "Requires peer review verification", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_envelope_conformance(&v, true);
    assert!(v["meta"]["tx"].is_number(), "trust must have tx");

    // dismiss
    let out = dont()
        .args(["flag", &id, "--evidence", "https://example.test/proof", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_envelope_conformance(&v, true);
    assert!(v["meta"]["tx"].is_number(), "dismiss must have tx");

    // show (read-only — tx must be null)
    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_envelope_conformance(&v, true);
    assert!(v["meta"]["tx"].is_null(), "show must have null tx (read-only)");

    // list (read-only)
    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_envelope_conformance(&v, true);
    assert!(v["meta"]["tx"].is_null(), "list must have null tx (read-only)");
}

#[test]
fn envelope_conformance_error_commands() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim for refusal tests");

    // trust without reason → reason-required, exit 1
    let out = dont()
        .args(["trust", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_envelope_conformance(&v, false);
    assert!(v["meta"]["tx"].is_null(), "refusal must have null tx");

    // dismiss without evidence → no-evidence, exit 1
    let out = dont()
        .args(["flag", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    assert_envelope_conformance(&serde_json::from_slice::<Value>(&out).unwrap(), false);

    // show nonexistent → claim-not-found, exit 1
    let out = dont()
        .args(["show", "claim:01JNONE", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    assert_envelope_conformance(&serde_json::from_slice::<Value>(&out).unwrap(), false);

    // config-missing → exit 3
    let out = dont()
        .args(["conclude", "orphan", "--json"])
        .env("DONT_DIR", dir.path().join("nonexistent"))
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();
    assert_envelope_conformance(&serde_json::from_slice::<Value>(&out).unwrap(), false);
}

// --- Refusal loop ---

#[test]
fn workflow_refusal_loop_dismiss_no_evidence_then_with_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "vaccines cause immunity");

    // dismiss without evidence → error, exit 1, structured remediation
    let out = dont()
        .args(["flag", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty(), "error must have remediation");
    for entry in remediation {
        assert!(entry["command"].is_string(), "remediation entry must have command");
        assert!(entry["description"].is_string(), "remediation entry must have description");
    }

    // claim is still unverified after the refusal
    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "unverified", "status unchanged after refusal");

    // dismiss with evidence → verified
    dont()
        .args(["flag", &id, "--evidence", "https://cdc.gov/vaccines", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
}

// --- Persistence across processes ---

#[test]
fn persistence_across_process_boundaries() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "memory outlives the process");

    // Trust in one process
    dont()
        .args(["trust", &id, "--reason", "Needs verification in a fresh process too", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Verify in a completely separate process invocation
    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "doubted",
        "status must survive process exit and re-open");
}

// --- Performance ---

#[test]
fn list_performance_under_250ms_on_100_claims() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    for i in 0..100 {
        conclude_claim(&dir, &format!("claim number {i} for performance test"));
    }

    let start = Instant::now();
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 250,
        "list on 100 claims took {}ms (limit 250ms)",
        elapsed.as_millis()
    );
}
