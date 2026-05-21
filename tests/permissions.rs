mod common;

/// Read-only filesystem robustness tests (dont-qxrz).
///
/// Verifies that `dont` handles read-only filesystem conditions gracefully:
/// non-zero exit with a structured JSON error (`ok: false`), never a panic.
///
/// Restricted to Unix because Windows permission semantics differ.
/// Skipped when running as root (uid 0) because root ignores read-only bits.
#[cfg(unix)]
mod permissions_tests {
    use crate::common::{dont, init_dir};
    use serde_json::Value;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    /// Returns true when the current process is running as root (uid 0).
    /// Permission restrictions do not apply to root, so tests must be skipped.
    ///
    /// Detection strategy: create a temp dir, set it read-only, then try to
    /// create a file inside it. If that succeeds, we have root-level access.
    fn running_as_root() -> bool {
        let probe = TempDir::new().unwrap();
        let mut perms = fs::metadata(probe.path()).unwrap().permissions();
        perms.set_mode(0o500);
        fs::set_permissions(probe.path(), perms).unwrap();
        let can_write = fs::write(probe.path().join("probe"), b"x").is_ok();
        // Restore before drop so TempDir cleanup works.
        let mut restore = fs::metadata(probe.path()).unwrap().permissions();
        restore.set_mode(0o700);
        fs::set_permissions(probe.path(), restore).unwrap();
        can_write
    }

    // -----------------------------------------------------------------------
    // Scenario 1: the entire DONT_DIR is made read-only (mode 0o500)
    // -----------------------------------------------------------------------

    /// Making DONT_DIR read-only prevents `dont conclude` from writing new
    /// claims. The command must exit non-zero and emit `ok: false` with a
    /// meaningful message rather than panicking.
    #[test]
    fn conclude_on_readonly_dont_dir_exits_nonzero_with_ok_false() {
        if running_as_root() {
            eprintln!("skipping: running as root, permission restrictions don't apply");
            return;
        }

        let dir = TempDir::new().unwrap();
        init_dir(&dir);

        // Make DONT_DIR itself read-only (owner: r-x, no write).
        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o500);
        fs::set_permissions(dir.path(), perms).unwrap();

        let result = dont()
            .args(["conclude", "test claim", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .failure();

        let stdout = result.get_output().stdout.clone();

        // Restore write permissions so TempDir cleanup succeeds.
        let mut restore = fs::metadata(dir.path()).unwrap().permissions();
        restore.set_mode(0o700);
        fs::set_permissions(dir.path(), restore).unwrap();

        let v: Value = serde_json::from_slice(&stdout).unwrap_or_else(|_| {
            panic!(
                "output must be valid JSON, got: {:?}",
                String::from_utf8_lossy(&stdout)
            )
        });

        assert_eq!(
            v["ok"], false,
            "readonly DONT_DIR: expected ok:false, got: {v}"
        );

        // The error message must contain something useful — not be empty.
        let message = v["data"]["message"].as_str().unwrap_or("");
        assert!(
            !message.is_empty(),
            "readonly DONT_DIR: error message must not be empty, got envelope: {v}"
        );
    }

    // -----------------------------------------------------------------------
    // Scenario 2: db.cozo (the store file) is made read-only (mode 0o400)
    // -----------------------------------------------------------------------

    /// Making the store file (`db.cozo`) read-only prevents write operations.
    /// `dont conclude` must exit non-zero and emit `ok: false` rather than
    /// panicking or silently succeeding.
    #[test]
    fn conclude_on_readonly_store_file_exits_nonzero_with_ok_false() {
        if running_as_root() {
            eprintln!("skipping: running as root, permission restrictions don't apply");
            return;
        }

        let dir = TempDir::new().unwrap();
        init_dir(&dir);

        let db_path = dir.path().join("db.cozo");

        // Make the store file read-only.
        let mut perms = fs::metadata(&db_path).unwrap().permissions();
        perms.set_mode(0o400);
        fs::set_permissions(&db_path, perms).unwrap();

        let stdout = dont()
            .args(["conclude", "test claim", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();

        // Restore permissions so the directory is cleaned up properly.
        let mut restore = fs::metadata(&db_path).unwrap().permissions();
        restore.set_mode(0o600);
        fs::set_permissions(&db_path, restore).unwrap();

        let v: Value = serde_json::from_slice(&stdout).unwrap_or_else(|_| {
            panic!(
                "output must be valid JSON, got: {:?}",
                String::from_utf8_lossy(&stdout)
            )
        });

        assert_eq!(
            v["ok"], false,
            "readonly db.cozo: expected ok:false, got: {v}"
        );

        let message = v["data"]["message"].as_str().unwrap_or("");
        assert!(
            !message.is_empty(),
            "readonly db.cozo: error message must not be empty, got envelope: {v}"
        );
    }

    // -----------------------------------------------------------------------
    // Scenario 3: --json flag is required — verify structured output is present
    // -----------------------------------------------------------------------

    /// The JSON envelope produced on a read-only error must have the required
    /// envelope fields (`ok`, `envelope_version`, `data.message`).
    #[test]
    fn readonly_error_envelope_has_required_fields() {
        if running_as_root() {
            eprintln!("skipping: running as root, permission restrictions don't apply");
            return;
        }

        let dir = TempDir::new().unwrap();
        init_dir(&dir);

        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o500);
        fs::set_permissions(dir.path(), perms).unwrap();

        let stdout = dont()
            .args(["conclude", "envelope field check", "--json"])
            .env("DONT_DIR", dir.path())
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();

        let mut restore = fs::metadata(dir.path()).unwrap().permissions();
        restore.set_mode(0o700);
        fs::set_permissions(dir.path(), restore).unwrap();

        let v: Value = serde_json::from_slice(&stdout).unwrap_or_else(|_| {
            panic!(
                "output must be valid JSON, got: {:?}",
                String::from_utf8_lossy(&stdout)
            )
        });

        assert_eq!(v["ok"], false, "envelope must have ok:false");
        assert!(
            v["envelope_version"].is_string(),
            "envelope must have envelope_version, got: {v}"
        );
        assert!(
            v["data"].is_object(),
            "envelope must have a data object, got: {v}"
        );
        let message = v["data"]["message"].as_str().unwrap_or("");
        assert!(
            !message.is_empty(),
            "data.message must be non-empty, got envelope: {v}"
        );
    }
}
