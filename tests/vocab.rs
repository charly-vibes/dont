mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

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

#[test]
fn vocab_returns_terms_envelope_kind() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let term_id = define_term(&dir, "WB:P001", "a process by which X becomes Y");

    let out = dont()
        .args(["vocab", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term_list");
    let terms = v["data"].as_array().unwrap();
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0]["id"], term_id);
    assert_eq!(terms[0]["entity_kind"], "term");
    assert_eq!(terms[0]["curie"], "WB:P001");
}

#[test]
fn vocab_is_equivalent_to_list_kind_term() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001", "a process by which X becomes Y");
    define_term(&dir, "WB:P002", "a secondary concept");

    let vocab_out = dont()
        .args(["vocab", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_out = dont()
        .args(["list", "--kind", "terms", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let vocab_v: Value = serde_json::from_slice(&vocab_out).unwrap();
    let list_v: Value = serde_json::from_slice(&list_out).unwrap();

    assert_eq!(vocab_v["envelope_kind"], list_v["envelope_kind"]);
    let vocab_terms = vocab_v["data"].as_array().unwrap();
    let list_terms = list_v["data"].as_array().unwrap();
    assert_eq!(vocab_terms.len(), list_terms.len());
}

#[test]
fn vocab_empty_project_returns_empty_array() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["vocab", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term_list");
    assert_eq!(v["data"].as_array().unwrap().len(), 0);
}
