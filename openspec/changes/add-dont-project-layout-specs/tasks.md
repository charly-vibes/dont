## 1. Extract project-layout capability
- [ ] 1.1 Write `dont-project-layout` spec for the `.dont/` directory structure and owned artifacts
- [ ] 1.2 Include canonical doc ownership and root managed-block relationships
- [ ] 1.3 Include rule/import/session/schema subdirectories and their roles

## 2. Extract project-config capability
- [ ] 2.1 Write `dont-project-config` spec for `[project]`, `[output]`, and `[llm]`
- [ ] 2.2 Include `[harness]`, `[rules]`, and `[trust.hedges]`
- [ ] 2.3 Include `[storage]`, `[verify_evidence]`, and `[import]`
- [ ] 2.4 Include cross-feature effects such as mode changes and direct-mode boundaries

## 3. Validate
- [ ] 3.1 Run `openspec validate add-dont-project-layout-specs --strict`
