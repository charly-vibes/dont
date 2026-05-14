---
tags: [pipeline-run:tdd-ro5-2026-05-13-dont-4k5-8-project-initialization-error-paths, pipeline-step:red]
---

Red phase for dont-4k5.8: added failing tests in tests/init.rs for two uncovered init error-path requirements: source audit forbidding expect() in project.rs init-related code, and structured JSON error messaging for init I/O failures to include the failing path plus operation context. Both tests fail on current code: project.rs still uses expect() in init_event, and init against a file target returns 'I/O error: File exists (os error 17)' without path/operation detail.
