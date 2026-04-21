# Research basis

The documentation site is grounded in two main source categories in this repository:

- the draft specification’s purpose and references sections
- the tracked research artifacts under `.wai/projects/dont/research/`, especially the long-form syntheses on why LLMs defend prior outputs and how institutions force better verification

## Main conclusions carried into the tool design

### 1. In-context self-correction is weak

The research summary argues that models often rationalize earlier answers instead of reliably inspecting them. Asking the same model, in the same context, to "be more critical" often produces better-sounding prose rather than better epistemic behavior.

This motivates `dont`’s refusal-oriented design: the system should block or route unsupported assertions instead of merely asking for a nicer explanation.

### 2. External signals are stronger than introspection

The strongest correction patterns in the cited literature depend on something outside the initial generation stream:

- retrieved evidence
- tests or executable checks
- trained verifiers
- independent critics or clean-context retries

This is why the spec emphasizes evidence, explicit remediation, and spawn-style verification requests instead of plain self-reflection.

### 3. Institutions solve this with structure

The research notes compare LLM verification failures to long-studied human failures in science, law, medicine, and engineering. Across those domains, reliability improves when systems enforce:

- separation between generator and evaluator
- explicit burden of proof
- auditable state transitions
- procedures for challenge, review, and lock-in

`dont` translates that pattern into a CLI protocol for agents.

## How this maps into the spec draft

The current draft spec turns those conclusions into concrete design choices:

- a forcing-function CLI with a narrow role
- append-only history instead of silent rewrite
- explicit claim and term entities
- status transitions rather than implicit confidence
- refusal messages that tell the agent what to do next
- clean-context verification hooks when independent checking matters

## Traceability

For deeper reading, start here:

- [Draft specification (`dont-spec-v0_3_2.md`)](https://github.com/charly-vibes/dont/blob/main/dont-spec-v0_3_2.md) — especially section 1 (Purpose) and section 20 (References and learning material)
- [Research: forcing doubt in minds and machines](https://github.com/charly-vibes/dont/blob/main/.wai/projects/dont/research/2026-04-20-forcing-doubt-in-minds-and-machines-why-llms-de.md)
- [Research: designing doubt into AI interaction](https://github.com/charly-vibes/dont/blob/main/.wai/projects/dont/research/2026-04-20-designing-doubt-into-ai-interaction-what-the.md)

## Limits of this page

This page is a synthesis, not the full literature review. It is meant to explain why the project exists, not to replace the source documents.
