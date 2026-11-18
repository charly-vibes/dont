## MODIFIED Requirements

> **Change:** Extends the canonical `envelope_kind` enum to include new values introduced by the
> `add-eval-readiness` change.

### Requirement: Typed payload discriminator

The system SHALL include an `envelope_kind` field that discriminates the shape of the `data` field,
and SHALL define a canonical set of `envelope_kind` values for envelope version 0.2. As of this
change the canonical set additionally includes:

- `"stats"` — payload is a `StatsView` document as defined in `dont-analytics`
- `"eval_export"` — payload is an `EvalExport` document as defined in `dont-eval-export`

All existing parsers that implement the forward-compatible unknown-kind default branch required by
the base `dont-envelope` spec will handle these new values without modification.

#### Scenario: envelope_kind matches the data payload type

- **WHEN** a command returns a claim payload
- **THEN** `envelope_kind` is `"claim"` and `data` contains the claim payload shape

#### Scenario: canonical envelope_kind values are enumerated

- **WHEN** envelope version is 0.2
- **THEN** the canonical `envelope_kind` values include at minimum: `claim`, `claims`, `term`, `term_list`, `event`, `events`, `spawn_request`, `spawn_requests`, `rule`, `rule_result`, `prime`, `why`, `doctor`, `examples`, `schema_doc`, `version`, `empty`, `error`, plus `stats` and `eval_export`

#### Scenario: stats command returns stats envelope_kind

- **WHEN** the caller runs `dont stats --json`
- **THEN** `envelope_kind` is `"stats"`
- **AND** `data` conforms to the `StatsView` payload shape

#### Scenario: eval export command returns eval_export envelope_kind

- **WHEN** the caller runs `dont export --eval --json`
- **THEN** `envelope_kind` is `"eval_export"`
- **AND** `data` conforms to the `EvalExport` payload shape
