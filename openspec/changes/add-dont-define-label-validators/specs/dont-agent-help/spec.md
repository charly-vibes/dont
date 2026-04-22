## MODIFIED Requirements

### Requirement: Orientation prompt contract
The system SHALL provide a minimum-viable orientation prompt for LLM sessions. The orientation text MUST instruct the LLM to use `--json`, distinguish the core verbs from lifecycle verbs, explain permissive versus strict mode, require remediation-driven recovery on refusal, require harness fulfilment of spawn requests, recommend `dont suggest-term` before `define`, recommend supplying `--label "<a noun phrase>"` alongside `--doc` when coining terms (noting that the label is shape-checked and appears in diagrams), and point to `dont help --tutorial` for the full teaching walkthrough.

#### Scenario: refusal guidance in orientation block
- **WHEN** the reader consults the orientation block
- **THEN** it instructs them to read `data.remediation[0].command` and run it rather than guessing reformulations

#### Scenario: spawn guidance in orientation block
- **WHEN** the orientation block describes `spawn_request` envelopes
- **THEN** it tells the reader to invoke the harness subagent mechanism rather than performing the verification in the original session

#### Scenario: label coining guidance in orientation block
- **WHEN** the reader consults the orientation block
- **THEN** it explicitly recommends passing `--label '<a noun phrase>'` alongside `--doc` when coining terms
- **AND** it explains that the label is what appears in diagrams and is shape-checked
- **AND** the guidance appears between the `suggest-term` recommendation line and the `dont help --tutorial` pointer

#### Scenario: orientation points to deeper docs
- **WHEN** the orientation block reaches the end of its quick-start guidance
- **THEN** it points to `dont help <cmd>`, `.dont/AGENTS.md`, `dont help --tutorial`, and `dont help --howto <topic>` for more detail
