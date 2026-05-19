// dont-ybgt: import surface — all adapters except linkml return not-implemented
//
// Contract being tested:
//   Each stub adapter (obo, ols, wikidata, openalex, bioregistry, jsonld, ttl)
//   must exit non-zero, emit an ok=false envelope with code="not-implemented",
//   mention the adapter name in the message, and provide a remediation that
//   suggests the linkml adapter.

mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn assert_stub_adapter_contract(adapter: &str) {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["import", adapter, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out)
        .unwrap_or_else(|_| panic!("{adapter}: response must be valid JSON"));

    assert_eq!(
        v["ok"], false,
        "{adapter}: ok must be false for unimplemented adapter"
    );

    let code = v["data"]["code"].as_str().unwrap_or("");
    assert_eq!(
        code, "not-implemented",
        "{adapter}: error code must be 'not-implemented', got {code:?}"
    );

    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains(adapter),
        "{adapter}: error message must mention the adapter name, got: {message:?}"
    );

    // Remediation must suggest linkml (the one working adapter)
    let remediation = v["data"]["remediation"].to_string();
    assert!(
        remediation.to_lowercase().contains("linkml"),
        "{adapter}: remediation must suggest linkml, got: {remediation}"
    );
}

#[test]
fn stub_adapter_obo_returns_not_implemented() {
    assert_stub_adapter_contract("obo");
}

#[test]
fn stub_adapter_ols_returns_not_implemented() {
    assert_stub_adapter_contract("ols");
}

#[test]
fn stub_adapter_wikidata_returns_not_implemented() {
    assert_stub_adapter_contract("wikidata");
}

#[test]
fn stub_adapter_openalex_returns_not_implemented() {
    assert_stub_adapter_contract("openalex");
}

#[test]
fn stub_adapter_bioregistry_returns_not_implemented() {
    assert_stub_adapter_contract("bioregistry");
}

#[test]
fn stub_adapter_jsonld_returns_not_implemented() {
    assert_stub_adapter_contract("jsonld");
}

#[test]
fn stub_adapter_ttl_returns_not_implemented() {
    assert_stub_adapter_contract("ttl");
}
