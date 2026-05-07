## Context

The current tracer-bullet experience shows that `file://.../README.md` is enough to prove the envelope path, but not enough to create high-trust repository-grounded memory. Scientific/software architecture workflows need evidence that can answer: *what exact source location justified this claim?*

## Goals / Non-Goals

- Goals:
  - support precise repository-grounded evidence
  - preserve human-auditable excerpts
  - support later drift detection without requiring a full content-addressed archive
- Non-Goals:
  - full provenance/version-control system integration
  - semantic code understanding
  - replacing `dont verify-evidence` with code-aware proof

## Decisions

- Decision: evidence locators are repository-relative first, URI-only second
  - Why: the main adoption target is sidecar grounding of repository facts
  - Clarification: plain URI-only evidence remains supported as a first-class compatibility path, but repository-relative locators are the recommended form for project-grounded claims
- Decision: excerpt capture is optional but first-class
  - Why: some evidence can be summarized structurally, while doc-based evidence benefits from visible quotes
- Decision: fingerprints are lightweight stability aids, not trust roots
  - Why: the immediate need is drift detection, not tamper-proof archival

## Risks / Trade-offs

- More structure raises command complexity
  - Mitigation: keep a shorthand path for simple URI evidence and document structured forms as the recommended mode
- Fingerprints may create a false sense of proof
  - Mitigation: specify them as drift aids only, not cryptographic guarantees of truth

## Open Questions

- Should repository locators allow byte offsets in addition to line spans?
- Should excerpt capture be stored exactly as supplied, or normalized for whitespace/newlines?
- Should code and prose evidence share one shape or have different `kind` defaults?
