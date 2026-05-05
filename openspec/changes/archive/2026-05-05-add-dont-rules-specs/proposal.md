# Change: add dont rule engine and rule-facing CLI specs

## Why

The monolithic spec still contains the methodology-as-rules contract that turns `dont` from a passive store into an epistemic forcing function. The shipped rule catalogue, severity model, and rule-facing commands (`dont rules`, `dont explain`) are not yet captured as focused OpenSpec capabilities. Extracting them makes rule behaviour testable and gives later config/import/layout work something stable to reference.

## What Changes
- Add `dont-rule-engine` for rule source format (abstract declarative graph-query/Datalog dialect), shipped-rule semantics, severity defaults (`warn` and `strict`), absolute override boundaries (shipped rules cannot be deleted or set below `warn`), and the strict distinction between rule-layer graph evaluations and verb-level input validators.
- Establish that all rules are standardized as violation queries returning `?entity_id, ?detail`.
- Add `dont-rule-cli` for `dont rules` and `dont explain`, including dry-run `test` behaviour, strict syntax validation, and the absolute requirement for structured sibling English translation documents (paired by filename base, e.g. `rule.dl` and `rule.md`, containing `# Rationale`, `# Remediation`).

## Deferred
- Full `config.toml` schema beyond rule severity references — project-layout concern
- Storage/layout details of `.dont/rules/` directory ownership — rule files are identified by name, but physical layout is deferred to project-layout
- Import-generated rules beyond the fact that they participate in the same rule surface

## Traceability
- `dont-rule-engine` is sourced from `dont-spec-v0_3_2.md` §13 plus the related `rule-not-met` clarification in §10.5.
- `dont-rule-cli` is sourced from the §10 command summaries (`dont rules`, `dont explain`), the rule-authoring how-to in §11.4.1, and the v0.3 note on `vague-reason`.

## Impact
- Affected specs: `dont-rule-engine`, `dont-rule-cli` (both new)
- Cross-references: `dont-cli-core` and `dont-lifecycle-verbs` (commands gated by rules), `dont-errors` (`rule-not-met`, `rule_name`, `spec_ref`), `dont-agent-help` (help/explanation surface), `dont-data-model` (dependency edges and referenced relations)
- Affected workflow: future project-layout and import specs can refer to this rule contract instead of restating severity and rule-file behaviour
