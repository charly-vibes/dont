## 1. Extract import surface capability
- [x] 1.1 Write `dont-import-surface` spec for supported importer commands and source forms
- [x] 1.2 Include write targets, strict declarative idempotence (full replace-or-upsert sync), and strict no-LLM/no-MCP contract
- [x] 1.3 Include hardcoded default HTTP rate limits (with config override) and local-file exemptions
- [x] 1.4 Include rule bypass contract (imports ignore project methodology rules)

## 2. Extract LinkML adapter capability
- [x] 2.1 Write `dont-linkml-import` spec for shell-out behaviour and lossy lowering
- [x] 2.2 Include explicitly defined feature tiers: flattened-without-warning, imported-with-warning, and refused-without-partial-import
- [x] 2.3 Include pre-flight `dont doctor` check for the `linkml` CLI binary

## 3. Validate
- [x] 3.1 Run `openspec validate add-dont-import-specs --strict`
