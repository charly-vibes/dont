# Change: add dont import adapter specs

## Why

The monolithic spec still contains the import surface that grounds `dont` against external vocabularies and references. The importer command family, its idempotence/rate-limit contract, and the LinkML adapter's special-case behaviour are not yet captured as focused OpenSpec capabilities. Extracting them makes import behaviour testable without mixing it into rule, layout, or harness specs.

## What Changes
- Add `dont-import-surface` for importer commands, write targets, strict declarative idempotence (full replace/upsert per source), hardcoded default HTTP rate limits (e.g. 5 req/sec with config override), the explicit no-LLM contract (strictly deterministic mapping), and the rule bypass contract (imports bypass project methodology rules).
- Add `dont-linkml-import` for the lossy LinkML adapter, including explicit feature tiers (flattened-without-warning, imported-with-warning, refused-without-partial-import) and `dont doctor` pre-flight integration checking for the `linkml` binary.

## Deferred
- Full project-layout/config schema for importer configuration blocks
- Import-generated rule packaging beyond the LinkML adapter's contract
- Broader MCP transport concerns

## Traceability
- `dont-import-surface` is sourced from `dont-spec-v0_3_2.md` §15 command surface and operational notes.
- `dont-linkml-import` is sourced from `dont-spec-v0_3_2.md` §15 LinkML adapter scope and auxiliary-tool dependency notes.

## Impact
- Affected specs: `dont-import-surface`, `dont-linkml-import` (both new)
- Cross-references: `dont-data-model` (imported_term/reference/prefix relations), `dont-errors` (`config-missing`, `linkml-unsupported-feature`), `dont-derived-queries` (`doctor` warn check), `dont-rule-engine` (optional generated rules)
- Affected workflow: future project-layout specs can reference importer config and doctor integration instead of restating importer semantics
