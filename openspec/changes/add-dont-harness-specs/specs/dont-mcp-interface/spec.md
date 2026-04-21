## ADDED Requirements
### Requirement: Optional MCP server mode
The system SHALL provide an optional `dont mcp` mode that runs `dont` as an MCP server over stdio. This mode SHALL expose the same command surface as MCP tools without redefining the underlying command semantics.

#### Scenario: harness starts MCP mode
- **WHEN** a harness launches `dont mcp`
- **THEN** `dont` starts an MCP server over stdio
- **AND** the server exposes tool entry points corresponding to the supported `dont` command surface

### Requirement: Envelope-preserving MCP tool results
Every MCP tool exposed by `dont mcp` SHALL return the same `Envelope` JSON contract used by the direct CLI surface as its tool result payload. MCP transport MUST NOT introduce a second result schema that diverges from `dont-envelope` and `dont-errors`.

#### Scenario: MCP tool returns success
- **WHEN** an MCP client invokes a `dont` tool that succeeds
- **THEN** the tool result contains the standard success envelope JSON
- **AND** the payload shape matches the corresponding CLI contract

#### Scenario: MCP tool returns refusal or error
- **WHEN** an MCP client invokes a `dont` tool that refuses or fails
- **THEN** the tool result contains the standard error envelope JSON
- **AND** the remediation and error-code semantics match the direct CLI contract

### Requirement: MCP is optional and non-privileged
The system SHALL treat MCP as an optional transport for harnesses that prefer MCP over direct CLI calls. MCP mode MUST NOT become the privileged or canonical integration surface; harnesses using generic shell execution SHALL retain the same behavioural contract.

#### Scenario: harness uses direct CLI instead of MCP
- **WHEN** a harness invokes `dont` through direct CLI commands rather than `dont mcp`
- **THEN** the harness receives the same behavioural surface and envelope contracts
- **AND** the absence of MCP does not reduce command availability
