//! CI pipeline contract tests.
//!
//! These tests read the workflow and justfile on disk and assert that
//! test execution is wired into the CI pipeline. They are intentionally
//! file-level checks (not unit tests of application logic) so that a
//! misconfigured CI recipe is caught before it reaches the remote.
const CI_WORKFLOW: &str = include_str!("../.github/workflows/ci.yml");
const DOCS_WORKFLOW: &str = include_str!("../.github/workflows/docs.yml");
const JUSTFILE: &str = include_str!("../justfile");

/// The `ci:` recipe in the justfile must invoke tests so that every CI
/// run exercises the test suite.
#[test]
fn ci_recipe_invokes_tests() {
    // Collect only the lines that belong to the `ci:` recipe.
    let ci_body: String = JUSTFILE
        .lines()
        .skip_while(|l| !l.starts_with("ci:"))
        // keep the header line and all indented continuation lines
        .take_while(|l| l.starts_with("ci:") || l.starts_with("  ") || l.starts_with('\t'))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        ci_body.contains("just test") || ci_body.contains("cargo test"),
        "The `ci:` recipe in the justfile must call `just test` or `cargo test`.\n\
         Current ci recipe body:\n{ci_body}"
    );
}

/// The docs workflow must trigger on pull_request events so that doc
/// build failures are caught before merging, not only after a push to main.
#[test]
fn docs_workflow_triggers_on_pull_request() {
    assert!(
        DOCS_WORKFLOW.contains("pull_request"),
        "docs.yml must have a `pull_request:` trigger so doc-build failures \
         are caught on every PR, not only after merging to main."
    );
}

/// The workflow file must contain a reference to test execution so that
/// the intent is visible to reviewers reading the YAML directly.
#[test]
fn ci_workflow_references_test_execution() {
    assert!(
        CI_WORKFLOW.contains("cargo test")
            || CI_WORKFLOW.contains("just test")
            || CI_WORKFLOW.contains("just ci"),
        "The CI workflow must invoke tests (via `cargo test`, `just test`, or `just ci` \
         which itself must call tests).\nWorkflow content does not contain any of these."
    );
}

/// A release workflow must exist so that tagged releases are automated
/// and produce auditable artifacts (binary + checksum).
///
/// Checks:
///   1. A file matching `*release*.yml` or `*release*.yaml` exists under
///      `.github/workflows/`.
///   2. It triggers on pushed version tags (`push: tags:`).
///   3. It builds a release binary (`cargo build --release`).
#[test]
fn release_workflow_exists_with_required_steps() {
    use std::fs;
    use std::path::Path;

    let workflows_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");

    // 1. Find a release workflow file.
    let release_file = fs::read_dir(&workflows_dir)
        .expect("cannot read .github/workflows/")
        .filter_map(|e| e.ok())
        .find(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            (name.contains("release")) && (name.ends_with(".yml") || name.ends_with(".yaml"))
        });

    let release_entry = release_file.unwrap_or_else(|| {
        panic!(
            "No release workflow found in .github/workflows/. \
             Expected a file whose name contains 'release' with a .yml or .yaml extension."
        )
    });

    let content =
        fs::read_to_string(release_entry.path()).expect("failed to read release workflow file");

    // 2. Must trigger on pushed version tags.
    assert!(
        content.contains("push:") && content.contains("tags:"),
        "Release workflow must trigger on `push: tags:` (e.g. \"v*\"). \
         Neither 'push:' nor 'tags:' found together in the workflow."
    );

    // 3. Must build a release binary.
    assert!(
        content.contains("cargo build --release"),
        "Release workflow must run `cargo build --release` to produce the release binary."
    );
}
