## MODIFIED Requirements

### Requirement: Late callbacks and spawn errors
The system SHALL use `spawn-not-found` when a callback references an unknown request ID and `spawn-expired` when a terminal callback arrives after expiry. Terminal resolver outcomes are `timeout-restored` (sweep restored pre-spawn status) and `callback-applied` (dismiss-or-trust callback applied). On `spawn-expired`, a late callback SHALL still be applied when no terminal resolver outcome has been committed for that `request_id`, while lateness is recorded as a warning. A late callback MUST NOT apply if the same request has already reached a committed terminal resolver outcome.

#### Scenario: unknown request id refuses callback
- **WHEN** a callback references a request ID the store does not know
- **THEN** the command returns an error with code `spawn-not-found`

#### Scenario: expired callback still applies when unresolved
- **WHEN** a terminal callback arrives after `expires_at`
- **AND** no terminal resolver outcome has been committed yet
- **THEN** the command records a warning for `spawn-expired`
- **AND** it still applies the dismiss-or-trust action to the entity

#### Scenario: duplicate terminal callback does not re-apply transition
- **WHEN** a request has already reached a terminal resolver outcome
- **THEN** a later callback for the same request does not apply a second terminal transition
- **AND** the envelope includes a warning describing duplicate resolution

## ADDED Requirements

### Requirement: Deterministic timeout-versus-callback race resolution
The system SHALL resolve races between spawn timeout sweep and terminal callback using transaction commit order. For a single `request_id`, exactly one terminal resolver (timeout restore or callback action) may mutate entity status. Any later competing resolver MUST be surfaced as an envelope warning and MUST NOT perform a second status mutation.

#### Scenario: timeout and callback race yields one status mutation
- **WHEN** a timeout sweep and terminal callback for the same request execute concurrently
- **THEN** only the first committed resolver mutates entity status
- **AND** the later resolver is surfaced as a warning with no additional status mutation

#### Scenario: callback wins if committed first
- **WHEN** a terminal callback transaction commits before timeout sweep commits
- **THEN** callback semantics apply and timeout sweep does not restore pre-spawn status for that request

#### Scenario: timeout wins if committed first
- **WHEN** timeout sweep commits before terminal callback commits
- **THEN** timeout restoration applies and the later callback is treated as late/no-op with warning

#### Scenario: concurrent duplicate callbacks resolve once
- **WHEN** two terminal callbacks for the same request race concurrently
- **THEN** exactly one callback produces `callback-applied`
- **AND** the other callback is surfaced as duplicate/no-op warning
