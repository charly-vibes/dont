## 1. Extract import surface capability
- [ ] 1.1 Write `dont-import-surface` spec for supported importer commands and source forms
- [ ] 1.2 Include write targets, idempotence, and no-LLM/no-MCP contract
- [ ] 1.3 Include HTTP rate limiting and local-file exemptions
- [ ] 1.4 Include auxiliary-tool availability expectations and doctor integration

## 2. Extract LinkML adapter capability
- [ ] 2.1 Write `dont-linkml-import` spec for shell-out behaviour and lossy lowering
- [ ] 2.2 Include flattened, warning, and refusal feature tiers
- [ ] 2.3 Include no-partial-import behaviour on unsupported features

## 3. Validate
- [ ] 3.1 Run `openspec validate add-dont-import-specs --strict`
