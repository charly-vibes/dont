## 1. Extract rule engine capability
- [ ] 1.1 Write `dont-rule-engine` spec for the single rule source format and sibling translation-doc requirement
- [ ] 1.2 Include the shipped rule catalogue and each rule's semantic role
- [ ] 1.3 Include severity defaults, override boundaries, and mode-dependent behaviour
- [ ] 1.4 Include the split between rule-layer failures and verb-level validators

## 2. Extract rule CLI capability
- [ ] 2.1 Write `dont-rule-cli` spec for `rules list`, `show`, `add`, and `test`
- [ ] 2.2 Include `dont explain <rule>` and the requirement to read the sibling English translation
- [ ] 2.3 Include dry-run/testing and compile-failure behaviour

## 3. Validate
- [ ] 3.1 Run `openspec validate add-dont-rules-specs --strict`
