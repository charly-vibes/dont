## ADDED Requirements

### Requirement: Supported import command family
The system SHALL provide the following importer commands: `dont import obo <path.owl|.obo|.ttl|url>`, `dont import ols <ontology-prefix>`, `dont import wikidata --entity <Qid> | --sparql <file.rq>`, `dont import openalex --work <doi> | --snapshot <path>`, `dont import bioregistry`, `dont import jsonld <file>`, `dont import ttl <file>`, and `dont import linkml <schema.yaml>`.

#### Scenario: HTTP-backed ontology import
- **WHEN** the caller runs `dont import ols efo --json`
- **THEN** the command uses the OLS adapter contract for that ontology prefix

#### Scenario: local-file import variant
- **WHEN** the caller runs `dont import ttl ontology.ttl --json`
- **THEN** the command imports from the local Turtle file rather than requiring a network source

#### Scenario: obo importer accepts URL source
- **WHEN** the caller runs `dont import obo https://example.org/ontology.owl --json`
- **THEN** the command treats the argument as an HTTP-backed source for the OBO adapter

#### Scenario: openalex source forms are distinct
- **WHEN** the caller uses `dont import openalex --work <doi>` or `--snapshot <path>`
- **THEN** both forms are accepted as first-class source modes for the OpenAlex adapter

### Requirement: Import writes and idempotence
The system SHALL treat import as a grounding operation that writes only to import-related local relations. Importers MUST write to `imported_term`, `reference`, or `prefix` as appropriate, and repeated import of the same source URI or equivalent source identity MUST be idempotent rather than duplicating imported state.

#### Scenario: import populates import relations
- **WHEN** an import succeeds
- **THEN** the resulting grounded data is written into the local import relations rather than directly into coined `term` entities

#### Scenario: re-import is idempotent
- **WHEN** the caller imports the same source twice
- **THEN** the second import does not duplicate previously imported rows for that source identity

### Requirement: Import is non-LLM and non-MCP work
The system SHALL execute imports without invoking an LLM and without requiring MCP as a transport. Import commands are grounding adapters over HTTP, local files, or local subprocesses, not reasoning tasks.

#### Scenario: import does not spawn reasoning
- **WHEN** the caller runs any `dont import ...` command
- **THEN** the operation completes without emitting a `spawn_request` envelope
- **AND** it does not depend on harness-mediated reasoning

### Requirement: Shared HTTP rate limiting
The system SHALL apply a shared network politeness contract to HTTP-backed importers. For importers that fetch over HTTP, the default behaviour MUST cap concurrency at four requests per invocation, sustain at most two requests per second per host with a burst of four, honour `Retry-After` on `429` and `503`, and use the same `User-Agent` convention as evidence verification. Local-file imports MUST be exempt from HTTP rate limiting. Projects MUST be able to tighten these defaults on a per-importer basis through importer-specific configuration blocks.

#### Scenario: HTTP importer uses rate limits
- **WHEN** the caller runs an HTTP-backed importer such as `dont import wikidata --entity Q42`
- **THEN** outbound requests observe the shared per-host concurrency and retry policy

#### Scenario: local snapshot is not rate-limited
- **WHEN** the caller runs `dont import openalex --snapshot snapshot.json`
- **THEN** the local-file import is not subject to HTTP request throttling

#### Scenario: importer-specific rate limits can be tightened
- **WHEN** the project config tightens the HTTP politeness settings for one importer
- **THEN** that importer uses the more restrictive per-importer limits instead of the shared defaults

### Requirement: Auxiliary-tool expectation boundary
The system SHALL treat most importers as self-contained adapters, with LinkML as the explicit auxiliary-tool exception. Missing LinkML tooling MUST refuse the command with `config-missing` and remediation pointing at LinkML installation. `dont doctor` MUST report LinkML availability as a warning check rather than a hard failure so projects not using LinkML remain healthy.

#### Scenario: missing linkml cli refuses command
- **WHEN** the caller runs `dont import linkml schema.yaml` and the `linkml` CLI is not on `PATH`
- **THEN** the command returns an error with code `config-missing`
- **AND** remediation points at installing the LinkML CLI

#### Scenario: doctor warns but does not fail on missing linkml
- **WHEN** LinkML tooling is unavailable in a project that otherwise works
- **THEN** `dont doctor` reports that availability as a warning rather than a fail check
