//! Genesis::fixture adoption tests (dont-7xv8).
//!
//! Verifies the shared `init_project` helper in `tests/common` is backed by
//! `genesis::fixture::Fixture` and produces a usable dont project.

mod common;

use common::init_project;

#[test]
fn init_project_creates_initialised_dont_dir() {
    let project = init_project();
    // `dont init` writes config.toml into DONT_DIR
    assert!(project.path().join("config.toml").exists());
}

#[test]
fn init_project_dirs_are_unique_and_cleaned_up() {
    let a = init_project();
    let b = init_project();
    assert_ne!(a.path(), b.path());
    let path_a = a.path().to_path_buf();
    drop(a);
    assert!(!path_a.exists(), "fixture should clean up on drop");
}
