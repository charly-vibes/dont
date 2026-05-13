---
tags: [pipeline-run:tdd-ro5-2026-05-13-dont-4k5-8-project-initialization-error-paths, pipeline-step:fix]
---

Review findings addressed for dont-4k5.8

Implemented fixes:
- Wrapped managed-doc writes in `src/project.rs:306-314` with contextual `io_error("write", path, err)` mapping so managed-doc failures now report both operation and path instead of falling back to generic `I/O error: ...`.
- Added integration coverage in `tests/init.rs` for malformed pre-existing config (`init_treats_malformed_existing_config_as_already_initialised`) and for a late `.gitignore` write failure after normal init setup (`init_reports_late_gitignore_write_failures_with_path_context`).
- Added managed-doc failure coverage in `tests/managed_docs.rs` with `doctor_fix_reports_managed_doc_write_failures_with_path_context`, proving the shared refresh path now emits contextual write errors.
- Renamed the broad `.expect(` guard test to `project_source_has_no_expect_calls` so its name matches its actual file-wide scope.

Validation:
- `cargo test --test init --test managed_docs` ✅
- `cargo test` ✅ full suite passing
