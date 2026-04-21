## ADDED Requirements

### Requirement: Spawn-producing derived commands
The system SHALL treat `dont guess`, `dont assume`, and `dont overlook` as spawn-producing derived commands. In harness mode they MUST emit `spawn_request` envelopes instead of calling an LLM directly. In direct mode they MUST call the configured provider and use the same command intent: `guess` requests diverse candidate answers, `assume` requests independent verification of one claim, and `overlook` requests a premortem-style adversarial challenge. `dont guess` MUST accept candidate-count and temperature shaping inputs so callers can request multiple diverse candidates.

#### Scenario: assume emits spawn request in harness mode
- **WHEN** the caller runs `dont assume claim:01HX05A9K8VP --json` in harness mode
- **THEN** the command returns `envelope_kind: "spawn_request"`
- **AND** it does not call an LLM provider directly

#### Scenario: overlook emits adversarial spawn request
- **WHEN** the caller runs `dont overlook claim:01HX05A9K8VP --json` in harness mode
- **THEN** the spawn request instructs a subagent to assume the claim is wrong and try to explain why

#### Scenario: guess requests candidate set
- **WHEN** the caller runs `dont guess "<question>" --json`
- **THEN** the command produces a spawn request for multiple candidate answers rather than writing to the claim store directly

#### Scenario: guess carries candidate count and temperature
- **WHEN** the caller runs `dont guess "<question>" --n 3 --temperature 0.7 --json`
- **THEN** the resulting direct call or spawn request preserves the requested candidate count and temperature settings

#### Scenario: direct mode bypasses spawn envelope
- **WHEN** direct mode is active for `guess`, `assume`, or `overlook`
- **THEN** the tool calls the configured provider directly instead of emitting a harness-facing `spawn_request`

### Requirement: Harness-agnostic spawn contract
The system SHALL keep the spawn protocol transport-agnostic. A harness MUST be able to receive the `spawn_request` through direct CLI stdout, an MCP tool return, or another structured transport without changing the payload contract. Spawn requests MUST carry clean-context subagent instructions and restricted tool permissions, and the only terminal callback actions for verification spawns MUST be `dont dismiss` or `dont trust`.

#### Scenario: direct CLI harness consumes stdout envelope
- **WHEN** a harness runs `dont assume ... --json` through the CLI
- **THEN** it can detect `envelope_kind: "spawn_request"` on stdout and start its own subagent mechanism from that data

#### Scenario: subagent tool permissions are restricted
- **WHEN** the harness fulfils an `assume` or `overlook` request
- **THEN** it starts the subagent with the exact allowed-tool list from the request context
- **AND** the subagent's terminal action is limited to `dont dismiss` or `dont trust`

### Requirement: Harness-mode detection order
The system SHALL resolve harness mode in the documented priority order. `--direct` MUST force direct mode. Otherwise a truthy `DONT_HARNESS` environment value MUST force harness mode, invocation through `dont mcp` MUST force harness mode, non-terminal stdout MUST imply harness mode, and interactive terminal invocation without those signals MUST fall back to direct mode.

#### Scenario: direct flag overrides environment
- **WHEN** `DONT_HARNESS=1` is set and the caller passes `--direct`
- **THEN** direct mode is selected

#### Scenario: environment variable selects harness mode
- **WHEN** `DONT_HARNESS` is set to a non-empty value other than `0` or `false`
- **THEN** harness mode is selected even if stdout is a terminal

#### Scenario: piped stdout implies harness mode
- **WHEN** stdout is redirected or piped and no stronger signal applies
- **THEN** harness mode is selected

#### Scenario: interactive shell defaults to direct mode
- **WHEN** the caller runs a spawn-producing command in an interactive terminal without harness signals
- **THEN** direct mode is selected

### Requirement: Spawn expiry and timeout recovery
The system SHALL assign every spawn request an `expires_at` timestamp derived from the configured spawn-timeout window. On subsequent invocations it MUST sweep pending spawns for expiry, emit `spawn_timeout` events for expired requests, restore the entity to its pre-spawn status, and avoid automatic re-spawn.

#### Scenario: expired spawn records timeout event
- **WHEN** a pending spawn passes its `expires_at` time without a terminal callback
- **THEN** the next `dont` invocation emits a `spawn_timeout` event with the timeout timestamps against the original entity

#### Scenario: timeout restores pre-spawn status
- **WHEN** a spawn expires
- **THEN** the entity is restored to the status it had when the spawn was issued
- **AND** an entity that was already `unverified` remains `unverified`

#### Scenario: timeout does not auto-respawn
- **WHEN** a spawn times out
- **THEN** the tool does not retry the verification automatically
- **AND** a caller must re-run `dont assume` or `dont overlook` explicitly to request another verification round

### Requirement: Spawn listing queries
The system SHALL provide `dont spawns` to inspect spawn state. `dont spawns --pending` MUST list active non-expired requests sorted by age, `dont spawns --timed-out` MUST list expired requests that have not been re-assumed since, and `dont spawns --all` MUST return the full history.

#### Scenario: pending view excludes expired requests
- **WHEN** the caller runs `dont spawns --pending --json`
- **THEN** the response contains only non-expired pending spawn requests sorted by age

#### Scenario: timed-out view isolates unresolved expiries
- **WHEN** the caller runs `dont spawns --timed-out --json`
- **THEN** the response contains expired spawn requests that remain relevant because the original entity has not been re-assumed since timeout

### Requirement: Late callbacks and spawn errors
The system SHALL use `spawn-not-found` when a callback references an unknown request ID and `spawn-expired` when a terminal callback arrives after expiry. On `spawn-expired`, the callback SHALL still be applied to the entity's terminal action while the lateness is recorded as a warning, preventing clock-skew or transport delay from discarding completed verification work.

#### Scenario: unknown request id refuses callback
- **WHEN** a callback references a request ID the store does not know
- **THEN** the command returns an error with code `spawn-not-found`

#### Scenario: expired callback still applies
- **WHEN** a terminal callback arrives after `expires_at`
- **THEN** the command records a warning for `spawn-expired`
- **AND** it still applies the dismiss-or-trust action to the entity
