mod common;

use common::{dont, init_dir};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Minimum viable config without [storage] and [harness] blocks.
/// Tests that customize those blocks provide their own full replacement.
const BASE_CONFIG: &str = r#"
[project]
name = "test-project"
mode = "permissive"

[output]
default_format = "json"
"#;

/// Default [storage] block (used by `write_config` when test doesn't override it).
const STORAGE_DEFAULTS: &str = "[storage]\nbusy_retry_attempts = 5\nbusy_retry_base_ms = 100\n";

/// Default [harness] block (used by `write_config` when test doesn't override it).
const HARNESS_DEFAULTS: &str =
    "[harness]\nmanaged_docs = [\"AGENTS.md\", \"CLAUDE.md\"]\nspawn_timeout_hours = 24\n";

/// Write a full config from scratch (no defaults appended).
fn write_full_config(dir: &TempDir, toml: &str) {
    fs::write(dir.path().join("config.toml"), toml).unwrap();
}

/// Write a config with built-in defaults plus extra blocks.
///
/// Tests that override [storage] or [harness] MUST use `write_full_config`
/// instead to avoid duplicate key errors.
fn write_config(dir: &TempDir, extra: &str) {
    let path = dir.path().join("config.toml");
    let content = format!("{BASE_CONFIG}\n{STORAGE_DEFAULTS}\n{HARNESS_DEFAULTS}\n{extra}\n");
    fs::write(path, content).unwrap();
}

/// Assert a CLI command exits 0.
fn assert_config_parses(dir: &TempDir) {
    let output = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
    let body: Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    assert_eq!(body.get("ok").and_then(|v| v.as_bool()), Some(true));
}

/// Assert a CLI command fails with a mention of `keyword` in its output.
fn assert_rejected(dir: &TempDir, block: &str, keyword: &str) {
    let output = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure();
    let stderr = std::str::from_utf8(&output.get_output().stderr).unwrap();
    let stdout = std::str::from_utf8(&output.get_output().stdout).unwrap();
    let combined = format!("{stderr}{stdout}");
    assert!(
        combined.contains(keyword),
        "expected output to contain '{keyword}' for {block} block, got stderr: {stderr:?}, stdout: {stdout:?}",
    );
}

// ============================================================
// [storage] block — busy retry tuning
// ============================================================

#[test]
fn config_storage_block_custom_values_parse() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_full_config(
        &dir,
        &format!(
            "{BASE_CONFIG}\n\
             [storage]\n\
             busy_retry_attempts = 10\n\
             busy_retry_base_ms = 500\n\
             {HARNESS_DEFAULTS}"
        ),
    );
    assert_config_parses(&dir);
}

// ============================================================
// [harness] block — managed docs and spawn timeout
// ============================================================

#[test]
fn config_harness_block_custom_values_parse() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_full_config(
        &dir,
        &format!(
            "{BASE_CONFIG}\n\
             {STORAGE_DEFAULTS}\n\
             [harness]\n\
             managed_docs = [\"README.md\"]\n\
             spawn_timeout_hours = 48\n"
        ),
    );
    assert_config_parses(&dir);
}

// ============================================================
// [import] adapter blocks
// ============================================================

#[test]
fn config_import_adapter_blocks_parse() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "\
[import.wikidata]
enabled = true
endpoint = \"https://query.wikidata.org/sparql\"

[import.linkml]
enabled = false

[import.ols]
base_url = \"https://www.ebi.ac.uk/ols\"
",
    );
    assert_config_parses(&dir);
}

// ============================================================
// [verify_evidence] block
// ============================================================

#[test]
fn config_verify_evidence_block_parses() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[verify_evidence]\n\
         concurrency = 4\n\
         rate_limit_per_host = 2.0\n\
         burst_per_host = 3\n\
         retry_limit = 5\n\
         default_timeout_s = 30\n",
    );
    assert_config_parses(&dir);
}

// ============================================================
// [define.shape] block
// ============================================================

#[test]
fn config_define_shape_block_parses() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[define.shape]\ncheck_indefinite = false\ncheck_punctuated = false\n",
    );
    assert_config_parses(&dir);
}

#[test]
fn config_define_shape_compound_markers_parse() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[define.shape]\ncompound_markers = [\"a pair\", \"a triple\"]\n",
    );
    assert_config_parses(&dir);
}

// ============================================================
// [trust.hedges] block
// ============================================================

#[test]
fn config_trust_hedges_parse() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[trust.hedges]\npatterns = [\"i'm not sure\", \"probably\"]\n",
    );
    assert_config_parses(&dir);
}

// ============================================================
// [rules] severity lists
// ============================================================

#[test]
fn config_rules_severity_lists_parse() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[rules]\nwarn = [\"correlated-error\"]\nstrict = [\"lockable\"]\n",
    );
    assert_config_parses(&dir);
}

// ============================================================
// [rules.term_nonfunctional] block
// ============================================================

#[test]
fn config_rules_term_nonfunctional_parses() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[rules.term_nonfunctional]\n\
         enabled = true\n\
         patterns = [\"enables\", \"provides a\"]\n",
    );
    assert_config_parses(&dir);
}

// ============================================================
// [rules.rule_claim_structure] block
// ============================================================

#[test]
fn config_rules_rule_claim_structure_parses() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(
        &dir,
        "[rules.rule_claim_structure]\n\
         enabled = true\n\
         tag_term_id = \"dont:RuleClaim\"\n",
    );
    assert_config_parses(&dir);
}

// ============================================================
// Unknown-field rejection for every config block
// ============================================================

#[test]
fn config_unknown_field_in_storage_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_full_config(
        &dir,
        &format!(
            "{BASE_CONFIG}\n\
             [storage]\n\
             unknown_field = 42\n\
             {HARNESS_DEFAULTS}"
        ),
    );
    assert_rejected(&dir, "storage", "unknown_field");
}

#[test]
fn config_unknown_field_in_harness_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_full_config(
        &dir,
        &format!(
            "{BASE_CONFIG}\n\
             {STORAGE_DEFAULTS}\n\
             [harness]\n\
             bad_key = true\n"
        ),
    );
    assert_rejected(&dir, "harness", "bad_key");
}

#[test]
fn config_unknown_field_in_verify_evidence_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(&dir, "[verify_evidence]\nbad_field = 1\n");
    assert_rejected(&dir, "verify_evidence", "bad_field");
}

#[test]
fn config_unknown_field_in_define_shape_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(&dir, "[define.shape]\nbad_toggle = true\n");
    assert_rejected(&dir, "define.shape", "bad_toggle");
}

#[test]
fn config_unknown_field_in_trust_hedges_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(&dir, "[trust.hedges]\nunknown_prop = \"x\"\n");
    assert_rejected(&dir, "trust.hedges", "unknown_prop");
}

#[test]
fn config_unknown_field_in_rules_severity_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(&dir, "[rules]\nbad_severity_list = [\"x\"]\n");
    assert_rejected(&dir, "rules", "bad_severity_list");
}

#[test]
fn config_unknown_field_in_rule_claim_structure_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(&dir, "[rules.rule_claim_structure]\nbad_flag = true\n");
    assert_rejected(&dir, "rules.rule_claim_structure", "bad_flag");
}

#[test]
fn config_unknown_field_in_term_nonfunctional_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_config(&dir, "[rules.term_nonfunctional]\nbad_prop = \"x\"\n");
    assert_rejected(&dir, "rules.term_nonfunctional", "bad_prop");
}
