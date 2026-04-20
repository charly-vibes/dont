## ADDED Requirements

### Requirement: Lock promotes verified claims to a terminal state
The system SHALL provide a `lock` operation that promotes a verified claim to the `locked` state only when the claim is in the `verified` state, has at least three assessed hypotheses, and has at least two independent supporting evidence items.

#### Scenario: lock succeeds for an eligible verified claim
- **WHEN** an actor invokes `lock` on a verified claim with at least three assessed hypotheses and at least two independent supporting evidence items
- **THEN** the claim transitions to `locked`

#### Scenario: lock refuses claims that do not satisfy the gate
- **WHEN** an actor invokes `lock` on a claim that lacks the required assessed hypotheses or independent supporting evidence items
- **THEN** the command is refused

#### Scenario: lock refuses non-verified claims
- **WHEN** an actor invokes `lock` on a claim that is not in the `verified` state
- **THEN** the command is refused

#### Scenario: lock refuses already-locked claims
- **WHEN** an actor invokes `lock` on a claim that is already `locked`
- **THEN** the command is refused

#### Scenario: lock refuses terms
- **WHEN** an actor invokes `lock` on a term
- **THEN** the command is refused because term locking is not supported for non-seed terms in this version

### Requirement: Reopen bypasses stale-cascade manually
The system SHALL provide a `reopen` operation that moves a stale claim or term to `unverified` so it can be reconsidered on its own merits.

#### Scenario: reopen moves stale entity to unverified
- **WHEN** an actor invokes `reopen` on a stale entity
- **THEN** the entity transitions to `unverified`

#### Scenario: reopen refuses non-stale entities
- **WHEN** an actor invokes `reopen` on an entity that is not stale
- **THEN** the command is refused

### Requirement: Ignore moves entities to a terminal escape state
The system SHALL provide an `ignore` operation that moves a claim or term to the `ignored` state and SHALL require a non-empty, non-hedge-only reason for doing so.

#### Scenario: ignore moves eligible entity to ignored
- **WHEN** an actor invokes `ignore` on a claim or term with a non-empty, non-hedge-only reason
- **THEN** the entity transitions to `ignored`

#### Scenario: ignore requires a reason
- **WHEN** an actor invokes `ignore` without a reason
- **THEN** the command is refused

#### Scenario: ignore refuses hedge-only reasons
- **WHEN** an actor invokes `ignore` with a reason that contains only hedge language and no specific defect or justification
- **THEN** the command is refused

### Requirement: Verify-evidence checks liveness without changing status
The system SHALL provide a `verify-evidence` operation that checks the liveness of evidence references attached to a claim or term, records per-reference outcome details and warnings, and does not change the entity's status.

#### Scenario: verify-evidence records per-reference liveness results
- **WHEN** an actor invokes `verify-evidence` for a claim or term with attached evidence references
- **THEN** the tool records a per-reference outcome for those references
- **AND** includes any warning details associated with failed or degraded checks
- **AND** does not change the claim or term status

#### Scenario: verify-evidence returns partial results on per-reference timeout
- **WHEN** one evidence reference times out while others can still be checked
- **THEN** the command returns partial liveness results rather than aborting the whole verification run

#### Scenario: verify-evidence warns on stale or malformed evidence references
- **WHEN** the command encounters malformed, timed-out, or failing evidence references
- **THEN** the results include warnings describing those evidence problems

#### Scenario: verify-evidence fails structurally when no evidence can be checked
- **WHEN** an actor invokes `verify-evidence` for a target that has no attached evidence references
- **THEN** the command fails structurally rather than reporting a successful verification run

#### Scenario: verify-evidence fails structurally for an unknown target
- **WHEN** an actor invokes `verify-evidence` for a target that does not exist
- **THEN** the command fails structurally rather than recording liveness results

### Requirement: Verify-evidence is separate from dismiss
The system SHALL keep evidence liveness verification separate from `dismiss` so that dismissal remains deterministic and network-independent, and SHALL apply bounded network politeness measures when checking remote evidence.

#### Scenario: dismiss does not require live network verification
- **WHEN** an actor invokes `dismiss` with well-formed evidence references
- **THEN** dismissal behavior does not depend on live network checks performed during that command

#### Scenario: verify-evidence handles network-sensitive checks
- **WHEN** a project wants to assess whether evidence references are still reachable
- **THEN** it uses `verify-evidence` rather than changing the dismissal contract

#### Scenario: verify-evidence uses bounded polite network behavior
- **WHEN** `verify-evidence` checks remote evidence references
- **THEN** it uses bounded concurrency or retry behavior that avoids unbounded request flooding against cited hosts
