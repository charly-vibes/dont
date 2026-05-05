## Context

The repo already has focused specs for verbs, lifecycle, envelope/error contracts, data shapes, and harness/help surfaces. What remains central but undecomposed is the rule layer: the shipped methodology rules, the severity model that changes behaviour between permissive and strict modes, and the command surface for listing, testing, adding, and explaining rules. These behaviours live mainly in §13, but they also connect to §10.5 errors and §11.4.1 rule authoring guidance.

## Goals
- Capture the methodology-as-rules model as a standalone capability independent of CLI verbs and storage substrate.
- Standardize declarative Datalog rules as violation queries returning specific schemas (`?entity_id, ?detail`).
- Establish strict severity boundaries (`warn` and `strict`) and un-deletable shipped core rules with a minimum `warn` severity.
- Capture the operator-facing rule CLI, syntax validation, and dry-run `test` surface as a separate capability.
- Preserve the absolute boundary between rule-layer failures (graph evaluation) and verb-level validators (input/syntax validation).

## Non-Goals
- Specify the full project layout or `config.toml` schema
- Specify import-generated rule translation details
- Re-specify command semantics already covered in core/lifecycle verb specs

## Decisions
- **Two capabilities, not one**: the rule engine changes independently from the CLI affordances for managing and explaining them.
- **Abstract Declarative Language**: The engine normatively requires a declarative graph-query language (abstractly a Datalog dialect), leaving specific engine syntax to the implementation.
- **Violation Queries**: All rules are violation queries returning a standard `?entity_id, ?detail` schema. A non-empty result set means the rule failed. Project-wide violations MUST bind `?entity_id` to the reserved `project:` entity.
- **Severity Boundaries and Static Shipped Rules**: Exactly two severities exist (`warn`, `strict`). Shipped rules have a minimum severity of `warn` and default configurations. To prevent accidentally bricking the system, shipped core rules and their Markdown translations MUST be statically bundled/embedded in the binary and cannot be deleted from the filesystem.
- **Verb-level validators stay out of the rule engine**: Basic command validation (e.g. `reason-required`, `evidence-required`, `reason-not-hedge`) remains hardcoded in the CLI. Datalog rules strictly evaluate the epistemic lattice graph state and CANNOT override core data-model invariants (like the atom-completion gate).
- **Sibling translation document requirement**: The engine MUST refuse to load any rule lacking a structured (e.g. `# Rationale`, `# Remediation`) English translation document.
- **`dont explain` belongs with rule CLI**: It statically reads and formats the structured sibling document; it does not execute the rule.
- **Syntax validation and Dry-runs**: `dont rules test` operates as a dry-run against a temporary snapshot. Commands like `add` or `test` MUST validate syntax immediately and emit specific error envelopes (e.g., `code: "rule-syntax-error"`).
- **Storage layout deferred**: The rule specs define behavior by rule name, deferring filesystem specifics (like `.dont/rules/`) to the project layout spec.

## Source Mapping
- `dont-rule-engine`: §13 default rules and severity table; §10.5 `rule_name`, `rule-not-met`, and warning semantics; v0.3 notes on `vague-reason`
- `dont-rule-cli`: §10 command summaries for `rules` and `explain`; §11.4.1 workflow for authoring rule `.dl` + `.md`, testing, and severity assignment

## Risks / Trade-offs
- The rule engine spec could drift into implementation details about Datalog evaluation.
  - Mitigation: keep requirements at the contract level: abstract Datalog dialect, violation queries, severity outcomes, and override boundaries.
- The structured sibling document requirement creates friction for rule authors.
  - Mitigation: the forcing function for explanation is central to the project's goals. Strict validation of `# Rationale` and `# Remediation` prevents opaque rule failures.
- Rule definitions depend on data-model details.
  - Mitigation: reference `dont-data-model` rather than restating relation definitions.

## Open Questions
- None remain for the high-level boundary of these two capabilities; boundaries and semantics resolved by design interview.
