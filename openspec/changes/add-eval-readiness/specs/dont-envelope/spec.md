## MODIFIED Requirements

> **Change:** Extends the canonical `envelope_kind` enum to include new values introduced by the
> `add-eval-readiness` change.

### Requirement: Typed payload discriminator

The `envelope_kind` field SHALL include the following additional canonical values as of this change:

- `"stats"` — payload is a `StatsView` document as defined in `dont-analytics`
- `"eval_export"` — payload is an `EvalExport` document as defined in `dont-eval-export`

All existing parsers that implement the forward-compatible unknown-kind default branch required by
the base `dont-envelope` spec will handle these new values without modification.

#### Scenario: stats command returns stats envelope_kind

- **WHEN** the caller runs `dont stats --json`
- **THEN** `envelope_kind` is `"stats"`
- **AND** `data` conforms to the `StatsView` payload shape

#### Scenario: eval export command returns eval_export envelope_kind

- **WHEN** the caller runs `dont export --eval --json`
- **THEN** `envelope_kind` is `"eval_export"`
- **AND** `data` conforms to the `EvalExport` payload shape
