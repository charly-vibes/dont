## ADDED Requirements

### Requirement: Epistemic forcing-function purpose
The system SHALL define `dont` as a project-local command-line tool whose purpose is to interrupt unsupported assertions by autonomous LLM harnesses and require grounding before claims or terms may become `verified`.

#### Scenario: tool purpose is framed around grounding before verification
- **WHEN** a project adopts `dont`
- **THEN** the tool is specified as enforcing doubt and grounding rather than task tracking, workflow orchestration, or generic knowledge-base authoring

### Requirement: Tool independence
The system SHALL specify `dont`, `wai`, and `beads`/`bd` as independent tools that may share conventions and author identity strings but do not require shared code, shared configuration, or a shared runtime.

#### Scenario: companion tool roles remain distinct
- **WHEN** the project describes how `dont` interacts with companion tools
- **THEN** `wai` is treated as workflow context, `beads`/`bd` as memory or issue tracking, and `dont` as epistemic discipline

#### Scenario: dont can operate without companion-tool runtime coupling
- **WHEN** `dont` is executed in a project that follows the shared conventions
- **THEN** its specification does not require `wai` or `beads`/`bd` to be linked into the same runtime or configuration surface

### Requirement: Append-only event history
The system SHALL represent state changes as append-only events and SHALL forbid destructive deletion as a normal state-management mechanism.

#### Scenario: retraction is recorded instead of deleting history
- **WHEN** a claim or term is challenged, superseded, or otherwise reconsidered
- **THEN** the specification records that as a new event in history instead of deleting the earlier assertion
