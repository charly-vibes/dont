## 1. Extract rule engine capability
- [x] 1.1 Write `dont-rule-engine` spec for the single rule source format (abstract Datalog dialect) and structured sibling translation-doc requirement (`# Rationale`, `# Remediation`)
- [x] 1.2 Include the shipped rule catalogue, enforcing absolute minimum severity (`warn`) and default configurations
- [x] 1.3 Include severity defaults (`warn` and `strict`), override boundaries, and rule invariants
- [x] 1.4 Include the absolute split between rule-layer graph failures (violation queries) and verb-level validators (e.g. `reason-required`, `evidence-required`)

## 2. Extract rule CLI capability
- [x] 2.1 Write `dont-rule-cli` spec for `rules list`, `show`, `add`, and `test`
- [x] 2.2 Include `dont explain <rule>` as a static read of the sibling English translation, not a graph execution
- [x] 2.3 Include dry-run `test` behaviour against temporary snapshots, and specific error envelopes for syntax validation (`code: "rule-syntax-error"`) and missing/malformed sibling documents (`code: "rule-missing-doc"`)

## 3. Validate
- [x] 3.1 Run `openspec validate add-dont-rules-specs --strict`
