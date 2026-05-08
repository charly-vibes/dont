use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_project(dir: &TempDir) {
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

mod layout {
    use super::*;

    #[test]
    fn open_with_missing_required_subdir_returns_layout_invalid() {
        let dir = TempDir::new().unwrap();
        init_project(&dir);

        fs::remove_dir_all(dir.path().join("sessions")).unwrap();

        let output = dont()
            .args(["prime", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .code(3)
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["data"]["code"], "layout-invalid");
        let message = v["data"]["message"].as_str().unwrap();
        assert!(
            message.contains("sessions"),
            "error message should name the missing directory, got: {message}"
        );
    }

    #[test]
    fn open_validates_each_required_subdir() {
        for subdir in &["seed", "vocab", "rules", "imports", "sessions", "schemas"] {
            let dir = TempDir::new().unwrap();
            init_project(&dir);

            fs::remove_dir_all(dir.path().join(subdir)).unwrap();

            let output = dont()
                .args(["prime", "--json"])
                .env("DONT_DIR", dir.path())
                .assert()
                .code(3)
                .get_output()
                .stdout
                .clone();

            let v: Value = serde_json::from_slice(&output).unwrap();
            assert_eq!(
                v["data"]["code"], "layout-invalid",
                "expected layout-invalid for missing {subdir}"
            );
            let message = v["data"]["message"].as_str().unwrap();
            assert!(
                message.contains(subdir),
                "error message should name missing directory '{subdir}', got: {message}"
            );
        }
    }

    #[test]
    fn layout_invalid_remediation_suggests_dont_init() {
        let dir = TempDir::new().unwrap();
        init_project(&dir);
        fs::remove_dir_all(dir.path().join("schemas")).unwrap();

        let output = dont()
            .args(["prime", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .code(3)
            .get_output()
            .stdout
            .clone();

        let v: Value = serde_json::from_slice(&output).unwrap();
        let remediation = v["data"]["remediation"].as_array().unwrap();
        assert!(
            !remediation.is_empty(),
            "layout-invalid should have remediation entries"
        );
        let has_init = remediation
            .iter()
            .any(|r| r["command"].as_str().unwrap_or("").contains("dont init"));
        assert!(has_init, "remediation should suggest 'dont init': {remediation:?}");
    }
}
