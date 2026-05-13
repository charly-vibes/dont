use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_project(root: &TempDir) -> assert_cmd::assert::Assert {
    let dont_dir = root.path().join(".dont");
    dont()
        .args(["init", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
}

fn doctor_json(root: &TempDir) -> assert_cmd::assert::Assert {
    let dont_dir = root.path().join(".dont");
    dont()
        .args(["doctor", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
}

fn doctor_fix(root: &TempDir) -> assert_cmd::assert::Assert {
    let dont_dir = root.path().join(".dont");
    dont()
        .args(["doctor", "--fix", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
}

fn root_block(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn init_creates_canonical_and_root_managed_docs() {
    let root = TempDir::new().unwrap();
    init_project(&root).success();

    let canonical = fs::read_to_string(root.path().join(".dont/AGENTS.md")).unwrap();
    let agents = root_block(&root.path().join("AGENTS.md"));
    let claude = root_block(&root.path().join("CLAUDE.md"));

    assert!(
        canonical.contains("managed by `dont init`") || canonical.contains("managed by `dont`")
    );

    for doc in [&agents, &claude] {
        assert!(
            doc.contains("<!-- DONT:START -->"),
            "missing start marker: {doc}"
        );
        assert!(
            doc.contains("<!-- DONT:END -->"),
            "missing end marker: {doc}"
        );
        assert!(
            doc.contains("dont prime --json"),
            "missing prime instruction: {doc}"
        );
        assert!(
            doc.contains(".dont/AGENTS.md"),
            "missing canonical pointer: {doc}"
        );
        assert!(
            doc.contains("DO NOT EDIT") || doc.contains("do not edit"),
            "missing warning: {doc}"
        );
        assert!(
            !doc.contains("underlying model is still conclude"),
            "root managed block should stay minimal and not inline the canonical guidance: {doc}"
        );
    }
}

#[test]
fn init_preserves_existing_root_content_outside_managed_block() {
    let root = TempDir::new().unwrap();
    fs::write(
        root.path().join("AGENTS.md"),
        "# Team notes\n\nKeep this intro.\n",
    )
    .unwrap();

    init_project(&root).success();

    let agents = root_block(&root.path().join("AGENTS.md"));
    assert!(
        agents.contains("# Team notes"),
        "preexisting content should remain: {agents}"
    );
    assert!(
        agents.contains("Keep this intro."),
        "preexisting content should remain: {agents}"
    );
    assert!(
        agents.contains("<!-- DONT:START -->"),
        "managed block should be injected: {agents}"
    );
}

#[test]
fn doctor_reports_pass_for_fresh_init() {
    let root = TempDir::new().unwrap();
    init_project(&root).success();

    let output = doctor_json(&root).success().get_output().stdout.clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    let checks = v["data"]["checks"].as_array().unwrap();
    let managed_docs = checks
        .iter()
        .find(|check| check["name"] == "managed_docs")
        .expect("managed_docs check missing");

    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "doctor");
    assert_eq!(managed_docs["status"], "pass");
}

#[test]
fn doctor_reports_warn_for_edited_root_block_and_ignores_whitespace_only_drift() {
    let root = TempDir::new().unwrap();
    init_project(&root).success();

    let agents_path = root.path().join("AGENTS.md");
    let original = fs::read_to_string(&agents_path).unwrap();
    let whitespace_only = original.replace("\n", "  \r\n");
    fs::write(&agents_path, whitespace_only).unwrap();

    let pass_output = doctor_json(&root).success().get_output().stdout.clone();
    let pass_json: Value = serde_json::from_slice(&pass_output).unwrap();
    let pass_checks = pass_json["data"]["checks"].as_array().unwrap();
    let pass_managed = pass_checks
        .iter()
        .find(|check| check["name"] == "managed_docs")
        .expect("managed_docs check missing");
    assert_eq!(pass_managed["status"], "pass");

    let edited = original.replace("dont prime --json", "dont prime --json\n\nMANUAL EDIT");
    fs::write(&agents_path, edited).unwrap();

    let warn_output = doctor_json(&root).success().get_output().stdout.clone();
    let warn_json: Value = serde_json::from_slice(&warn_output).unwrap();
    let warn_checks = warn_json["data"]["checks"].as_array().unwrap();
    let warn_managed = warn_checks
        .iter()
        .find(|check| check["name"] == "managed_docs")
        .expect("managed_docs check missing");
    assert_eq!(warn_managed["status"], "warn");
    assert!(
        warn_managed["detail"]
            .as_str()
            .unwrap()
            .contains("dont doctor --fix")
    );
}

#[test]
fn doctor_warns_when_seed_snapshot_file_is_missing() {
    let root = TempDir::new().unwrap();
    init_project(&root).success();

    fs::remove_file(root.path().join(".dont/seed/dont-seed.yaml")).unwrap();

    let output = doctor_json(&root).success().get_output().stdout.clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    let checks = v["data"]["checks"].as_array().unwrap();
    let seed_snapshot = checks
        .iter()
        .find(|check| check["name"] == "seed_snapshot")
        .expect("seed_snapshot check missing");

    assert_eq!(seed_snapshot["status"], "warn");
    assert!(
        seed_snapshot["detail"]
            .as_str()
            .unwrap()
            .contains("dont init")
    );
}

#[test]
fn doctor_fix_restores_stale_docs_and_is_idempotent() {
    let root = TempDir::new().unwrap();
    init_project(&root).success();

    let agents_path = root.path().join("AGENTS.md");
    let canonical_path = root.path().join(".dont/AGENTS.md");
    fs::write(&agents_path, "# custom\nno markers here\n").unwrap();
    fs::write(&canonical_path, "stale canonical\n").unwrap();

    doctor_fix(&root).success();

    let repaired_agents = fs::read_to_string(&agents_path).unwrap();
    let repaired_canonical = fs::read_to_string(&canonical_path).unwrap();
    assert!(repaired_agents.contains("<!-- DONT:START -->"));
    assert!(repaired_agents.contains("dont prime --json"));
    assert!(
        repaired_agents.contains("# custom"),
        "existing content should remain outside inserted block"
    );
    assert!(
        repaired_canonical.contains("managed by `dont init`")
            || repaired_canonical.contains("managed by `dont`")
    );

    let before_second_fix_agents = repaired_agents.clone();
    let before_second_fix_canonical = repaired_canonical.clone();
    doctor_fix(&root).success();
    assert_eq!(
        fs::read_to_string(&agents_path).unwrap(),
        before_second_fix_agents
    );
    assert_eq!(
        fs::read_to_string(&canonical_path).unwrap(),
        before_second_fix_canonical
    );
}
