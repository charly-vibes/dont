## ADDED Requirements

### Requirement: Single Binary Distribution
The `dont` tool SHALL be distributed as a single binary with no runtime dependencies beyond the host operating system. Auxiliary import adapters (e.g., `dont import linkml`) MAY require separately installed tools; the core binary itself SHALL not.

#### Scenario: Binary executes without external runtime
- **WHEN** the `dont` binary is placed on a system without Rust, CozoDB, or any `dont`-specific runtime installed
- **THEN** `dont --version` SHALL execute successfully and print a version string

#### Scenario: Core commands work without auxiliary tools
- **WHEN** a project uses only core verbs (`conclude`, `define`, `trust`, `dismiss`) and query commands (`show`, `list`)
- **THEN** no tool beyond the `dont` binary is required

### Requirement: Embedded Storage Engine
The `dont` tool SHALL embed its storage engine within the binary, requiring no external database process or network connection for local operations.

#### Scenario: Database created locally without network
- **WHEN** `dont init` is run in a directory with no prior `dont` project and no network connectivity
- **THEN** a local database SHALL be created under `.dont/` and the command SHALL succeed

#### Scenario: No external database process required
- **WHEN** `dont` commands are executed on an initialized project
- **THEN** no external database process (e.g., PostgreSQL, Redis) SHALL be required for the commands to function

### Requirement: Cold Start Performance
The `dont` tool SHALL achieve sub-50ms cold start for read-only operations on projects with fewer than 10,000 claims.

#### Scenario: Fast read on small project
- **WHEN** `dont list --json` is run on a project containing 100 claims
- **THEN** the command SHALL complete (exit) within 50 milliseconds of process start

#### Scenario: Fast read on medium project
- **WHEN** `dont show <id> --json` is run on a project containing 5,000 claims
- **THEN** the command SHALL complete within 50 milliseconds of process start
