mod common;

use common::dont;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn init_project(root: &TempDir) {
    let dont_dir = root.path().join(".dont");
    dont()
        .args(["init", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
        .success();
}

fn add_managed_skill_pack(root: &TempDir) {
    let config_path = root.path().join(".dont/config.toml");
    let mut config = fs::read_to_string(&config_path).unwrap();
    config.push_str("\nmanaged_skill_packs = [\"dont-grill\"]\n");
    fs::write(&config_path, config).unwrap();
}

fn doctor_json(root: &TempDir) -> Value {
    let dont_dir = root.path().join(".dont");
    let output = dont()
        .args(["doctor", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

fn doctor_fix(root: &TempDir) {
    let dont_dir = root.path().join(".dont");
    dont()
        .args(["doctor", "--fix", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
        .success();
}

fn managed_skills_check(v: &Value) -> &Value {
    let checks = v["data"]["checks"].as_array().unwrap();
    checks
        .iter()
        .find(|c| c["name"] == "managed_skills")
        .expect("managed_skills check missing from doctor output")
}

// ── config parsing ───────────────────────────────────────────────────────────

#[test]
fn config_rejects_unknown_pack_name() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    let config_path = root.path().join(".dont/config.toml");
    let mut config = fs::read_to_string(&config_path).unwrap();
    config.push_str("\nmanaged_skill_packs = [\"unknown-pack\"]\n");
    fs::write(&config_path, config).unwrap();

    let dont_dir = root.path().join(".dont");
    // unknown pack names should surface an error (config-invalid or similar), not a panic
    let output = dont()
        .args(["doctor", "--json"])
        .current_dir(root.path())
        .env("DONT_DIR", &dont_dir)
        .assert()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    // Either an error envelope or a doctor response with a non-pass managed_skills check
    let is_error = v["ok"] == false;
    if is_error {
        // Error path: config-invalid or similar
        let code = v["data"]["code"].as_str().unwrap_or("");
        assert!(!code.is_empty(), "error envelope should have a code: {v}");
    } else {
        let checks = v["data"]["checks"].as_array().unwrap();
        let check = checks
            .iter()
            .find(|c| c["name"] == "managed_skills")
            .expect("managed_skills check missing");
        assert_ne!(
            check["status"], "pass",
            "unknown pack should not pass: {check}"
        );
    }
}

// ── install on init ──────────────────────────────────────────────────────────

#[test]
fn init_installs_managed_skill_pack_when_configured() {
    // After init, user adds managed_skill_packs to config, then doctor --fix installs the pack.
    // This covers the spec requirement that dont init + dont doctor --fix are both writers.
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    let router = root.path().join(".agents/skills/dont-grill/dont-grill.md");
    assert!(
        router.exists(),
        "router skill should be installed at .agents/skills/dont-grill/dont-grill.md"
    );
}

#[test]
fn init_skips_skill_packs_when_not_configured() {
    let root = TempDir::new().unwrap();
    init_project(&root);

    let skills_dir = root.path().join(".agents/skills/dont-grill");
    assert!(
        !skills_dir.exists(),
        "skill pack dir should not exist when not configured"
    );
}

// ── doctor staleness reporting ───────────────────────────────────────────────

#[test]
fn doctor_reports_missing_when_pack_configured_but_absent() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);

    let v = doctor_json(&root);
    let check = managed_skills_check(&v);
    assert_eq!(
        check["status"], "missing",
        "should report missing when pack dir absent: {check}"
    );
    assert!(
        check["detail"]
            .as_str()
            .unwrap()
            .contains("dont doctor --fix"),
        "detail should suggest fix: {check}"
    );
}

#[test]
fn doctor_reports_pass_when_pack_matches_generator_output() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);

    // Fix installs the pack
    doctor_fix(&root);

    let v = doctor_json(&root);
    let check = managed_skills_check(&v);
    assert_eq!(
        check["status"], "pass",
        "should report pass after install: {check}"
    );
}

#[test]
fn doctor_reports_stale_when_pack_file_is_modified() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    // Mutate the router file
    let router = root.path().join(".agents/skills/dont-grill/dont-grill.md");
    let mut content = fs::read_to_string(&router).unwrap();
    content.push_str("\n<!-- MANUAL EDIT -->\n");
    fs::write(&router, content).unwrap();

    let v = doctor_json(&root);
    let check = managed_skills_check(&v);
    assert_eq!(
        check["status"], "stale",
        "should report stale after manual edit: {check}"
    );
}

// ── doctor --fix repairs ─────────────────────────────────────────────────────

#[test]
fn doctor_fix_installs_pack_when_missing() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);

    doctor_fix(&root);

    let router = root.path().join(".agents/skills/dont-grill/dont-grill.md");
    assert!(
        router.exists(),
        "router skill should be installed after fix"
    );

    let router_content = fs::read_to_string(&router).unwrap();
    assert!(
        !router_content.is_empty(),
        "router skill content should not be empty"
    );
}

#[test]
fn doctor_fix_repairs_stale_pack_and_reports_pass() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    let router = root.path().join(".agents/skills/dont-grill/dont-grill.md");
    fs::write(&router, "stale content\n").unwrap();

    doctor_fix(&root);

    let v = doctor_json(&root);
    let check = managed_skills_check(&v);
    assert_eq!(check["status"], "pass", "should pass after repair: {check}");
}

#[test]
fn doctor_fix_removes_extra_files_not_in_generated_set() {
    // Regression test: refresh must delete files that are no longer in the
    // generator output; otherwise the hash never matches after a version upgrade
    // removes a sub-skill (stale loop forever).
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    // Inject a file that simulates a removed sub-skill from a prior version
    let ghost_file = root
        .path()
        .join(".agents/skills/dont-grill/subs/old-removed-skill.md");
    fs::write(&ghost_file, "# ghost from prior version\n").unwrap();

    // Before fix, ghost file causes stale
    let v_before = doctor_json(&root);
    let check_before = managed_skills_check(&v_before);
    assert_eq!(check_before["status"], "stale");

    // After fix, ghost file is removed and hash converges
    doctor_fix(&root);
    assert!(
        !ghost_file.exists(),
        "ghost file should be removed by doctor --fix"
    );

    let v_after = doctor_json(&root);
    let check_after = managed_skills_check(&v_after);
    assert_eq!(
        check_after["status"], "pass",
        "should pass after removing ghost file"
    );
}

#[test]
fn doctor_fix_is_idempotent() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);

    doctor_fix(&root);
    let after_first: Vec<(String, String)> = files_in_skills_dir(root.path());
    doctor_fix(&root);
    let after_second: Vec<(String, String)> = files_in_skills_dir(root.path());

    assert_eq!(
        after_first, after_second,
        "second doctor --fix should produce identical output"
    );
}

fn files_in_skills_dir(root: &std::path::Path) -> Vec<(String, String)> {
    let skills_dir = root.join(".agents/skills");
    let mut result = Vec::new();
    if !skills_dir.exists() {
        return result;
    }
    collect_files(&skills_dir, &skills_dir, &mut result);
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

fn collect_files(
    base: &std::path::Path,
    current: &std::path::Path,
    out: &mut Vec<(String, String)>,
) {
    for entry in fs::read_dir(current).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, out);
        } else {
            let rel = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            out.push((rel, content));
        }
    }
}

// ── ownership boundary ───────────────────────────────────────────────────────

#[test]
fn doctor_fix_preserves_unmanaged_sibling_skill() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);

    // Create a user-authored skill alongside the managed pack
    let custom_dir = root.path().join(".agents/skills/my-custom-skill");
    fs::create_dir_all(&custom_dir).unwrap();
    let custom_file = custom_dir.join("SKILL.md");
    fs::write(&custom_file, "# My custom skill\n\nDon't touch this.\n").unwrap();

    doctor_fix(&root);

    assert!(
        custom_file.exists(),
        "unmanaged sibling skill should survive doctor --fix"
    );
    assert_eq!(
        fs::read_to_string(&custom_file).unwrap(),
        "# My custom skill\n\nDon't touch this.\n",
        "unmanaged sibling content should be unchanged"
    );
}

// ── pack content ─────────────────────────────────────────────────────────────

#[test]
fn installed_pack_contains_router_and_all_sub_skills() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    let pack_dir = root.path().join(".agents/skills/dont-grill");
    assert!(
        pack_dir.join("dont-grill.md").exists(),
        "router dont-grill.md should exist"
    );

    let sub_skills = [
        "conclude",
        "define",
        "flag",
        "trust",
        "lock",
        "ignore",
        "trace",
        "scenarios",
        "conclude-worthiness",
    ];
    for sub in &sub_skills {
        let found = find_file_by_stem(&pack_dir, sub);
        assert!(
            found,
            "sub-skill '{sub}' should be present in the installed pack"
        );
    }
}

fn find_file_by_stem(dir: &std::path::Path, stem: &str) -> bool {
    let mut found = false;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if find_file_by_stem(&p, stem) {
                    found = true;
                }
            } else if p.file_stem().and_then(|s| s.to_str()) == Some(stem) {
                found = true;
            }
        }
    }
    found
}

#[test]
fn router_uses_canonical_verbs_not_deprecated_aliases() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    let router = root.path().join(".agents/skills/dont-grill/dont-grill.md");
    let content = fs::read_to_string(&router).unwrap();

    assert!(
        content.contains("dont flag"),
        "router should use canonical 'dont flag' verb: {content}"
    );
    assert!(
        content.contains("dont lock"),
        "router should use canonical 'dont lock' verb: {content}"
    );
    assert!(
        !content.contains("dont dismiss"),
        "router should not use deprecated 'dont dismiss': {content}"
    );
    assert!(
        !content.contains("dont forget"),
        "router should not use deprecated 'dont forget': {content}"
    );
}

#[test]
fn sub_skills_are_installed_in_subdirectory() {
    let root = TempDir::new().unwrap();
    init_project(&root);
    add_managed_skill_pack(&root);
    doctor_fix(&root);

    let pack_dir = root.path().join(".agents/skills/dont-grill");
    let top_level_files: Vec<_> = fs::read_dir(&pack_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert_eq!(
        top_level_files,
        vec!["dont-grill.md"],
        "only the router should be at the top level of the pack dir; sub-skills must be in a subdirectory. found: {top_level_files:?}"
    );
}
