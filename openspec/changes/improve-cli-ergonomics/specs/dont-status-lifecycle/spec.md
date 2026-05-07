# dont-status-lifecycle Deltas

## MODIFIED Requirements

### Requirement: Status transition terminology
The system SHALL use the terms `trust`, `flag`, and `undoubt` to describe the primary lifecycle transitions in v0.3. Documentation and error messages SHALL reflect this terminology while acknowledging deprecated aliases.

#### Scenario: trust transition records its reason
- **WHEN** an actor transitions an entity to `doubted` using the `trust` command
- **THEN** the transition record includes the stated reason for doubt

#### Scenario: flag transition records evidence references
- **WHEN** an actor transitions an entity to `verified` using the `flag` command
- **THEN** the transition record includes the evidence references used for that verification

#### Scenario: undoubt transition reverts doubt without evidence
- **WHEN** an actor transitions an entity from `doubted` to `unverified` using the `undoubt` command
- **THEN** the transition record captures the retraction without requiring evidence
