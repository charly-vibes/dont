use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_dir(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
}

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn dismiss(dir: &TempDir, id: &str, evidence: &str) -> Vec<u8> {
    dont()
        .args(["dismiss", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

fn define_term(dir: &TempDir, curie: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", "a valid definition", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

fn dismiss_file(dir: &TempDir, id: &str, extra_args: &[&str]) -> Vec<u8> {
    let mut args = vec!["dismiss", id, "--json"];
    args.extend_from_slice(extra_args);
    dont()
        .args(&args)
        .env("DONT_DIR", dir.path().join(".dont"))
        .output()
        .unwrap()
        .stdout
}

// --- Valid transitions ---

#[test]
fn dismiss_unverified_claim_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the earth is round");
    let out = dismiss(&dir, &id, "https://nasa.gov/earth-shape");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn dismiss_doubted_claim_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "photosynthesis converts CO2 to O2");
    dont()
        .args(["trust", &id, "--reason", "Need to verify the chemistry", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success();
    let out = dismiss(&dir, &id, "https://acs.org/photosynthesis");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn dismiss_unverified_term_produces_verified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P001");
    let out = dismiss(&dir, &id, "https://example.test/term-evidence");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn dismiss_carries_tx_in_meta() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "tx is tracked for mutations");
    let out = dismiss(&dir, &id, "https://example.test/tx-proof");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["meta"]["tx"].is_number());
}

#[test]
fn dismiss_evidence_appears_in_claim_view() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence is tracked");
    let ev = "https://example.test/evidence-1";
    let out = dismiss(&dir, &id, ev);
    let v: Value = serde_json::from_slice(&out).unwrap();
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(
        evidence.iter().any(|e| e.as_str() == Some(ev)),
        "evidence URI missing from claim view"
    );
}

// --- Already-verified evidence append (Phase 8) ---

#[test]
fn dismiss_already_verified_appends_evidence_without_status_change() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence accumulates over time");
    dismiss(&dir, &id, "https://example.test/ev-1");
    // Second dismiss on verified claim
    let out = dismiss(&dir, &id, "https://example.test/ev-2");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "verified");
    // Both evidence URIs should appear
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let uris: Vec<&str> = evidence.iter().filter_map(|e| e.as_str()).collect();
    assert!(uris.contains(&"https://example.test/ev-1"), "first evidence missing");
    assert!(uris.contains(&"https://example.test/ev-2"), "second evidence missing");
}

// --- Refusals ---

#[test]
fn dismiss_without_evidence_returns_no_evidence_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "evidence-free dismiss should fail");
    let out = dont()
        .args(["dismiss", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn dismiss_refuses_claims_with_unverified_term_dependencies() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001");

    let out = dont()
        .args([
            "conclude",
            "Uses WB:P001",
            "--depends-on",
            "WB:P001",
            "--json",
        ])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap().to_string();

    let output = dont()
        .args(["dismiss", &id, "--evidence", "https://example.test/proof", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-met");
    assert_eq!(v["data"]["rule_name"], "stale-cascade");

    let shown = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shown_v: Value = serde_json::from_slice(&shown).unwrap();
    assert_eq!(shown_v["data"]["status"], "unverified");
}

#[test]
fn dismiss_claim_not_found_returns_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["dismiss", "claim:01JNONEXISTENT", "--evidence", "https://example.test", "--json"])
        .env("DONT_DIR", dir.path().join(".dont"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
}

// --- Repository evidence locators ---

#[test]
fn dismiss_file_locator_stored_as_structured_entry() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::write(dir.path().join("README.md"), "API docs\n").unwrap();
    let id = conclude_claim(&dir, "README documents the API");
    let out = dismiss_file(&dir, &id, &["--file", "README.md"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "dismiss --file should succeed: {v}");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).expect("should have a structured locator entry");
    assert_eq!(locator["kind"], "repo-file");
    assert_eq!(locator["path"], "README.md");
}

#[test]
fn dismiss_file_locator_with_line_span() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let spec = (1..=18)
        .map(|n| format!("line {n}\n"))
        .collect::<String>();
    std::fs::write(dir.path().join("spec.md"), spec).unwrap();
    let id = conclude_claim(&dir, "spec defines the contract on lines 10-18");
    let out = dismiss_file(&dir, &id, &["--file", "spec.md", "--lines", "10-18"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "dismiss --file --lines should succeed: {v}");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).unwrap();
    assert_eq!(locator["kind"], "repo-file");
    assert_eq!(locator["path"], "spec.md");
    assert_eq!(locator["line_start"], 10);
    assert_eq!(locator["line_end"], 18);
}

#[test]
fn dismiss_file_locator_with_anchor() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::create_dir(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("docs/api.md"), "# authentication\n").unwrap();
    let id = conclude_claim(&dir, "section heading anchors the claim");
    let out = dismiss_file(&dir, &id, &["--file", "docs/api.md", "--anchor", "authentication"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let evidence = v["data"]["evidence"].as_array().unwrap();
    let locator = evidence.iter().find(|e| e.is_object()).unwrap();
    assert_eq!(locator["anchor"], "authentication");
}

#[test]
fn dismiss_file_locator_escape_via_dotdot_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim needing evidence");
    let out = dismiss_file(&dir, &id, &["--file", "../../etc/passwd"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "path escape should be refused: {v}");
    assert_eq!(v["data"]["code"], "path-escapes-root");
}

#[test]
fn dismiss_file_locator_absolute_path_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "claim needing evidence");
    let out = dismiss_file(&dir, &id, &["--file", "/etc/passwd"]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "absolute path should be refused: {v}");
    assert_eq!(v["data"]["code"], "path-not-relative");
}

#[test]
fn dismiss_file_and_uri_evidence_combined() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    std::fs::write(dir.path().join("README.md"), "proof\n").unwrap();
    let id = conclude_claim(&dir, "claim grounded in both repo and URI evidence");
    let out = dismiss_file(
        &dir,
        &id,
        &["--file", "README.md", "--evidence", "https://example.test/proof"],
    );
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert!(evidence.iter().any(|e| e.as_str() == Some("https://example.test/proof")), "URI evidence missing");
    assert!(evidence.iter().any(|e| e.is_object()), "locator entry missing");
}

#[test]
fn dismiss_no_evidence_without_file_flag_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "must provide evidence");
    let out = dismiss_file(&dir, &id, &[]);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
}
