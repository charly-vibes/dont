# Changelog

## [0.3.0] — 2026-07-31

### Changed
- Adopt ALL genesis v0.4.0 shared modules — stop reimplementing shared infrastructure (dont-ctrh):
  - `genesis::config` for shared config I/O
  - `genesis::guide` for CLI scaffold and ErrorSink
  - `genesis::doctor` — doctor command refactored onto the `DoctorCheck` trait
  - `genesis::scaffold`, `discovery`, `aix`, `suite_linter`, `cli` (completions, version JSON)
  - `genesis::guide`/`status` — `CliVerbosity`, `CliFormat`, `DoctorStatusBridge`
- Depend on published `genesis-vibes` crate from crates.io instead of path deps.

### Added
- `mine-claims` script — extract epistemic claims from repo artifacts.
- `wasm-claims-query` openspec change proposal.

### Fixed
- Safe UTF-8 truncation in `dont list` output (dont-edoc).

### Internal
- Seed repo with mined claims; ignore stale unverified claims.
- Archive genesis openspec changes; resolve spec-test drift.
- Test coverage for ErrorSink scratch and `feedback --from-last-error` loop.
- Refresh wai managed blocks to 2026.7.29.
- Add epigraph quote to README and docs homepage.

## [0.2.2] — 2026-07-18

### Internal
- Deploy `ah` check via `ah init` for agent-health verification. (dont-bt4n)
- Replace pre-push shell script with native `testaruda --safe` mode. (dont-j9k3)
- Init `testaruda` for selective test runs on pre-push. (dont-m2x7)
- Add Git & Workflow Discipline section to AGENTS.md. (dont-r5p1)
- Remove `prek` from lint recipe after `prek.toml` removal. (dont-w8c6)
- Investigation: full findings on why agents don't use `dont`. (dont-f2v9)

## [0.2.1] — 2026-07-12

### Fixed
- `dont flag --file` on staged-but-uncommitted files now works correctly. (dont-h1l5)
- `dont prime` excludes ignored claims from status counts. (dont-08m3)

### Changed
- CLI consistency audit: unified flag naming and output formatting.
- Published as `dont-cli` on crates.io (`dont` name taken); binary names remain `dont`/`dt`.

### Internal
- Fix CI: `clippy::question_mark` lint, install `wai-cli` (crate was renamed), trailing newlines for generated files.
- Enable beads backup via tracked `issues.jsonl`.
- Add installation section with `cargo install` to README.

## [0.2.0] — 2026-07-08

### Added
- `dont flag --file` now accepts tracked files with unstaged (dirty) modifications.
  Uses `git hash-object` to produce a `git:content:<sha>` provenance ref instead
  of requiring commit-before-ground. (dont-h1l5)
- Semicolons are now allowed in claim statement text. Prose punctuation (`;`, `:`, `/`)
  is no longer rejected by the shell-metacharacter guard. Genuine injection vectors
  (`|`, `` ` ``, `$`, `\`, `<`, `>`, NUL) remain blocked. (dont-qau6)
- Duplicate claim error (`dont ground` / `dont conclude` on an existing statement)
  now suggests `dont flag <id> --evidence <locator>` as an actionable next step,
  in addition to `dont show`. (dont-cfki)
- Mode baseline write failures are silently ignored instead of printing a warning
  on every command. Mode tracking is best-effort infrastructure. (dont-tp8f)
