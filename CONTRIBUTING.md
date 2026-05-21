# Contributing

## Prerequisites

- [Rust](https://rustup.rs) — stable toolchain (edition 2024)
- [just](https://just.systems) — command runner (`cargo install just` or `brew install just`)
- [wai](https://github.com/charly-vibes/wai) — workflow context tracking
- [bd / beads](https://github.com/charly-vibes/beads) — issue tracking (`bd` CLI)
- [mdBook](https://rust-lang.github.io/mdBook/) — docs build (`cargo install mdbook`)
- [typos](https://github.com/crate-ci/typos) — spell checker (`cargo install typos-cli`)
- [vale](https://vale.sh) — prose linter
- [prek](https://github.com/charly-vibes/prek) — pre-commit hooks

## Setup

```bash
git clone https://github.com/charly-vibes/dont.git
cd dont
cargo build
```

Verify the build works:

```bash
cargo test
just doctor
```

## Workflow

1. Run `wai status` to orient yourself — see the active project phase and suggestions.
2. Search existing context: `wai search "<topic>"` before starting new research or tickets.
3. Pick up an issue: `bd ready` or `just ready`.
4. Capture reasoning with `wai add research`, `wai add design`, or `wai add plan`.
5. Run quality checks before committing:

```bash
just ci
```

`just ci` runs tests, lints, docs build, claim check, and `wai doctor` in sequence.

## Key Commands

```bash
just build        # cargo build
just test         # cargo test
just lint         # rustfmt check + clippy + prek + typos + vale
just ci           # full check suite (run before every commit)
just docs-build   # build the mdBook site locally
just status       # wai status
just doctor       # wai doctor
just ready        # bd ready — unblocked issues
```

## Code Style

- Follow standard Rust conventions enforced by `cargo clippy --all-targets --all-features -- -D warnings`.
- Run `just lint` before committing — it includes `cargo fmt --all --check` and is included in `just ci`.
- No `#[allow(...)]` attributes without a comment explaining why.
- Tests live alongside the code they test (inline `#[cfg(test)]` modules) or in `tests/` for integration tests.
- Each public function and struct should have a doc comment.

## PR Process

1. Create or claim an issue with `bd` before starting work.
2. Work on a branch named after the issue or feature (e.g. `add-rule-my-rule`).
3. Write tests first (TDD) — red, green, refactor, in separate commits where practical.
4. Separate refactoring commits from feature commits.
5. Run `just ci` and ensure it passes completely.
6. Open a PR. The PR description should reference the `bd` issue ID.
7. Spec-driven changes (new capabilities, breaking changes) require an `openspec` proposal
   approved before implementation — see [`openspec/AGENTS.md`](openspec/AGENTS.md).

## Adding a New Built-in Rule

Built-in (shipped) rules live in `src/rules/`. Each rule is a Rust module:

1. **Create the rule module** — `src/rules/<rule_name>.rs`:

   ```rust
   use crate::store::{Store, StoreError};
   use super::RuleMatch;

   pub const EXPLANATION: &str = include_str!("<rule_name>.md");

   pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
       // Return a RuleMatch for each violation found.
       Ok(vec![])
   }
   ```

2. **Write the explanation** — `src/rules/<rule_name>.md`:
   Plain prose that `dont rules explain <rule_name>` will display to users.

3. **Register in `src/rules/mod.rs`**:
   - Add `pub mod <rule_name>;` at the top with the other module declarations.
   - Add a private wrapper function `fn check_<rule_name>(...) -> ...`.
   - Add a `ShippedRule` entry to `SHIPPED_RULE_CATALOG`.

4. **Write tests** — inline `#[cfg(test)]` module in your rule file, following the
   pattern in existing rules (see `src/rules/ungrounded.rs`).

5. **Run `just ci`** to confirm everything passes.

For Datalog-based rules (file-based, not shipped), place a `<rule_name>.dl` file in the
project's rules directory. The rule query MUST return columns `[entity_id, detail]`.

## Project Conventions

- Prefer `just` recipes over raw `cargo` or shell commands.
- Keep reasoning in `wai` artifacts when making design decisions.
- Use `prek` hooks for basic hygiene checks; pre-commit includes `cargo fmt --all --check` as a blocker.
- Update specs and project context together with code changes.
- Spec-level changes go through the `openspec` proposal workflow.
