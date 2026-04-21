# Change: add dont rule engine and rule-facing CLI specs

## Why

The monolithic spec still contains the methodology-as-rules contract that turns `dont` from a passive store into an epistemic forcing function. The shipped rule catalogue, severity model, and rule-facing commands (`dont rules`, `dont explain`) are not yet captured as focused OpenSpec capabilities. Extracting them makes rule behaviour testable and gives later config/import/layout work something stable to reference.

## What Changes
- Add `dont-rule-engine` for rule source format, shipped-rule semantics, severity defaults, override boundaries, and the distinction between rule-layer failures and verb-level validators.
- Add `dont-rule-cli` for `dont rules` and `dont explain`, including list/show/add/test behaviour and the requirement for sibling English translations.

## Deferred
- Full `config.toml` schema beyond rule severity references — project-layout concern
- Storage/layout details of `.dont/rules/` directory ownership beyond the rule-file contract
- Import-generated rules beyond the fact that they participate in the same rule surface

## Traceability
- `dont-rule-engine` is sourced from `dont-spec-v0_3_2.md` §13 plus the related `rule-not-met` clarification in §10.5 and the v0.3 note on `vague-reason`.
- `dont-rule-cli` is sourced from the §10 command summaries (`dont rules`, `dont explain`) and the rule-authoring how-to in §11.4.1.

## Impact
- Affected specs: `dont-rule-engine`, `dont-rule-cli` (both new)
- Cross-references: `dont-cli-core` and `dont-lifecycle-verbs` (commands gated by rules), `dont-errors` (`rule-not-met`, `rule_name`, `spec_ref`), `dont-agent-help` (help/explanation surface), `dont-data-model` (dependency edges and referenced relations)
- Affected workflow: future project-layout and import specs can refer to this rule contract instead of restating severity and rule-file behaviour
