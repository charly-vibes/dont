## ADDED Requirements

### Requirement: Managed agent-document block
The system SHALL own a managed documentation block describing `dont` usage in project docs. `.dont/AGENTS.md` MUST be the canonical LLM-facing document. Root-level managed blocks in files such as `AGENTS.md` and `CLAUDE.md` MUST remain shorter pointers to `.dont/AGENTS.md` and MUST warn that edits inside the managed markers will be overwritten by `dont sync-docs`.

#### Scenario: managed block points to canonical docs
- **WHEN** `dont` renders the managed block into a project document
- **THEN** the block tells the reader to run `dont prime --json` at session start
- **AND** it points to `.dont/AGENTS.md` or `dont help` for full usage

#### Scenario: managed block marks overwrite boundary
- **WHEN** a project document contains the managed block markers
- **THEN** the content between those markers is treated as tool-owned and overwriteable by `dont sync-docs`

### Requirement: Orientation prompt contract
The system SHALL provide a minimum-viable orientation prompt for LLM sessions. The orientation text MUST instruct the LLM to use `--json`, distinguish the core verbs from lifecycle verbs, explain permissive versus strict mode, require remediation-driven recovery on refusal, require harness fulfilment of spawn requests, and recommend `dont suggest-term` before `define`.

#### Scenario: refusal guidance in orientation block
- **WHEN** the reader consults the orientation block
- **THEN** it instructs them to read `data.remediation[0].command` and run it rather than guessing reformulations

#### Scenario: spawn guidance in orientation block
- **WHEN** the orientation block describes `spawn_request` envelopes
- **THEN** it tells the reader to invoke the harness subagent mechanism rather than performing the verification in the original session

#### Scenario: orientation points to deeper docs
- **WHEN** the orientation block reaches the end of its quick-start guidance
- **THEN** it points to `dont help <cmd>`, `.dont/AGENTS.md`, `dont help --tutorial`, and `dont help --howto <topic>` for more detail

### Requirement: Help and teaching entry points
The system SHALL provide `dont help` as the primary agent-addressed help surface. Bare `dont help` MUST list the available commands and major help entry points. `dont help <cmd>` and `<cmd> --help` MUST route to the same command-specific help content. `dont help --tutorial` MUST print the first-session tutorial, `dont help --howto <topic>` MUST print a goal-oriented how-to guide, and `dont help --topics` MUST list the available tutorial and how-to topics.

#### Scenario: bare help lists commands and entry points
- **WHEN** the caller runs `dont help`
- **THEN** the output lists the available commands and the major tutorial/how-to entry points

#### Scenario: subcommand help routing matches help verb
- **WHEN** the caller runs `dont lock --help`
- **THEN** the output matches the content of `dont help lock`

#### Scenario: tutorial help entry point
- **WHEN** the caller runs `dont help --tutorial`
- **THEN** the output is the sequenced first-session walkthrough rather than per-command reference text

#### Scenario: how-to topic selection
- **WHEN** the caller runs `dont help --howto harness-integration`
- **THEN** the output is the corresponding goal-oriented guide if that topic exists

#### Scenario: help topics listing
- **WHEN** the caller runs `dont help --topics`
- **THEN** the output lists the available tutorial and how-to entry points

### Requirement: Canonical teaching artifacts
The system SHALL ship normative teaching artifacts beyond command reference. It MUST include a worked example showing the canonical define → conclude → spawn → dismiss → lock flow, a first-session tutorial that explains why each step is taken, and goal-oriented how-to guides for project-specific rule authoring, harness integration, and `.dont/` store recovery.

#### Scenario: worked example teaches canonical flow
- **WHEN** the caller reads the worked example artifact
- **THEN** it shows a representative session beginning with `dont prime --json`
- **AND** it demonstrates spawn-based verification rather than self-verification

#### Scenario: tutorial emphasises orient-search-coin-conclude-spawn loop
- **WHEN** the caller reads the first-session tutorial
- **THEN** it presents the workflow as a sequenced learning path rather than as isolated command reference entries

#### Scenario: how-to corpus covers the three named operator goals
- **WHEN** the caller browses the how-to guides
- **THEN** the corpus includes guides for authoring a project-specific rule, integrating `dont` into a new harness, and recovering a corrupted `.dont/` store

### Requirement: Managed-doc sync command
The system SHALL provide `dont sync-docs` to rewrite the managed block in configured project documents without editing surrounding user-authored content.

#### Scenario: sync-docs rewrites only managed region
- **WHEN** the caller runs `dont sync-docs`
- **THEN** the command rewrites the content between the managed markers in configured docs
- **AND** it leaves content outside those markers unchanged
