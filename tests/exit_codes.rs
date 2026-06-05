mod common;

use common::{conclude_claim, dont, init_dir};
use tempfile::TempDir;

// --- exit 0 on success ---

#[test]
fn init_exits_0_on_success() {
    let dir = TempDir::new().unwrap();
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn version_exits_0() {
    dont().args(["--version"]).assert().success().code(0);
}

#[test]
fn conclude_exits_0_on_success() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["conclude", "test claim", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn define_exits_0_on_success() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["define", "ex:Foo", "--doc", "a term", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn list_exits_0_when_empty() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn prime_exits_0_when_no_blockers() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn show_exits_0_for_existing_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    let id = conclude_claim(&dir, "a claim to show");
    dont()
        .args(["show", &id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn completions_exits_0() {
    dont()
        .args(["completions", "bash", "--json"])
        .assert()
        .success()
        .code(0);
}

#[test]
fn rules_list_exits_0() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["rules", "list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .code(0);
}

// --- exit non-zero on error ---

#[test]
fn missing_required_arg_exits_2() {
    // clap usage errors are exit 2
    dont().args(["conclude"]).assert().failure().code(2);
}

#[test]
fn unknown_flag_exits_2() {
    dont()
        .args(["--totally-unknown-flag"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn re_init_exits_1_not_3() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    // second init should refuse with exit 1 (general error), not 3
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn show_nonexistent_id_exits_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["show", "claim:00000000000000000000000000", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn conclude_empty_statement_exits_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["conclude", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn define_empty_doc_exits_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["define", "ex:Foo", "--doc", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn explain_unknown_rule_exits_1() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);
    dont()
        .args(["explain", "no-such-rule", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn config_missing_exits_1_not_3() {
    // A dir that exists but was never initialised — open_project_or_exit hits ConfigMissing
    let dir = TempDir::new().unwrap();
    dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .failure()
        .code(1);
}
