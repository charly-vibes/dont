## ADDED Requirements
### Requirement: Help recommends trace when blocker labels are not enough
Operator-facing help and tutorials SHALL recommend `dont trace <entity-id>` as the next diagnostic step when a claim or term is blocked by dependency/support fallout and `show` or `why` alone do not explain the causal path clearly.

#### Scenario: blocked verification guidance points to trace
- **WHEN** the tutorial or a refusal-oriented how-to explains what to do after seeing a blocker such as `stale` or `unresolved-term`
- **THEN** it recommends `dont trace <entity-id>` as the path-oriented diagnostic command
