use assert_cmd::Command;
use serde_json::Value;
use std::fs;
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

fn append_config(dir: &TempDir, extra: &str) {
    let path = dir.path().join("config.toml");
    let mut content = fs::read_to_string(&path).unwrap();
    content.push('\n');
    content.push_str(extra);
    content.push('\n');
    fs::write(path, content).unwrap();
}

#[test]
fn import_verify_shape() {
    // --- 1. Disabled adapter causes refusal ---
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    append_config(&dir, "[import.wikidata]\nenabled = false");

    let output = dont()
        .args(["import", "wikidata", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "adapter-disabled");

    // --- 2. verify_evidence uses config default_timeout_s ---
    let dir2 = TempDir::new().unwrap();
    init_dir(&dir2);
    append_config(&dir2, "[verify_evidence]\ndefault_timeout_s = 7");

    let claim_out = dont()
        .args(["conclude", "test claim for timeout config", "--json"])
        .env("DONT_DIR", dir2.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let cv: Value = serde_json::from_slice(&claim_out).unwrap();
    let id = cv["data"]["id"].as_str().unwrap().to_string();

    dont()
        .args(["flag", &id, "--evidence", "https://example.test/ref", "--json"])
        .env("DONT_DIR", dir2.path())
        .assert()
        .success();

    let verify_out = dont()
        .args(["verify-evidence", &id, "--json"])
        .env("DONT_DIR", dir2.path())
        .env("DONT_VERIFY_EVIDENCE_MOCK", r#"{"https://example.test/ref":{"outcome":"reachable"}}"#)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let vv: Value = serde_json::from_slice(&verify_out).unwrap();
    assert_eq!(vv["ok"], true);
    assert_eq!(vv["data"]["timeout_seconds"], 7);

    // --- 3. [define.shape] check_indefinite = false skips indefinite-article check ---
    let dir3 = TempDir::new().unwrap();
    init_dir(&dir3);
    append_config(&dir3, "[define.shape]\ncheck_indefinite = false");

    // "Ricci tensor" lacks indefinite article — normally refused, but check is disabled.
    let output3 = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "Ricci tensor", "--json"])
        .env("DONT_DIR", dir3.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v3: Value = serde_json::from_slice(&output3).unwrap();
    assert_eq!(v3["ok"], true);
}

#[test]
fn hedges_rules() {
    // --- 1. [trust.hedges] custom pattern causes refusal ---
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    append_config(
        &dir,
        "[trust.hedges]\npatterns = [\"speculative at best\"]",
    );

    let claim_out = dont()
        .args(["conclude", "a claim to trust with hedge", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let cv: Value = serde_json::from_slice(&claim_out).unwrap();
    let id = cv["data"]["id"].as_str().unwrap().to_string();

    let trust_out = dont()
        .args([
            "trust",
            &id,
            "--reason",
            "speculative at best but worth it",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let tv: Value = serde_json::from_slice(&trust_out).unwrap();
    assert_eq!(tv["ok"], false);
    assert_eq!(tv["data"]["code"], "reason-not-hedge");

    // --- 2. [rules.term_nonfunctional] enabled → warning emitted on define ---
    let dir2 = TempDir::new().unwrap();
    init_dir(&dir2);
    append_config(
        &dir2,
        "[rules.term_nonfunctional]\nenabled = true\npatterns = [\"responsible for\"]",
    );

    let def_out = dont()
        .args([
            "define",
            "WB:P002",
            "--doc",
            "A component that routes requests",
            "--label",
            "a component responsible for routing",
            "--json",
        ])
        .env("DONT_DIR", dir2.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let dv: Value = serde_json::from_slice(&def_out).unwrap();
    assert_eq!(dv["ok"], true);
    let warnings = dv["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["rule_name"].as_str() == Some("term-nonfunctional-label")),
        "expected term-nonfunctional-label warning, got: {warnings:?}"
    );

    // --- 3. term_nonfunctional disabled by default → no warning ---
    let dir3 = TempDir::new().unwrap();
    init_dir(&dir3);

    let def_out3 = dont()
        .args([
            "define",
            "WB:P003",
            "--doc",
            "A component that routes requests",
            "--label",
            "a component responsible for routing",
            "--json",
        ])
        .env("DONT_DIR", dir3.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let dv3: Value = serde_json::from_slice(&def_out3).unwrap();
    assert_eq!(dv3["ok"], true);
    let warnings3 = dv3["warnings"].as_array().unwrap();
    assert!(
        !warnings3
            .iter()
            .any(|w| w["rule_name"].as_str() == Some("term-nonfunctional-label")),
        "expected no term-nonfunctional-label warning, got: {warnings3:?}"
    );
}
