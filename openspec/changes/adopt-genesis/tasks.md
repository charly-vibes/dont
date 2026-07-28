## 1. Dependency
- [ ] 1.1 Add `genesis = { git = "https://github.com/charly-vibes/genesis", tag = "v0.1.0" }` to `Cargo.toml`.
- [ ] 1.2 Verify the build with envelope/managed_block/suggestions modules stable.

## 2. Migrate envelope (dont-2j6o supersession)
- [ ] 2.1 Delete `src/envelope.rs`; re-export `genesis::envelope::*`.
- [ ] 2.2 Preserve envelope_version `"0.2"` contract and all `dont-envelope` requirements.
- [ ] 2.3 Regression: `dont prime --json` envelope shape unchanged (ok, envelope_version, cli_version, envelope_kind, data, warnings, hints, meta).
- [ ] 2.4 Confirm `dont-2j6o` child adoption issue for dont is closed by this change.

## 3. Migrate managed_block
- [ ] 3.1 Source the `<!-- DONT:START/END -->` injector from `genesis::managed_block`.
- [ ] 3.2 Keep dont's block content (pointer to `.dont/AGENTS.md`, auto-managed warning).
- [ ] 3.3 Regression: `dont init` / `dont doctor --fix` still inject/refresh the block.

## 4. Adopt suggestions
- [ ] 4.1 Register dont's command list with `genesis::suggestions::SuggestionEngine`.
- [ ] 4.2 Wire the error footer in `main.rs` to emit `genesis::suggestions::Suggestion` fixes.
- [ ] 4.3 Regression: `dont defin` (typo) prints "Did you mean 'define'?".

## 5. Clean up
- [ ] 5.1 Remove dead local code; `cargo clippy -- -D warnings` clean.
- [ ] 5.2 Verify tool-craft (genesis `.wai` research) Appendix A.3 dont row; file a charly-monorepo ticket if inaccurate.

## 6. Add `feedback` subcommand (wraps `genesis::feedback`)
- [ ] 6.1 Add `Feedback` variant to the `Commands` enum with `KIND` + flags (per agent-issue-reporting playbook §2).
- [ ] 6.2 Read dont's error scratch (`$XDG_CACHE_HOME/dont/errors.jsonl`) for `--from-last-error`; never shadow the real error.
- [ ] 6.3 Default target repo = dont's `Cargo.toml` `repository`; labels from playbook §8.
- [ ] 6.4 Wire the error-footer hook: non-zero exits with no `genesis::suggestions::Fix` print `Feedback: dont feedback bug --from-last-error`.
- [ ] 6.5 Regression: `dont feedback bug --dry-run` prints body + exact `gh` line; redactor strips a `https://<pat>@…` remote.
