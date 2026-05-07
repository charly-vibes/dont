## ADDED Requirements
### Requirement: Ground captures a claim and its evidence in one command
The system SHALL provide `dont ground <statement>` as a derived orchestration command for the sidecar workflow where an operator already has both claim text and supporting evidence. In its initial version, `ground` SHALL accept statement text plus one or more evidence references, along with the standard invocation-level author override, and SHALL NOT introduce new convenience syntax for atoms, dependencies, or references beyond what the underlying core verbs already support. `ground` SHALL require at least one evidence reference and SHALL return a verified claim when all eligibility checks succeed.

#### Scenario: repository fact is grounded in one step
- **WHEN** an operator runs `dont ground "Chacana parses tensor expressions into a MathJSON-style AST." --evidence <locator> --json`
- **THEN** the command returns a verified claim entity without requiring a separate manual `conclude` followed by `dismiss`

#### Scenario: ground without evidence is refused
- **WHEN** an operator runs `dont ground "..."` without any evidence
- **THEN** the command is refused rather than silently creating an ungrounded claim

### Requirement: Ground preserves the underlying core-verb semantics
`ground` SHALL be defined as a convenience orchestration over the core workflow rather than as an alternate lifecycle path. Its successful execution MUST emit the existing `concluded` event followed by the existing `dismissed` event, in that order, and it MUST respect all existing refusal conditions that would apply to the corresponding `conclude` and `dismiss` operations.

#### Scenario: successful ground emits the normal logical sequence
- **WHEN** `dont ground ...` succeeds
- **THEN** the resulting audit history reflects both claim introduction and evidence-backed verification rather than a single magical state jump

#### Scenario: atom rules still apply under ground
- **WHEN** a future `ground` invocation supplies a claim shape that would violate atom-verification invariants
- **THEN** the command is refused under the same invariant rules that apply to the underlying lifecycle verbs

### Requirement: Ground avoids confusing partial side effects by default
If `ground` cannot complete its verification step, the default behaviour SHALL avoid leaving behind a new partially created claim unless the operator explicitly opts into retaining that partial state in a future extension. The default sidecar workflow is optimized for high-signal fact capture rather than for littering the ledger with failed half-attempts. To satisfy this, a `ground` invocation SHALL execute as one atomic command transaction: either both the underlying claim-creation and evidence-backed verification effects commit, or neither commits.

#### Scenario: failed evidence validation leaves no new claim by default
- **WHEN** an operator runs `dont ground "..." --evidence <bad-locator>` and the evidence is rejected before verification can complete
- **THEN** the command fails without creating a new lingering unverified claim by default

#### Scenario: duplicate-equivalent claim follows conclude duplicate policy
- **WHEN** an operator runs `dont ground` for a statement that is equivalent to an existing claim under the project's deduplication rules
- **THEN** the command is refused under the same duplicate-claim policy that would apply to the underlying `conclude` operation
