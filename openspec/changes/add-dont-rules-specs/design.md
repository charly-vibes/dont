## Context

The repo already has focused specs for verbs, lifecycle, envelope/error contracts, data shapes, and harness/help surfaces. What remains central but undecomposed is the rule layer: the shipped methodology rules, the severity model that changes behaviour between permissive and strict modes, and the command surface for listing, testing, adding, and explaining rules. These behaviours live mainly in §13, but they also connect to §10.5 errors and §11.4.1 rule authoring guidance.

## Goals
- Capture the methodology-as-rules model as a standalone capability independent of CLI verbs and storage substrate
- Capture the operator-facing rule CLI and explanation surface as a separate capability
- Preserve the v0.3 distinction between rule-layer refusals and verb-level validators like `reason-required`

## Non-Goals
- Specify the full project layout or `config.toml` schema
- Specify import-generated rule translation details
- Re-specify command semantics already covered in core/lifecycle verb specs

## Decisions
- **Two capabilities, not one**: the rule engine (what rules are, what ships, when they warn/refuse) changes independently from the CLI affordances for managing and explaining them.
- **Rule semantics remain normative even without Datalog syntax details**: the spec names Cozo Datalog and the sibling translation file requirement, but focuses normatively on externally visible semantics rather than parser internals.
- **Verb-level validators stay out of the rule engine**: `reason-required` and `reason-not-hedge` are explicitly excluded from the shipped rule catalogue to preserve the v0.3 split and prevent configuration ambiguity.
- **`dont explain` belongs with rule CLI**: although it is part of the broader help surface, its source of truth is the rule-specific sibling `.md` file and its audience is rule interpretation.

## Source Mapping
- `dont-rule-engine`: §13 default rules and severity table; §10.5 `rule_name`, `rule-not-met`, and warning semantics; v0.3 notes on `vague-reason`
- `dont-rule-cli`: §10 command summaries for `rules` and `explain`; §11.4.1 workflow for authoring rule `.dl` + `.md`, testing, and severity assignment

## Risks / Trade-offs
- The rule engine spec could drift into implementation details about Datalog evaluation.
  - Mitigation: keep requirements at the contract level: rule source format, shipped semantics, severity outcomes, and override boundaries.
- The rule CLI may overlap with help/documentation specs.
  - Mitigation: scope `dont explain` narrowly to rule documentation sourced from sibling files.
- Some rule semantics depend on data-model details like dependency edges and evidence provenance.
  - Mitigation: reference `dont-data-model` rather than restating relation definitions.

## Open Questions
- Whether future user-defined rule packaging needs its own capability once import-generated and bundled rule sets arrive.
