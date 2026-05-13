---
tags: [pipeline-run:tdd-ro5-2026-05-13-dont-4k5-8-project-initialization-error-paths, pipeline-step:ship]
---

Cycle complete for dont-4k5.8

Checklist:
- All tests pass: yes (`cargo test`)
- All review findings addressed: yes
- No TODO/FIXME left in `src/` or `tests/`: yes
- Changes are ready for atomic commit: yes

Outcome:
- Init and managed-doc failure paths now report write/create context with target paths.
- Coverage now includes malformed pre-existing config, late `.gitignore` write failure, and managed-doc write failure during `doctor --fix`.
- Review-only naming mismatch was corrected by renaming the broad `.expect(` guard test.
