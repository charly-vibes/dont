---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-14-claim-structure-rule-edge-cases, pipeline-step:review]
review: "2 HIGH findings addressed: fragile contains() assertion replaced with exact string match; all len() checks have failure messages. 1 MEDIUM addressed: very_long_claim_body now asserts violation types. Integration patterns consistent."
---

ro5 review findings for dont-4k5.14 rule_claim_structure edge case tests

HIGH: whitespace_only test assertion matches[0].detail.contains('[TRIGGER]') is fragile — if violations reorder, assertion still passes. Fix: assert specific detail string.
HIGH: Missing assertion messages on len() checks — cryptic failures. Add failure messages.
MEDIUM: very_long_claim_body_does_not_panic — only checks count=2, not which violations. Add type assertions.
MEDIUM: unicode/prose tests lack assertion messages for debugging.
LOW: prose_use_of_trigger_text test name could clarify it's documenting a known limitation.
INTEGRATION: New tests correctly follow TempDir→store→append→check pattern. Minor: add failure messages for consistency with codebase style.
