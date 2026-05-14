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

// --- [project] [output] [storage] config blocks ---

#[test]
fn config_project_output_storage_blocks_parse_without_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Overwrite config with all three blocks explicitly set
    let config_path = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"[project]
name = "test-project"
mode = "permissive"

[output]
default_format = "json"

[storage]
busy_retry_attempts = 3
busy_retry_base_ms = 50
"#,
    )
    .unwrap();

    // Any command should succeed without error — config is valid
    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn config_unknown_keys_emit_warning_not_failure() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Add an unknown top-level key — should not cause a parse failure
    append_config(
        &dir,
        "[project]\nname = \"dont-project\"\nunknown_future_key = \"value\"",
    );

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "unknown config key should not cause failure");
}

#[test]
fn prime_reflects_mode_from_project_config() {
    let dir = TempDir::new().unwrap();
    // Init in strict mode
    dont()
        .args(["init", "--strict", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["mode"], "strict");
}

// --- dont-4k5.4: config field validation at parse/load time ---

#[test]
fn invalid_project_mode_is_rejected_with_named_field_error() {
    // project.mode must be "permissive" or "strict"; any other value should
    // produce an error naming the field and the invalid value — not silently
    // treat the project as non-strict.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let config_path = dir.path().join("config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let updated = config.replace("mode = \"permissive\"", "mode = \"banana\"");
    fs::write(&config_path, updated).unwrap();

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "invalid mode should fail");
    let code = v["data"]["code"].as_str().unwrap_or("");
    assert!(
        code.contains("config") || code.contains("invalid"),
        "error code should identify this as a config error, got: {code}"
    );
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("mode") && message.contains("banana"),
        "error message should name the field and invalid value, got: {message}"
    );
}

#[test]
fn invalid_output_format_is_rejected_with_named_field_error() {
    // output.default_format must be "json" or "human"; any other value should
    // produce an error naming the field and the invalid value.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Overwrite (not append) so we don't produce a duplicate-key TOML error.
    let config_path = dir.path().join("config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let updated = config.replace("default_format = \"json\"", "default_format = \"yaml\"");
    fs::write(&config_path, updated).unwrap();

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "invalid default_format should fail");
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("default_format") && message.contains("yaml"),
        "error message should name the field and invalid value, got: {message}"
    );
}

#[test]
fn verify_evidence_zero_timeout_is_rejected_with_named_field_error() {
    // verify_evidence.default_timeout_s = 0 is not a valid timeout;
    // it should be rejected at load time with an error naming the field.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    append_config(&dir, "[verify_evidence]\ndefault_timeout_s = 0");

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "zero timeout_s should fail");
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("default_timeout_s") || message.contains("timeout"),
        "error message should name the field, got: {message}"
    );
}

#[test]
fn verify_evidence_zero_concurrency_is_rejected_with_named_field_error() {
    // verify_evidence.concurrency = 0 is not valid; must be >= 1 if provided.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    append_config(&dir, "[verify_evidence]\nconcurrency = 0");

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "zero concurrency should fail");
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("concurrency"),
        "error message should name the field, got: {message}"
    );
}

#[test]
fn mode_change_via_config_is_recorded_in_event_log() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Change config from permissive to strict
    let config_path = dir.path().join("config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let updated = config.replace("mode = \"permissive\"", "mode = \"strict\"");
    fs::write(&config_path, updated).unwrap();

    // Any command should detect the mode change and record it
    dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    // Check events.jsonl for mode_changed event
    let events_path = dir.path().join("events.jsonl");
    let events = fs::read_to_string(&events_path).unwrap();
    assert!(
        events.contains("mode_changed") || events.contains("mode.changed"),
        "events.jsonl should contain a mode change event after config update, got:\n{events}"
    );
}
