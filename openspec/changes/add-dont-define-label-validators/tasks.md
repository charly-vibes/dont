## 1. CLI core — label flag and validators

- [ ] 1.1 MODIFY `dont-cli-core`: extend "Define introduces coined terms" with `--label` flag description and validator precondition
- [ ] 1.2 ADD `dont-cli-core`: "Define label shape validators" requirement with all five validator scenarios

## 2. Error codes — new refusal and warning codes

- [ ] 2.1 MODIFY `dont-errors`: extend "Known error codes for envelope version 0.2" to include the five new `term-*` refusal codes
- [ ] 2.2 MODIFY `dont-errors`: extend warning codes list with the three new `term-doc-shape-*` warning codes
- [ ] 2.3 MODIFY `dont-errors`: extend "Scope boundary for rule-not-met" to name the new term-* codes as verb-level validators
- [ ] 2.4 MODIFY `dont-errors`: add `term-nonfunctional-label` to warning codes list (rule-layer origin)

## 3. Rule engine — off-by-default nonfunctional label rule

- [ ] 3.1 MODIFY `dont-rule-engine`: extend "Shipped rule catalogue" to include `term-nonfunctional-label` as off-by-default warn rule with translation requirement

## 4. Config — define shape and nonfunctional rule blocks

- [ ] 4.1 ADD `dont-project-config`: "[define.shape] configuration" requirement covering the validator toggles and compound-marker extension
- [ ] 4.2 ADD `dont-project-config`: "[rules.term_nonfunctional] configuration" requirement covering enable flag and pattern extension

## 5. Data model — optional label field on terms

- [ ] 5.1 MODIFY `dont-data-model`: extend "Term-specific attributes" to note the optional `label` field and its seed-immunity behaviour

## 6. Agent help — orientation block guidance

- [ ] 6.1 MODIFY `dont-agent-help`: extend "Orientation prompt contract" to require the `--label` coining guidance line

## 7. Validate

- [ ] 7.1 Run `openspec validate add-dont-define-label-validators --strict`
