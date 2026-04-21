# Testability & Implementability Evaluation: Tracer Bullet

Evaluation Summary:
```
Specification: openspec/changes/implement-tracer-bullet/design.md vs Core Capability Specs
Requirements Evaluated: Tracer Bullet Design + 4 Core Capability Specs (CLI, Status, Data Model, Envelope)
System Context Referenced: yes (Rust + CozoDB embedded)

Axis Scores:
  Implementability: CONDITIONAL
  Testability:      TESTABLE

Definition of Ready: READY_WITH_SPIKES
```

## Findings

**[IMPL-2.1] [HIGH] — Data Model vs Datom Mandate**
  **Axis:** Implementability
  **Framework:** PIECES / Information
  **Issue:** The tracer-bullet design proposes a simplified schema (`claims` and `events` tables). However, `dont-data-model/spec.md` normatively mandates: "The system SHALL store all facts as datoms — immutable atomic facts in the shape `(entity, attr, value, tx, assert_bit)`." While the tracer-bullet design documents this as an accepted trade-off ("tracer store is throwaway"), writing code that intentionally violates a normative capability spec creates an immediate compliance failure.
  **Constraint:** The architecture must prove CozoDB datoms. Using standard relational tables in CozoDB proves the embedded DB works, but *does not* prove the datom query patterns.
  **Remediation:** Either (a) update the tracer-bullet design to use the actual `(entity, attr, value, tx, assert_bit)` datom schema from day one, or (b) amend the capability spec to allow a Phase 0 transitional schema. Given CozoDB's native support for datoms, attempting the actual datom schema is strongly recommended to truly "prove the architecture."

**[IMPL-3.1] [MEDIUM] — Envelope Meta Object Omission**
  **Axis:** Implementability
  **Framework:** GLIA / Executability
  **Issue:** The tracer-bullet design defines `meta: Option<Meta>` and states it will be "always None for tracer". However, `dont-envelope/spec.md` mandates: "The system SHALL include a `meta` object on every envelope carrying `duration_ms`, `tx`... and `request_id`". Omitting the `meta` object entirely violates the envelope contract.
  **Constraint:** Parsers expecting `meta.duration_ms` will crash on `null` meta.
  **Remediation:** Update the tracer-bullet `Envelope` struct to make `meta: Meta` (not `Option`), and populate it with `{ duration_ms: <actual_time>, tx: null, request_id: null }`. `tx` and `request_id` can be `Option` inside the struct, but the `meta` object itself must exist.

**[TEST-4.1] [LOW] — Lattice Subset Definition**
  **Axis:** Testability
  **Dimension:** Intrinsic / Controllability
  **Issue:** The tracer-bullet design explicitly subsets the status lattice to `Unverified`, `Verified`, `Doubted`. This correctly aligns with `dont-status-lifecycle/spec.md`'s core states. However, it defers `stale`, `locked`, and `ignored`.
  **Impact:** Test harnesses cannot verify the "locked and ignored are terminal" or "doubt stales dependent entities" requirements from the lifecycle spec.
  **Transformation:** Acceptable for a tracer bullet. Explicitly document in the test plan that terminal and stale states are out-of-scope for the Phase 1-10 tracer tests, and will be covered in a subsequent iteration.

## Confirmed Strengths
- **Refusal Protocol:** The hardcoded `reason-required` and `no-evidence` checks perfectly map to the `dont-errors` spec's distinction between "verb-level validators" (which have dedicated codes and `rule_name: null`) and rule-engine refusals. Highly implementable.
- **Exit Codes:** The 0-4 exit code contract is clearly defined and matches the error spec, providing excellent process-level testability.
- **Envelope Structure:** Aside from the `meta` omission, the JSON envelope structs perfectly align with the complex requirements of `dont-envelope` and `dont-errors` (especially the non-empty `remediation` invariant).

## Spike Recommendations

```
Spike: Datom Schema Feasibility
  Validates: [IMPL-2.1]
  Question: Does implementing the true `(entity, attr, value, tx, assert_bit)` schema in CozoDB for the tracer bullet add unacceptable overhead compared to the proposed simplified schema?
  Timebox: 2 hours (can be done during Phase 2: Storage Layer task)
  Success Criteria: A decision to either adopt the datom schema immediately, or formally amend the specs to allow the simplified schema temporarily.
```

## Verdict Rationale
The design is structurally very sound and highly testable, but it intentionally violates two normative requirements of the newly split capability specs (the datom schema mandate, and the `meta` envelope inclusion). Fixing the `meta` struct is trivial. The datom schema discrepancy should be addressed either by adopting datoms early (recommended) or amending the spec, leading to a `READY_WITH_SPIKES` verdict. Once resolved, the `dont-s21` scaffold task can proceed safely.
