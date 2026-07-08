mod common;

use common::{dont, init_dir};
use std::fs;
use tempfile::TempDir;

// --- Mode tracking ---

/// The mode baseline warning must not appear on stderr when the events file
/// is unwritable. Mode tracking is best-effort infrastructure; failures
/// should be silent, not pollute user-facing stderr.
#[test]
fn mode_baseline_write_failure_is_silent() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // When DONT_DIR is set, the DONT_DIR path IS the .dont/ directory
    let events_path = dir.path().join("events.jsonl");
    assert!(
        events_path.exists(),
        "init must create events.jsonl at {:?}",
        events_path
    );

    // Make the events file read-only after recreating it (so init succeeded)
    // so that the append attempt fails.
    fs::write(&events_path, "").unwrap();
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&events_path, Permissions::from_mode(0o444)).unwrap();
    }

    // Run a simple command that opens the project
    let output = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("could not write mode baseline"),
        "mode baseline write failure must not emit a warning, got stderr: {stderr:?}"
    );

    // Restore permissions so cleanup works
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&events_path, Permissions::from_mode(0o600)).unwrap();
    }
}
