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
