# Rule of 5 review: managed docs refresh via doctor --fix

- Accuracy: managed_block normalization, root-block injection, init parity, and doctor --fix round-trips behave correctly under the new tests; full `cargo test` passes.
- Completeness findings:
  - MEDIUM: `doctor` currently hard-codes several non-managed checks as `pass`, especially `seed_snapshot`, without verifying on-disk seed state.
  - MEDIUM: the compatibility path where `DONT_DIR` does not end in `.dont` intentionally skips root managed docs, but that behavior is implicit rather than surfaced in `doctor` detail or code comments.
- Clarity: code is readable after extracting `emit_project_error_and_exit` and isolating `managed_block` helpers.
- Integration: new `config.harness` defaults, `.dont/AGENTS.md` generation, and root `AGENTS.md`/`CLAUDE.md` injection fit existing init/project flows without breaking prior tests.
- Recommended fixes:
  1. Make `doctor` verify seed snapshot presence rather than hard-coding `pass`.
  2. Clarify the direct-override compatibility path so future work does not accidentally regress root-doc behavior.
