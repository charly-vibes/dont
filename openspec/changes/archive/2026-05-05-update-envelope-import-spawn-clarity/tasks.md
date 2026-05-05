## 1. Envelope conformance (TDD)
- [x] 1.1 RED: add conformance scenarios for success `hints` presence (strictly required array), parser compatibility fallback, and `meta.tx` null/range rules (`[1, 2^53-1]`)
- [x] 1.2 GREEN: update `dont-envelope` delta requirements/scenarios until the conformance scenarios pass
- [x] 1.3 REFACTOR: remove redundant wording and keep producer-vs-parser obligations explicit

## 2. Import identity and network safety (TDD)
- [x] 2.1 RED: add scenarios for canonical source identity normalization (including SPARQL normalization and SHA-256 path-alias dedup)
- [x] 2.2 GREEN: update `dont-import-surface` idempotence requirement with deterministic canonicalization rules
- [x] 2.3 RED: add safety scenarios for blocked destination classes (loopback/private), mixed DNS answers, and refusal-code mapping
- [x] 2.4 GREEN: update URL scheme/destination policy and deterministic refusal-code mapping (`unresolvable-uri`)
- [x] 2.5 REFACTOR: tighten wording so policy checks, taxonomy, and examples are structurally separated

## 3. Spawn race determinism (TDD)
- [x] 3.1 RED: add concurrent timeout/callback, late timeout sweeps, and duplicate-callback race scenarios (`spawn-already-resolved`)
- [x] 3.2 GREEN: update `dont-spawn-protocol` with terminal resolver definitions, accepting late callbacks with warnings (`spawn-expired`), and ignoring late timeouts
- [x] 3.3 REFACTOR: simplify race wording and remove event-persistence ambiguity

## 4. Validation
- [x] 4.1 Run `openspec validate update-envelope-import-spawn-clarity --strict`
- [x] 4.2 Run `openspec validate --changes --strict`
