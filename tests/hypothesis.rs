mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn dismiss_claim(dir: &TempDir, id: &str, evidence: &str) {
    dont()
        .args(["flag", id, "--evidence", evidence, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn hypothesis_add_appends_to_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Hypotheses support lockable claims");

    let output = dont()
        .args([
            "hypothesis",
            "add",
            &id,
            "--text",
            "First alternative explanation",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    let hypotheses = v["data"]["hypotheses"].as_array().unwrap();
    assert_eq!(hypotheses.len(), 1);
    assert_eq!(hypotheses[0]["idx"], 0);
    assert_eq!(hypotheses[0]["text"], "First alternative explanation");
    assert_eq!(
        hypotheses[0]["assessment"]["supporting"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn hypothesis_add_multiple_increments_idx() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with multiple competing hypotheses");

    for (i, text) in ["H1", "H2", "H3"].iter().enumerate() {
        let output = dont()
            .args(["hypothesis", "add", &id, "--text", text, "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let v: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(v["ok"], true);
        let hypotheses = v["data"]["hypotheses"].as_array().unwrap();
        assert_eq!(hypotheses.len(), i + 1);
        assert_eq!(hypotheses[i]["idx"], i);
        assert_eq!(hypotheses[i]["text"], *text);
    }
}

#[test]
fn hypothesis_add_on_unknown_claim_returns_not_found() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args([
            "hypothesis",
            "add",
            "claim:NOTEXIST",
            "--text",
            "H1",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "claim-not-found");
}

#[test]
fn hypothesis_assess_adds_supporting_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim needing assessed hypotheses for locking");

    dont()
        .args(["hypothesis", "add", &id, "--text", "Hypothesis A", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args([
            "hypothesis",
            "assess",
            &id,
            "0",
            "--supporting",
            "https://evidence-a.example/source",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    let h = &v["data"]["hypotheses"][0];
    assert_eq!(h["idx"], 0);
    let supporting = h["assessment"]["supporting"].as_array().unwrap();
    assert!(
        supporting
            .iter()
            .any(|s| s == "https://evidence-a.example/source")
    );
}

#[test]
fn hypothesis_assess_adds_refuting_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with refuted hypothesis");

    dont()
        .args(["hypothesis", "add", &id, "--text", "Hypothesis B", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args([
            "hypothesis",
            "assess",
            &id,
            "0",
            "--refuting",
            "https://counter-evidence.example/source",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    let h = &v["data"]["hypotheses"][0];
    let refuting = h["assessment"]["refuting"].as_array().unwrap();
    assert!(
        refuting
            .iter()
            .any(|s| s == "https://counter-evidence.example/source")
    );
}

#[test]
fn hypothesis_assess_out_of_range_returns_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with one hypothesis");

    dont()
        .args([
            "hypothesis",
            "add",
            &id,
            "--text",
            "Only hypothesis",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args([
            "hypothesis",
            "assess",
            &id,
            "5",
            "--supporting",
            "https://evidence.example/source",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
}

#[test]
fn hypothesis_add_with_empty_text_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim whose hypothesis text must be non-empty");

    let output = dont()
        .args(["hypothesis", "add", &id, "--text", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "empty-text",
        "hypothesis add with empty text must return empty-text, got: {:?}",
        v["data"]["code"]
    );
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn hypothesis_assess_without_supporting_or_refuting_returns_no_assessment() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(
        &dir,
        "Claim whose hypothesis requires at least one assessment item",
    );

    dont()
        .args([
            "hypothesis",
            "add",
            &id,
            "--text",
            "An alternative explanation",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["hypothesis", "assess", &id, "0", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "no-assessment",
        "hypothesis assess with no --supporting or --refuting must return no-assessment, got: {:?}",
        v["data"]["code"]
    );
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn lock_reachable_via_cli_hypothesis_commands() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim reachable with only CLI commands");

    dismiss_claim(&dir, &id, "https://source-one.example/evidence");
    dismiss_claim(&dir, &id, "https://source-two.example/evidence");

    for text in ["Hypothesis 1", "Hypothesis 2", "Hypothesis 3"] {
        dont()
            .args(["hypothesis", "add", &id, "--text", text, "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success();
    }
    for idx in ["0", "1", "2"] {
        dont()
            .args([
                "hypothesis",
                "assess",
                &id,
                idx,
                "--supporting",
                "https://evidence.example/support",
                "--json",
            ])
            .env("DONT_DIR", dir.path())
            .assert()
            .success();
    }

    let output = dont()
        .args(["forget", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "locked");
}
