# Changelog

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
