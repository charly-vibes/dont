# dont-envelope spec delta: source from genesis

## MODIFIED Requirements

### Requirement: Versioned output envelope

dont SHALL source its output envelope from `genesis::envelope` rather than a local `src/envelope.rs`. The deployed envelope_version `"0.2"` contract and all field semantics (`ok`, `envelope_kind`, `hints`, `warnings`) SHALL be preserved unchanged; genesis's module SHALL conform to this contract.

#### Scenario: envelope shape unchanged after adoption

- **WHEN** `dont prime --json` is run after adopting genesis
- **THEN** the emitted JSON SHALL have top-level keys `ok`, `envelope_version`, `cli_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`
- **AND** `envelope_version` SHALL remain `"0.2"`
- **AND** no local `Envelope` struct SHALL remain in `src/envelope.rs`.
