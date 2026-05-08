# dont-lifecycle-verbs Specification

## Purpose
TBD - created by archiving change add-dont-operational-specs. Update Purpose after archive.
## Requirements
### Requirement: Lock promotes verified claims to a terminal state
The system SHALL provide a `lock` operation that promotes a verified claim to the `locked` state only when the claim is in the `verified` state, has at least three assessed hypotheses, has at least two independent supporting evidence items, and has no derived assessment that compromises dependency integrity (`stale`, `compromised-support`, `dangling-dependency`, or `unresolved-term`).

#### Scenario: lock succeeds for an eligible verified claim
- **WHEN** an actor invokes `lock` on a verified claim with at least three assessed hypotheses, at least two independent supporting evidence items, and an empty dependency-integrity `derived_assessments[]`
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

#### Scenario: lock refuses claims with stale support
- **WHEN** an actor invokes `lock` on a verified claim whose `derived_assessments[]` contains `stale`
- **THEN** the command is refused until the stale dependency trace is cleared

#### Scenario: lock refuses terms
- **WHEN** an actor invokes `lock` on a term
- **THEN** the command is refused because term locking is not supported for non-seed terms in this version

### Requirement: Undoubt retracts explicit doubt
The system SHALL provide an `undoubt` operation that moves a `doubted` claim or term back to `unverified`, allowing the doubt to be retracted when it was registered in error or superseded. `undoubt` SHALL only target entities in the `doubted` state and SHALL refuse all other persisted statuses. Use `reopen` for `ignored` entities — `undoubt` and `reopen` are distinct recovery operations for distinct closure states.

#### Scenario: undoubt moves doubted entity to unverified
- **WHEN** an actor invokes `undoubt` on an entity whose persisted status is `doubted`
- **THEN** the entity transitions to `unverified`

#### Scenario: undoubt refuses non-doubted entities
- **WHEN** an actor invokes `undoubt` on an entity whose persisted status is `unverified`, `verified`, `ignored`, or `locked`
- **THEN** the command is refused

### Requirement: Reopen restores ignored entities
The system SHALL provide a `reopen` operation that moves an `ignored` claim or term to `unverified` so it can be reconsidered on its own merits. `reopen` SHALL NOT target `stale` or any other derived assessment, because derived assessments are computed trace results rather than persisted lifecycle states. `reopen` SHALL NOT unlock `locked` entities in v0.3.

#### Scenario: reopen moves ignored entity to unverified
- **WHEN** an actor invokes `reopen` on an entity whose persisted status is `ignored`
- **THEN** the entity transitions to `unverified`

#### Scenario: reopen refuses derived-stale entity
- **WHEN** an actor invokes `reopen` on an entity whose persisted status is `verified` and whose `derived_assessments[]` contains `stale`
- **THEN** the command is refused because `stale` is not a persisted lifecycle state

#### Scenario: reopen refuses non-ignored entities
- **WHEN** an actor invokes `reopen` on an entity whose persisted status is `unverified`, `verified`, or `doubted`
- **THEN** the command is refused

#### Scenario: reopen refuses locked entities
- **WHEN** an actor invokes `reopen` on an entity whose persisted status is `locked`
- **THEN** the command is refused because locked entities are not reopenable in v0.3

### Requirement: Ignore moves entities to a terminal escape state
The system SHALL provide an `ignore` operation that moves a claim or term whose persisted status is `unverified`, `verified`, or `doubted` to the `ignored` state and SHALL require a non-empty, non-hedge-only reason for doing so. `ignore` SHALL refuse entities that are already `ignored` or `locked`.

#### Scenario: ignore moves eligible entity to ignored
- **WHEN** an actor invokes `ignore` on a claim or term with a non-empty, non-hedge-only reason
- **THEN** the entity transitions to `ignored`

#### Scenario: ignore requires a reason
- **WHEN** an actor invokes `ignore` without a reason
- **THEN** the command is refused

#### Scenario: ignore refuses hedge-only reasons
- **WHEN** an actor invokes `ignore` with a reason that contains only hedge language and no specific defect or justification
- **THEN** the command is refused

#### Scenario: ignore refuses locked entities
- **WHEN** an actor invokes `ignore` on a locked entity
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

### Requirement: Verify-evidence is separate from flag
The system SHALL keep evidence liveness verification separate from `flag` so that the `flag` operation remains deterministic and network-independent, and SHALL apply bounded network politeness measures when checking remote evidence.

#### Scenario: flag does not require live network verification
- **WHEN** an actor invokes `flag` with well-formed evidence references
- **THEN** `flag` behavior does not depend on live network checks performed during that command

#### Scenario: verify-evidence handles network-sensitive checks
- **WHEN** a project wants to assess whether evidence references are still reachable
- **THEN** it uses `verify-evidence` rather than changing the `flag` contract

#### Scenario: verify-evidence uses bounded polite network behavior
- **WHEN** `verify-evidence` checks remote evidence references
- **THEN** it uses bounded concurrency or retry behavior that avoids unbounded request flooding against cited hosts

### Requirement: trust registers explicit doubt
The system SHALL provide `trust` to register explicit doubt about a claim or term. Skepticism recorded via `trust` SHALL transition an entity from `unverified` or `verified` to `doubted`.

#### Scenario: trust registers doubt with reason
- **WHEN** `dont trust claim:X --reason "..."` is invoked
- **THEN** the claim transitions to `doubted` and the reason is recorded in the event log

### Requirement: flag (formerly dismiss) adds evidence
The system SHALL provide `flag` (formerly `dismiss`) to add evidence to an entity and transition it to `verified`. The `dismiss` verb remains supported as an alias for `flag` in v0.3 but is deprecated.

#### Scenario: flag transitions to verified with evidence
- **WHEN** `dont flag claim:X --evidence <uri>` is invoked
- **THEN** the claim transitions to `verified` and the evidence is recorded

#### Scenario: flag on doubted entity clears doubt
- **WHEN** `dont flag claim:X --evidence <uri>` is invoked on a doubted claim
- **THEN** the claim transitions to `verified` and doubt is cleared

#### Scenario: deprecated alias emits warning
- **WHEN** `dont dismiss` is invoked
- **THEN** a deprecation warning is emitted to stderr suggesting `flag` instead
- **AND** the command proceeds normally
- **AND** the deprecation warning goes to stderr regardless of whether `--json` is set
