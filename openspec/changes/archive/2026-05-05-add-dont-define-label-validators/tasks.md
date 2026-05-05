## 1. CLI core — label flag and validators

- [x] 1.1 MODIFY `dont-cli-core`: extend "Define introduces coined terms" with strictly required `--label` flag description and validator preconditions
- [x] 1.2 ADD `dont-cli-core`: "Define label shape validators" requirement with all five pass/fail validator scenarios (no magic text extraction)

## 2. Error codes — new refusal codes

- [x] 2.1 MODIFY `dont-errors`: extend "Known error codes for envelope version 0.2" to include the five new `term-*` strict refusal codes
- [x] 2.2 MODIFY `dont-errors`: extend "Scope boundary for rule-not-met" to name the new term-* codes as verb-level validators
- [x] 2.3 MODIFY `dont-errors`: add `term-nonfunctional-label` (rule-layer origin, minimum warn severity)

## 3. Rule engine — nonfunctional label rule

- [x] 3.1 MODIFY `dont-rule-engine`: extend "Shipped rule catalogue" to include `term-nonfunctional-label` as a shipped rule with a minimum severity of `warn` and translation requirement

## 4. Config — define shape and nonfunctional rule blocks

- [x] 4.1 ADD `dont-project-config`: "[define.shape] configuration" requirement covering the validator toggles and compound-marker extension
- [x] 4.2 ADD `dont-project-config`: "[rules.term_nonfunctional] configuration" requirement covering configuration of the heuristic rule patterns

## 5. Data model — strictly required label field on local coined terms

- [x] 5.1 MODIFY `dont-data-model`: extend "Term-specific attributes" to note the strictly required `label` field for coined terms, remaining optional only for imported/seed terms

## 6. Agent help — orientation block guidance

- [x] 6.1 MODIFY `dont-agent-help`: extend "Orientation prompt contract" to require the `--label` coining guidance line

## 7. Validate

- [x] 7.1 Run `openspec validate add-dont-define-label-validators --strict`
