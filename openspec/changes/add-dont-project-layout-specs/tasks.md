## 1. Extract project-layout capability
- [x] 1.1 Write `dont-project-layout` spec for the `.dont/` directory structure, making it strictly self-contained
- [x] 1.2 Include canonical doc ownership and root managed-block relationships via `sync-docs`
- [x] 1.3 Include rule subdirectories with convention-over-configuration auto-loading, plus import/session/schema roles

## 2. Extract project-config capability
- [x] 2.1 Write `dont-project-config` spec requiring strict failure on missing/unparseable config, covering `[project]`, `[output]`, and `[llm]`
- [x] 2.2 Include `[harness]` (strict separation from `[llm]`), `[rules]`, and `[trust.hedges]` (as case-insensitive substrings, not regexes)
- [x] 2.3 Include `[storage]`, `[verify_evidence]` (with `max_concurrent_requests` for network politeness), and `[import]`
- [x] 2.4 Include cross-feature effects such as mode changes and direct-mode boundaries

## 3. Validate
- [ ] 3.1 Run `openspec validate add-dont-project-layout-specs --strict`
