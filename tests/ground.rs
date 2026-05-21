mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// --- ground ---

#[test]
fn ground_returns_verified_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "Chacana parses expressions into ASTs",
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["status"], "verified");
    assert!(!v["data"]["id"].as_str().unwrap().is_empty());
}

#[test]
fn ground_without_evidence_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["ground", "claim with no evidence", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
}

#[test]
fn ground_without_evidence_leaves_no_partial_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args(["ground", "the orphan claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure();

    // Listing claims should show nothing
    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let claims = v["data"]["claims"].as_array().unwrap();
    assert!(
        claims.is_empty(),
        "failed ground must not leave a partial claim, found: {:?}",
        claims
    );
}

#[test]
fn ground_history_reflects_concluded_then_dismissed_events() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "historically grounded claim",
            "--evidence",
            "https://example.com/evidence",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let id = v["data"]["id"].as_str().unwrap();
    let events = v["data"]["events"].as_array().unwrap();

    let kinds: Vec<&str> = events.iter().map(|e| e["kind"].as_str().unwrap()).collect();
    assert!(
        kinds.contains(&"concluded"),
        "events should contain 'concluded', got: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"flagged"),
        "events should contain 'flagged', got: {:?}",
        kinds
    );
    let concluded_pos = kinds.iter().position(|&k| k == "concluded").unwrap();
    let flagged_pos = kinds.iter().position(|&k| k == "flagged").unwrap();
    assert!(
        concluded_pos < flagged_pos,
        "concluded must precede flagged in history for {}",
        id
    );
}

#[test]
fn ground_with_multiple_evidence_items() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "multi-source grounded claim",
            "--evidence",
            "https://source-a.example/ref",
            "--evidence",
            "https://source-b.example/ref",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
    let evidence = v["data"]["evidence"].as_array().unwrap();
    assert_eq!(evidence.len(), 2);
}

#[test]
fn ground_with_file_locator_returns_verified() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success();

    std::fs::write(root.join("README.md"), "# test").unwrap();

    let out = dont()
        .args([
            "ground",
            "documented in readme",
            "--file",
            "README.md",
            "--json",
        ])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["status"], "verified");
}

#[test]
fn ground_refuses_unreadable_file_locator_without_partial_claim() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success();

    let out = dont()
        .args([
            "ground",
            "missing file should not verify",
            "--file",
            "MISSING.md",
            "--lines",
            "1",
            "--json",
        ])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "unreadable-evidence");

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", root.join(".dont"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listed: Value = serde_json::from_slice(&out).unwrap();
    assert!(listed["data"]["claims"].as_array().unwrap().is_empty());
}

#[test]
fn ground_with_empty_statement_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "",
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "empty-statement");
}

#[test]
fn ground_with_only_empty_evidence_uris_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["ground", "valid statement", "--evidence", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
}

#[test]
fn ground_rejects_stdin_bulk_mode() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "-",
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "stdin-not-supported");
}

#[test]
fn ground_with_path_traversal_file_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "sneaky claim",
            "--file",
            "../../../etc/passwd",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert!(
        ["path-escapes-root", "path-not-relative", "no-evidence"]
            .contains(&v["data"]["code"].as_str().unwrap()),
        "expected path-related error, got: {:?}",
        v["data"]["code"]
    );
}

#[test]
fn ground_respects_author_override_flag() {
    // Spec: ground SHALL accept the standard invocation-level author override.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "--author",
            "llm:test-model",
            "ground",
            "authored grounded claim",
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(
        v["meta"]["author"], "llm:test-model",
        "ground must propagate --author override into envelope meta"
    );
}

#[test]
fn ground_duplicate_statement_follows_conclude_policy() {
    // Spec: a duplicate-equivalent claim follows the same duplicate-claim policy
    // that would apply to the underlying `conclude` operation.
    // Since `conclude` does not refuse duplicate statements, `ground` must not
    // add an extra refusal — both invocations should succeed and produce distinct
    // verified claims.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out1 = dont()
        .args([
            "ground",
            "duplicated grounded statement",
            "--evidence",
            "https://example.com/ref1",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let out2 = dont()
        .args([
            "ground",
            "duplicated grounded statement",
            "--evidence",
            "https://example.com/ref2",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v1: Value = serde_json::from_slice(&out1).unwrap();
    let v2: Value = serde_json::from_slice(&out2).unwrap();

    assert_eq!(v1["ok"], true);
    assert_eq!(v2["ok"], true);

    let id1 = v1["data"]["id"].as_str().unwrap();
    let id2 = v2["data"]["id"].as_str().unwrap();
    assert_ne!(
        id1, id2,
        "each ground invocation must produce a distinct claim entity"
    );
    assert_eq!(v1["data"]["status"], "verified");
    assert_eq!(v2["data"]["status"], "verified");
}

#[test]
fn ground_rejects_statement_with_path_traversal_sequence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "../evil",
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
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
fn ground_rejects_statement_with_shell_metacharacter() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    for statement in &["foo;bar", "foo|bar", "foo`bar`", "foo$bar", "foo\\bar"] {
        let out = dont()
            .args([
                "ground",
                statement,
                "--evidence",
                "https://example.com/proof",
                "--json",
            ])
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

#[test]
fn ground_accepts_prose_statement_with_slash() {
    // A slash in prose (e.g. "TCP/IP") is allowed; only `..` traversal is banned.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "TCP/IP is a protocol suite",
            "--evidence",
            "https://example.com/proof",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["status"], "verified");
}

// --- Malformed evidence URI validation ---

/// `dont ground` must reject a bare string with no URI scheme at input time,
/// not silently store it.
#[test]
fn ground_malformed_evidence_uri_no_scheme_is_rejected() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args([
            "ground",
            "claim grounded with garbage evidence",
            "--evidence",
            "not-a-valid-locator",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], false,
        "malformed evidence URI must be rejected: {v}"
    );
    assert_eq!(
        v["data"]["code"], "malformed-evidence-uri",
        "error code must be malformed-evidence-uri, got: {:?}",
        v["data"]["code"]
    );
    let msg = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("not-a-valid-locator"),
        "error message must include the offending locator, got: {msg}"
    );
}

/// A failed `ground` due to malformed URI must leave no partial claim behind.
#[test]
fn ground_malformed_evidence_uri_leaves_no_partial_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    dont()
        .args([
            "ground",
            "partial claim from bad evidence",
            "--evidence",
            "garbage-no-scheme",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1);

    let out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let claims = v["data"]["claims"].as_array().unwrap();
    assert!(
        claims.is_empty(),
        "failed ground must not leave a partial claim, found: {:?}",
        claims
    );
}
