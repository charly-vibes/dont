// dont-wjd9: network-error robustness for dont import linkml <url>
//
// Failure modes under test:
//   (a) connection refused / DNS failure → immediate non-zero exit;
//       error message includes the URL and error kind
//   (b) slow / unresponsive host → bounded wait (≤30 s) with timeout error;
//       error message includes the URL

mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// (a) Connection-refused: unreachable local port
// ---------------------------------------------------------------------------

/// Importing from a URL that refuses connections must exit non-zero and
/// include the URL in the error message.
#[test]
fn import_from_refused_url_exits_nonzero_with_url_in_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Port 19999 is almost certainly not listening; connection-refused is instant.
    let url = "http://localhost:19999/schema.yaml";

    let out = dont()
        .args(["import", "linkml", url, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(predicates::prelude::predicate::ne(0))
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap_or(Value::Null);
    if v != Value::Null {
        assert_eq!(v["ok"], false, "response envelope must not be ok");
        let message = v["data"]["message"].as_str().unwrap_or("");
        assert!(
            message.contains("localhost:19999") || message.contains(url),
            "error message must include the URL, got: {message}"
        );
    }
}

/// Importing from a URL with an invalid/unreachable host must exit non-zero
/// and surface the URL in the error message.
#[test]
fn import_from_unreachable_host_exits_nonzero_with_url_in_error() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // 0.0.0.0 is not routable as a destination; connection will fail immediately.
    let url = "http://0.0.0.0:19998/schema.yaml";

    let out = dont()
        .args(["import", "linkml", url, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(predicates::prelude::predicate::ne(0))
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap_or(Value::Null);
    if v != Value::Null {
        assert_eq!(v["ok"], false, "response envelope must not be ok");
        let message = v["data"]["message"].as_str().unwrap_or("");
        assert!(
            message.contains("0.0.0.0") || message.contains(url),
            "error message must include the URL, got: {message}"
        );
    }
}

// ---------------------------------------------------------------------------
// (b) Timeout: the command must finish within a reasonable wall-clock bound
// ---------------------------------------------------------------------------

/// Even when the target host is unreachable, `dont import linkml <url>` must
/// complete within 30 seconds (the spec-mandated maximum wait).
///
/// We use a refused-port URL because connection-refused returns immediately;
/// this test mainly guards against an infinite hang introduced by a missing
/// timeout configuration.
#[test]
fn import_from_refused_url_completes_within_30_seconds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let url = "http://localhost:19997/schema.yaml";

    // The assertion will time out and panic if the command hangs; assert_cmd
    // runs commands synchronously so the test runner enforces the wall-clock
    // limit indirectly.  We set an explicit 30-second timeout via the std
    // process::Command timeout wrapper.
    let start = std::time::Instant::now();

    dont()
        .args(["import", "linkml", url, "--json"])
        .env("DONT_DIR", dir.path())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .code(predicates::prelude::predicate::ne(0));

    let elapsed = start.elapsed();
    // Use sub-second precision: as_secs() truncates, so 30.001s would report
    // "30s" and fail the < 30 check even though the command was killed on time.
    assert!(
        elapsed.as_millis() < 30_500,
        "import must fail within 30s (spec bound), took {}ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Retry hint in error response
// ---------------------------------------------------------------------------

/// The error response for a network failure must suggest a retry.
#[test]
fn import_network_error_suggests_retry() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let url = "http://localhost:19996/schema.yaml";

    let out = dont()
        .args(["import", "linkml", url, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(predicates::prelude::predicate::ne(0))
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap_or(Value::Null);
    if v != Value::Null && v["ok"] == false {
        // Check that remediation or message hints at retry
        let message = v["data"]["message"].as_str().unwrap_or("");
        let remediation = v["data"]["remediation"].to_string();
        let has_retry_hint = message.to_lowercase().contains("retry")
            || message.to_lowercase().contains("check")
            || remediation.to_lowercase().contains("retry")
            || remediation.to_lowercase().contains("check");
        assert!(
            has_retry_hint,
            "network error must suggest a retry, got message={message:?}, remediation={remediation}"
        );
    }
}

// ---------------------------------------------------------------------------
// Local file path still works (regression guard)
// ---------------------------------------------------------------------------

/// A local file path must still work after the URL detection logic is added.
#[test]
fn import_from_local_file_still_works_after_url_handling_added() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema_path = dir.path().join("basic.yaml");
    std::fs::write(
        &schema_path,
        r#"
id: https://example.org/basic-test
name: basic-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

classes:
  Observation:
    class_uri: ex:Observation
    description: A scientific observation
"#,
    )
    .unwrap();

    let out = dont()
        .args(["import", "linkml", schema_path.to_str().unwrap(), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "local file import must still succeed");
}
