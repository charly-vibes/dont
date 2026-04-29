## 1. Extend the decomposition
- [x] 1.1 Add `dont-init-modes`: initialization semantics (`project:` entity), authored seed vocabulary (`tool:dont-init`), mode overrides (`permissive/strict`), and `DONT_HARNESS=1` (JSON force).
- [x] 1.2 Add `dont-lifecycle-verbs`: `lock` (claims only, no atom constraint), `reopen` (reversing terminal states only), `ignore` (requires reason), and `verify-evidence` (trace analysis without status mutation).

## 2. Preserve boundaries
- [x] 2.1 Keep initialization and mode semantics separate from core CLI verb specs while maintaining append-only and immutable invariants.
- [x] 2.2 Keep lifecycle-adjacent verbs separate from the four primary epistemic verbs and explicitly constrain their operational domains.

## 3. Validate
- [ ] 3.1 Run `openspec validate add-dont-operational-specs --strict`
