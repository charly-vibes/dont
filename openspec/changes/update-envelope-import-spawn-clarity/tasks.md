## 1. Envelope conformance (TDD)
- [ ] 1.1 RED: add conformance scenarios for success `hints` presence, parser compatibility fallback, and `meta.tx` null/range rules
- [ ] 1.2 GREEN: update `dont-envelope` delta requirements/scenarios until the conformance scenarios pass
- [ ] 1.3 REFACTOR: remove redundant wording and keep producer-vs-parser obligations explicit

## 2. Import identity and network safety (TDD)
- [ ] 2.1 RED: add scenarios for canonical source identity normalization (including SPARQL normalization and path-alias dedup)
- [ ] 2.2 GREEN: update `dont-import-surface` idempotence requirement with deterministic canonicalization rules
- [ ] 2.3 RED: add safety scenarios for blocked destination classes, mixed DNS answers, and refusal-code mapping
- [ ] 2.4 GREEN: update URL scheme/destination policy and deterministic refusal-code mapping
- [ ] 2.5 REFACTOR: tighten wording so policy checks, taxonomy, and examples are structurally separated

## 3. Spawn race determinism (TDD)
- [ ] 3.1 RED: add concurrent timeout/callback and duplicate-callback race scenarios
- [ ] 3.2 GREEN: update `dont-spawn-protocol` with terminal resolver definitions and single-mutation invariant
- [ ] 3.3 REFACTOR: simplify race wording and remove event-persistence ambiguity

## 4. Validation
- [ ] 4.1 Run `openspec validate update-envelope-import-spawn-clarity --strict`
- [ ] 4.2 Run `openspec validate --changes --strict`
