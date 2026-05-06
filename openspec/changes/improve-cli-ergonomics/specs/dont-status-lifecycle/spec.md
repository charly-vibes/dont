# dont-status-lifecycle Deltas

## MODIFIED Requirements

### Requirement: Status transition terminology
The system SHALL use the terms `doubt` and `verify` to describe the primary lifecycle transitions in v0.3. Documentation and error messages SHALL reflect this terminology while acknowledging deprecated aliases.

#### Scenario: doubt transition records its reason
- **WHEN** an actor transitions an entity to `doubted` using the `doubt` command
- **THEN** the transition record includes the stated reason for doubt

#### Scenario: verify transition records evidence references
- **WHEN** an actor transitions an entity to `verified` using the `verify` command
- **THEN** the transition record includes the evidence references used for that verification
