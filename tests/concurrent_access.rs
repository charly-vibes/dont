mod common;

use assert_cmd::cargo::cargo_bin;
use common::init_dir;
use serde_json::Value;
use tempfile::TempDir;

/// Two concurrent `dont conclude` invocations to the same store directory
/// must not corrupt data.  Both may succeed (2 claims) or one may fail
/// cleanly (1 claim + a non-zero exit code), but the store must remain
/// readable and consistent afterward.
#[test]
fn concurrent_conclude_does_not_corrupt_store() {
    let bin = cargo_bin("dont");
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let dont_dir = dir.path().to_owned();

    // Spawn two `dont conclude` processes simultaneously.
    let child_a = std::process::Command::new(&bin)
        .args(["conclude", "concurrent claim A", "--json"])
        .env("DONT_DIR", &dont_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn process A");

    let child_b = std::process::Command::new(&bin)
        .args(["conclude", "concurrent claim B", "--json"])
        .env("DONT_DIR", &dont_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn process B");

    let out_a = child_a.wait_with_output().expect("failed to wait for process A");
    let out_b = child_b.wait_with_output().expect("failed to wait for process B");

    // Each process must either succeed (exit 0) or fail with a clean JSON
    // error envelope (exit non-zero but valid JSON on stdout).  A panic
    // (non-JSON output or exit code 101) is never acceptable.
    for (label, out) in [("A", &out_a), ("B", &out_b)] {
        if !out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            // Exit code 101 means the process panicked — that is corruption.
            assert_ne!(
                out.status.code(),
                Some(101),
                "process {label} panicked (exit 101).\nstdout: {stdout}\nstderr: {stderr}"
            );

            // The stdout must still be valid JSON on a clean failure.
            let v: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
                panic!(
                    "process {label} exited with code {:?} and non-JSON stdout.\n\
                     parse error: {e}\nstdout: {stdout}\nstderr: {stderr}",
                    out.status.code()
                )
            });

            assert_eq!(
                v["ok"], false,
                "process {label} failed but ok != false.\nenvelope: {v}"
            );
        }
    }

    // After both processes finish the store must be readable and consistent.
    let list_out = std::process::Command::new(&bin)
        .args(["list", "--json"])
        .env("DONT_DIR", &dont_dir)
        .output()
        .expect("failed to run dont list");

    assert!(
        list_out.status.success(),
        "`dont list` failed after concurrent writes.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&list_out.stdout),
        String::from_utf8_lossy(&list_out.stderr),
    );

    let list_json: Value = serde_json::from_slice(&list_out.stdout)
        .expect("`dont list --json` returned non-JSON output after concurrent writes");

    assert_eq!(list_json["ok"], true, "`dont list` envelope ok != true");

    let claims = list_json["data"]["claims"]
        .as_array()
        .expect("`dont list` data.claims is not an array");

    // Both succeeded → 2 claims; one lost the race → 1 claim.  Zero or more
    // than 2 claims both indicate corruption.
    let count = claims.len();
    assert!(
        count == 1 || count == 2,
        "expected 1 or 2 claims after concurrent conclude, got {count}.\nclaims: {claims:?}"
    );

    // Every claim in the list must have a well-formed id and statement.
    for claim in claims {
        let id = claim["id"]
            .as_str()
            .expect("claim missing id field");
        assert!(
            id.starts_with("claim:"),
            "claim id does not have 'claim:' prefix: {id}"
        );

        assert!(
            claim["statement"].is_string(),
            "claim {id} has non-string statement field"
        );
    }
}
