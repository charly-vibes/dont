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
    serde_json::from_slice::<Value>(&out).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn define_term(dir: &TempDir, curie: &str, doc: &str) -> String {
    let out = dont()
        .args(["define", curie, "--doc", doc, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<Value>(&out).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// CURIE resolves to the correct term — satisfies "Done when: cargo test entity_lookup_curie"
#[test]
fn entity_lookup_curie() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    define_term(&dir, "WB:P001", "wellbeing principle 1");

    let out = dont()
        .args(["show", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["curie"], "WB:P001");
}

/// claim:PREFIX (8 chars) resolves to the correct claim
#[test]
fn entity_lookup_claim_short_prefix() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "the earth orbits the sun");
    let prefix8 = &id.strip_prefix("claim:").unwrap()[..8];

    let out = dont()
        .args(["show", &format!("claim:{prefix8}"), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "claim");
    assert_eq!(v["data"]["id"], id.as_str());
}

/// Bare short prefix (no colon) resolves to a claim when unique
#[test]
fn entity_lookup_bare_prefix_resolves_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "bare prefix test claim");
    let prefix8 = &id.strip_prefix("claim:").unwrap()[..8];

    let out = dont()
        .args(["show", prefix8, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["id"], id.as_str());
}

/// Bare short prefix resolves to a term when unique
#[test]
fn entity_lookup_bare_prefix_resolves_term() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = define_term(&dir, "WB:P042", "bare prefix term test");
    let prefix8 = &id.strip_prefix("term:").unwrap()[..8];

    let out = dont()
        .args(["show", prefix8, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["id"], id.as_str());
}

/// Prefix that matches no entity exits non-zero with a clear error
#[test]
fn entity_lookup_prefix_not_found_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "some claim");

    let out = dont()
        .args(["show", "claim:00000000", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
}

/// Empty prefix (claim:) with 2 claims → ambiguous-prefix error, exit non-zero
#[test]
fn entity_lookup_ambiguous_prefix_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    conclude_claim(&dir, "first claim");
    conclude_claim(&dir, "second claim");

    // "claim:" with no suffix is an empty prefix — matches every claim
    let out = dont()
        .args(["show", "claim:", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["data"]["code"].as_str().unwrap_or(""),
        "ambiguous-prefix",
        "expected ambiguous-prefix error, got: {v}"
    );
}

/// Lowercase prefix resolves the same as uppercase (ULID case-insensitivity)
#[test]
fn entity_lookup_lowercase_prefix_resolves_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "lowercase prefix test claim");
    let prefix8_upper = &id.strip_prefix("claim:").unwrap()[..8];
    let prefix8_lower = prefix8_upper.to_lowercase();

    let out = dont()
        .args(["show", &format!("claim:{prefix8_lower}"), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["id"], id.as_str());
}

/// why with a short prefix resolves to the correct claim (regression guard for why dispatch)
#[test]
fn entity_lookup_why_short_prefix() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "why short prefix test claim");
    let prefix8 = &id.strip_prefix("claim:").unwrap()[..8];

    let out = dont()
        .args(["why", &format!("claim:{prefix8}"), "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["entity"]["id"], id.as_str());
}

/// Full claim:ULID still resolves correctly (regression guard)
#[test]
fn entity_lookup_full_id_regression() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "regression guard claim");

    let out = dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["id"], id.as_str());
}
