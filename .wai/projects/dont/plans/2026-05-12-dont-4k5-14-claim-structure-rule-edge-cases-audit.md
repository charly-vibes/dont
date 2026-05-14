---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-14-claim-structure-rule-edge-cases, pipeline-step:plan]
---

dont-4k5.14 claim structure rule edge cases

Audit scope: rule_claim_structure.rs — validates tagged claims have [TRIGGER] and [CONFIG]/[MODE] slots.

Key finding: claims are plain text strings, not structured. Most ticket edge cases (nesting, duplicate keys) are N/A. Real gaps:

Test cases to write:
1. whitespace_only_statement_on_tagged_claim_fires_for_missing_slots — whitespace body, rule must flag missing [TRIGGER] and [CONFIG]/[MODE]
2. unicode_in_claim_body_does_not_confuse_slot_detection — Unicode body with valid slots should pass; Unicode body without slots should fire
3. claim_with_stale_tag_reference_is_silently_skipped — tag term id set, claim depends_on a term that no longer exists -> rule does not panic
4. tagged_claim_with_only_trigger_no_config_mode_fires — only [TRIGGER] present, no [CONFIG] or [MODE]
5. prose_containing_bracket_text_is_not_a_false_negative — body has '[TRIGGER] is a useful concept' as prose, not as the slot

Production code change expected: none — verify audit.
Panic risk: low — rule uses safe iterator operations.
