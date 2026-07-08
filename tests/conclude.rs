mod common;

use assert_cmd::cargo::cargo_bin;
use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

fn conclude_in(dir: &TempDir, statement: &str) -> Vec<u8> {
    dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

// --- Successful conclude ---

#[test]
fn conclude_returns_claim_envelope_with_ok_true() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "all bachelors are unmarried");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_version"], "0.2");
    assert_eq!(v["envelope_kind"], "claim");
}

#[test]
fn conclude_creates_claim_with_unverified_status() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "water is H2O");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "unverified");
}

#[test]
fn conclude_claim_has_prefixed_ulid_id() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "entropy always increases");
    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap();
    assert!(
        id.starts_with("claim:"),
        "id should have claim: prefix, got {id}"
    );
}

#[test]
fn conclude_claim_view_has_required_arrays() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "test claim");
    let v: Value = serde_json::from_slice(&out).unwrap();
    let data = &v["data"];
    assert!(
        data["derived_assessments"].is_array(),
        "derived_assessments"
    );
    assert!(data["atoms"].is_array(), "atoms");
    assert!(data["hypotheses"].is_array(), "hypotheses");
    assert!(data["evidence"].is_array(), "evidence");
    assert!(data["depends_on"].is_array(), "depends_on");
    assert!(data["applicable_rules"].is_object(), "applicable_rules");
}

#[test]
fn conclude_populates_statement_in_claim_view() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let statement = "the sky is blue during clear weather";
    let out = conclude_in(&dir, statement);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["statement"], statement);
}

#[test]
fn conclude_envelope_has_tx_set_for_mutation() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "mutations carry a tx");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(
        v["meta"]["tx"].is_number(),
        "mutation must have non-null tx"
    );
}

#[test]
fn conclude_persists_claim_across_invocations() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = conclude_in(&dir, "gravity attracts masses");
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "first conclude should succeed");
    let id = v["data"]["id"].as_str().unwrap().to_string();
    assert!(!id.is_empty(), "first conclude must return a claim id");

    // A second invocation with identical text is rejected as a duplicate.
    let out2 = dont()
        .args(["conclude", "gravity attracts masses", "--json"])
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();
    let v2: Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(v2["ok"], false, "duplicate conclude must be refused");
    assert_eq!(
        v2["data"]["code"], "duplicate-refused",
        "duplicate conclude must use code duplicate-refused"
    );

    // Remediation must include an actionable dont flag command
    let remediation = v2["data"]["remediation"].as_array().unwrap();
    let has_flag_remediation = remediation.iter().any(|r| {
        r["command"]
            .as_str()
            .map_or(false, |cmd| cmd.contains("dont flag"))
    });
    assert!(
        has_flag_remediation,
        "duplicate conclude error must include a dont flag remediation; got: {remediation:?}"
    );
    let has_id_in_remediation = remediation
        .iter()
        .any(|r| r["command"].as_str().map_or(false, |cmd| cmd.contains(&id)));
    assert!(
        has_id_in_remediation,
        "remediation must reference the existing claim ID {id}; got: {remediation:?}"
    );
}

// --- Refusals ---

#[test]
fn conclude_empty_statement_returns_validation_error_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let out = dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "empty-statement");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

// --- Conclude outside project ---

#[test]
fn conclude_outside_project_exits_3_with_config_missing() {
    let dir = TempDir::new().unwrap();
    // No init — DONT_DIR points to nonexistent dir
    let out = dont()
        .args(["conclude", "some claim", "--json"])
        .env("DONT_DIR", dir.path().join("nonexistent"))
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "config-missing");
    let remediation = v["data"]["remediation"].as_array().unwrap();
    assert!(!remediation.is_empty());
    // At least one remediation entry must have a command referencing "init".
    // Each item is an object with "command" and "description" fields.
    let mentions_init = remediation.iter().any(|item| {
        item["command"]
            .as_str()
            .map(|s| s.contains("init"))
            .unwrap_or(false)
            || item["description"]
                .as_str()
                .map(|s| s.contains("init"))
                .unwrap_or(false)
    });
    assert!(
        mentions_init,
        "remediation must reference 'init', got: {remediation:?}"
    );
}

// --- Parallel execution ---

#[test]
fn conclude_produces_unique_tx_ids_under_parallel_subprocess_load() {
    let bin = cargo_bin("dont");

    for attempt in 1..=3 {
        let dir = TempDir::new().unwrap();
        init_dir(&dir);
        let dont_dir = dir.path().to_owned();

        // Spawn 8 concurrent `dont conclude` subprocesses — the repro from dont-fl6.
        let children: Vec<_> = (0..8u32)
            .map(|i| {
                std::process::Command::new(&bin)
                    .args([
                        "conclude",
                        &format!("parallel claim {attempt}-{i}"),
                        "--json",
                    ])
                    .env("DONT_DIR", &dont_dir)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .expect("spawned")
            })
            .collect();

        let outputs: Vec<_> = children
            .into_iter()
            .map(|child| child.wait_with_output().expect("waited"))
            .collect();

        if outputs.iter().all(|out| out.status.success()) {
            let mut tx_ids: Vec<i64> = outputs
                .into_iter()
                .map(|out| {
                    let v: Value = serde_json::from_slice(&out.stdout).expect("valid json");
                    v["meta"]["tx"].as_i64().expect("meta.tx is i64")
                })
                .collect();

            tx_ids.sort_unstable();
            tx_ids.dedup();
            assert_eq!(tx_ids.len(), 8, "all tx IDs must be unique; got {tx_ids:?}");
            return;
        }

        if attempt == 3 {
            let diagnostics: Vec<_> = outputs
                .iter()
                .map(|out| {
                    (
                        out.status.code(),
                        String::from_utf8_lossy(&out.stdout).into_owned(),
                        String::from_utf8_lossy(&out.stderr).into_owned(),
                    )
                })
                .collect();
            panic!("parallel conclude remained flaky after 3 attempts: {diagnostics:?}");
        }
    }
}

#[test]
fn conclude_rejects_statement_with_path_traversal_sequence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "../evil", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"], "statement-contains-path-traversal",
        "expected statement-contains-path-traversal, got: {:?}",
        v["data"]["code"]
    );
}

#[test]
fn conclude_rejects_statement_with_shell_metacharacter() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    for statement in &["foo;bar", "foo|bar", "foo`bar`", "foo$bar", "foo\\bar"] {
        let out = dont()
            .args(["conclude", statement, "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            v["ok"], false,
            "statement {:?} should be rejected",
            statement
        );
        assert_eq!(
            v["data"]["code"], "statement-contains-metacharacter",
            "statement {:?} should produce statement-contains-metacharacter, got: {:?}",
            statement, v["data"]["code"]
        );
    }
}
