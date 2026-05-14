/// CI pipeline contract tests.
///
/// These tests read the workflow and justfile on disk and assert that
/// test execution is wired into the CI pipeline.  They are intentionally
/// file-level checks (not unit tests of application logic) so that a
/// misconfigured CI recipe is caught before it reaches the remote.

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
        CI_WORKFLOW.contains("cargo test") || CI_WORKFLOW.contains("just test") || CI_WORKFLOW.contains("just ci"),
        "The CI workflow must invoke tests (via `cargo test`, `just test`, or `just ci` \
         which itself must call tests).\nWorkflow content does not contain any of these."
    );
}
