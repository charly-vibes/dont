## 1. Specification
- [ ] 1.1 Create `specs/dont-glossary/spec.md` as a core glossary slice sourced from the v0.3.2 monolith Appendix A
- [ ] 1.2 Define canonical terms and aliases for the current core vocabulary (`dont-core`, `dont-status-lifecycle`, `dont-cli-core`)
- [ ] 1.3 Keep glossary entries definitional and point behavior-owning semantics to the relevant capability specs
- [ ] 1.4 Document the canonical-spec cross-linking convention in `design.md`
- [ ] 1.5 Update `openspec/project.md` with a concise Term Index that points to `dont-glossary`

## 2. Deferred follow-on integration
- [ ] 2.1 After archive, add glossary links to canonical specs such as `dont-core`, `dont-status-lifecycle`, and `dont-cli-core`
- [ ] 2.2 Expand the glossary in later changes to cover the remaining Appendix A terms
- [ ] 2.3 Add repository documentation hygiene for Markdown link verification if needed

## 3. Validation
- [ ] 3.1 Run `openspec validate add-dont-glossary-spec --strict`
- [ ] 3.2 Fix any structural or wording issues found during validation
