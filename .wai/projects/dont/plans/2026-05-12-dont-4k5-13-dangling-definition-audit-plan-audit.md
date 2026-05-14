---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-13-dangling-definition-detection-audit, pipeline-step:plan]
---

dont-4k5.13 dangling definition audit plan

Audit scope: dangling_definition rule in src/rules/dangling_definition.rs

Findings from research:
- Rule detects 'referenced but never defined' correctly (term:uuid dep that resolves to None)
- Rule does NOT detect 'defined but never referenced' — no reverse-index scan
- Circular references: structurally impossible in current data model (claims depend on terms, terms have no depends_on field)
- Comment-only references: not applicable (rule only checks structured depends_on array)

Test strategy for 5 ticket cases:
1. defined_with_zero_references_not_detected — term exists but no claim references it → expect ZERO violations (documents current limitation; create gap ticket)
2. reference_with_no_definition_fires — claim depends on term:nonexistent → violation fired (already covered, add explicit test)
3. definition_referenced_exactly_once_is_silent — term exists, one claim references it → no violation
4. comment_reference_is_not_structural — (NOT APPLICABLE; add doc test comment only)
5. circular_reference_impossible — (NOT APPLICABLE; data model prevents it; add doc comment)

Pass criteria outcome:
- Cases 2, 3: pass
- Cases 4, 5: not applicable by design
- Case 1: current rule doesn't detect this direction — document gap, create follow-up ticket

Deliverables:
- 3 new unit tests (cases 1, 2, 3)
- Follow-up ticket for 'detect unused term definitions' feature
- No production code changes (correct behavior for its spec scope)
