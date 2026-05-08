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

fn conclude_claim(dir: &TempDir, statement: &str) -> String {
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["data"]["id"].as_str().unwrap().to_string()
}

#[test]
fn atom_define_appends_unverified_atom_to_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with atomized verification plan");

    let output = dont()
        .args(["atom", "define", &id, "--text", "Sub-statement A", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    let atoms = v["data"]["atoms"].as_array().unwrap();
    assert_eq!(atoms.len(), 1);
    assert_eq!(atoms[0]["idx"], 0);
    assert_eq!(atoms[0]["text"], "Sub-statement A");
    assert_eq!(atoms[0]["status"], "unverified");
}

#[test]
fn atom_define_multiple_increments_idx() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with multiple atoms");

    for (i, text) in ["A", "B"].iter().enumerate() {
        let output = dont()
            .args(["atom", "define", &id, "--text", text, "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let v: Value = serde_json::from_slice(&output).unwrap();
        let atoms = v["data"]["atoms"].as_array().unwrap();
        assert_eq!(atoms.len(), i + 1);
        assert_eq!(atoms[i]["idx"], i);
        assert_eq!(atoms[i]["text"], *text);
    }
}

#[test]
fn atom_define_unknown_claim_returns_not_found() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args([
            "atom",
            "define",
            "claim:NOTEXIST",
            "--text",
            "Sub-statement A",
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
fn atom_dismiss_marks_atom_verified_with_evidence() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with dismissible atom");
    dont()
        .args(["atom", "define", &id, "--text", "Checkable part", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args([
            "atom",
            "dismiss",
            &id,
            "0",
            "--evidence",
            "https://evidence.example/atom",
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
    let atom = &v["data"]["atoms"][0];
    assert_eq!(atom["status"], "verified");
    let evidence = atom["evidence"].as_array().unwrap();
    assert!(
        evidence
            .iter()
            .any(|e| e == "https://evidence.example/atom")
    );
}

#[test]
fn atom_dismiss_out_of_range_returns_atom_not_found() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with one atom");
    dont()
        .args(["atom", "define", &id, "--text", "Only atom", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args([
            "atom",
            "dismiss",
            &id,
            "5",
            "--evidence",
            "https://evidence.example/atom",
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
    assert_eq!(v["data"]["code"], "atom-not-found");
}

#[test]
fn atom_dismiss_without_evidence_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "Claim with atom needing evidence");
    dont()
        .args(["atom", "define", &id, "--text", "Checkable part", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let output = dont()
        .args(["atom", "dismiss", &id, "0", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "no-evidence");
}
