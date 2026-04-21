# Change: add dont project layout and config specs

## Why

The monolithic spec still contains the persistent project structure and `config.toml` surface that all other capabilities rely on. The `.dont/` directory layout, managed-doc ownership, and operational configuration blocks are not yet captured as focused OpenSpec capabilities. Extracting them makes environment and configuration expectations testable and gives implementation work a stable filesystem/config contract.

## What Changes
- Add `dont-project-layout` for the `.dont/` directory structure, owned artifacts, and root managed-doc relationship.
- Add `dont-project-config` for the `config.toml` surface: project mode, output defaults, direct-mode LLM config, harness config, rule severity config, trust hedge patterns, storage tuning, verify-evidence tuning, and importer config blocks.

## Deferred
- Implementation-specific RocksDB or Cozo internals beyond the presence of the store path and exposed tuning knobs
- Future migration tooling such as `dont migrate-seed`
- Authentication, authorization, and encryption concerns that remain out of scope for v0.3

## Traceability
- `dont-project-layout` is sourced from `dont-spec-v0_3_2.md` §14 directory layout and managed-doc ownership statements.
- `dont-project-config` is sourced from `dont-spec-v0_3_2.md` §14 `config.toml` example plus cross-referenced operational notes in §§8, 9, 9A, 12, 13, and 15.

## Impact
- Affected specs: `dont-project-layout`, `dont-project-config` (both new)
- Cross-references: `dont-agent-help` (managed docs and `.dont/AGENTS.md`), `dont-spawn-protocol` (harness and direct-mode config), `dont-rule-engine` (rule severities), `dont-import-surface` (importer config), `dont-data-model` (store path / schema directory)
- Affected workflow: future init/migration implementation can target a stable on-disk and config contract instead of inferring it from the monolith
