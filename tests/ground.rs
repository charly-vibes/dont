use assert_cmd::Command;
use serde_json::Value;
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
    let claims = v["data"].as_array().unwrap();
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

    let kinds: Vec<&str> = events
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert!(
        kinds.contains(&"concluded"),
        "events should contain 'concluded', got: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"dismissed"),
        "events should contain 'dismissed', got: {:?}",
        kinds
    );
    let concluded_pos = kinds.iter().position(|&k| k == "concluded").unwrap();
    let dismissed_pos = kinds.iter().position(|&k| k == "dismissed").unwrap();
    assert!(
        concluded_pos < dismissed_pos,
        "concluded must precede dismissed in history for {}",
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
    init_dir(&dir);

    // Create a real file at the project root so the locator is valid
    let project_root = dir.path();
    std::fs::write(project_root.join("README.md"), "# test").unwrap();

    let out = dont()
        .args([
            "ground",
            "documented in readme",
            "--file",
            "README.md",
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
}

#[test]
fn ground_with_empty_statement_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["ground", "", "--evidence", "https://example.com/proof", "--json"])
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
        .args(["ground", "-", "--evidence", "https://example.com/proof", "--json"])
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
