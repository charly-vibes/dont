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

fn write_custom_rule(dir: &TempDir, name: &str, script: &str) {
    let rules_dir = dir.path().join("rules");
    std::fs::write(rules_dir.join(format!("{name}.dl")), script).unwrap();
}

// --- rules list ---

#[test]
fn rules_list_returns_rule_list_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "rule_list");
    assert!(v["data"].is_array());
}

#[test]
fn rules_list_contains_all_seven_shipped_rules() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let rules = v["data"].as_array().unwrap();
    let names: Vec<&str> = rules.iter().filter_map(|r| r["name"].as_str()).collect();

    for expected in [
        "ungrounded",
        "unresolved-terms",
        "stale-cascade",
        "lockable",
        "correlated-error",
        "dangling-definition",
        "term-nonfunctional-label",
    ] {
        assert!(names.contains(&expected), "missing shipped rule: {expected}");
    }
}

#[test]
fn rules_list_shipped_rules_have_correct_source_and_severity_fields() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let rules = v["data"].as_array().unwrap();

    for rule in rules {
        assert!(rule["name"].is_string(), "name must be string: {rule}");
        assert!(
            matches!(rule["severity"].as_str(), Some("warn") | Some("strict")),
            "severity must be warn or strict: {rule}"
        );
        assert_eq!(rule["source"], "shipped", "shipped rules must have source=shipped: {rule}");
    }
}

#[test]
fn rules_list_non_overridable_rules_have_strict_severity() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let rules = v["data"].as_array().unwrap();

    for name in ["unresolved-terms", "dangling-definition", "stale-cascade"] {
        let rule = rules.iter().find(|r| r["name"] == name).unwrap();
        assert_eq!(rule["severity"], "strict", "{name} must have strict severity");
    }
}

#[test]
fn rules_list_includes_custom_rules_from_rules_dir() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    write_custom_rule(
        &dir,
        "my-custom-rule",
        r#"?[entity_id, detail] <- [["x", "y"]]"#,
    );

    let out = dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let rules = v["data"].as_array().unwrap();
    let custom = rules
        .iter()
        .find(|r| r["name"] == "my-custom-rule")
        .expect("custom rule should appear in list");
    assert_eq!(custom["source"], "custom");
    assert!(
        matches!(custom["severity"].as_str(), Some("warn") | Some("strict")),
        "custom rule must have a severity field"
    );
}

// --- rules show ---

#[test]
fn rules_show_shipped_rule_returns_rule_envelope() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "show", "ungrounded", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "rule");
    assert_eq!(v["data"]["name"], "ungrounded");
    assert_eq!(v["data"]["source"], "shipped");
    assert!(
        matches!(v["data"]["severity"].as_str(), Some("warn") | Some("strict")),
        "severity must be present"
    );
}

#[test]
fn rules_show_custom_rule_includes_datalog_source() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let script = r#"?[entity_id, detail] <- [["entity-1", "found"]]"#;
    write_custom_rule(&dir, "my-rule", script);

    let out = dont()
        .args(["rules", "show", "my-rule", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "rule");
    assert_eq!(v["data"]["name"], "my-rule");
    assert_eq!(v["data"]["source"], "custom");
    assert_eq!(v["data"]["datalog"].as_str().unwrap().trim(), script.trim());
}

#[test]
fn rules_show_unknown_rule_returns_rule_not_found_exit_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "show", "nonexistent-rule", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "rule-not-found");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn rules_show_strict_rule_has_strict_severity() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["rules", "show", "unresolved-terms", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["data"]["severity"], "strict");
}
