# dont-import-surface Specification

## Purpose
Defines the full `dont import` command family — syntax, supported adapters, output envelope,
idempotence contract, HTTP rate-limiting policy, URL-safety rules, and error-code mapping.

Import commands are lightweight grounding adapters: they fetch or read an external source and
project the result into local `imported_term`, `reference`, or `prefix` relations.  They do not
invoke an LLM and do not require MCP.  Only `dont import linkml` requires an auxiliary CLI tool
(see `dont-linkml-import`); all other adapters are self-contained.

The supported adapter set for v0.3 is:
- `dont import obo <path.owl|.obo|.ttl|url>` — OBO/OWL/Turtle files or HTTP URLs
- `dont import ols <ontology-prefix>` — OLS REST API
- `dont import wikidata --entity <Qid> | --sparql <file.rq>` — Wikidata entity or SPARQL query
- `dont import openalex --work <doi> | --snapshot <path>` — OpenAlex paper or local snapshot
- `dont import bioregistry` — Bioregistry prefix registry
- `dont import jsonld <file>` — local JSON-LD file
- `dont import ttl <file>` — local Turtle file
- `dont import linkml <schema.yaml>` — LinkML schema (subprocess adapter, lossy)

Every adapter produces a standard success envelope (`ok: true`) with an `adapter`, `schema_name`
(or equivalent source identifier), and `stored` count in `data`, plus any warnings in
`warnings[]`.  Every error path produces a standard error envelope (`ok: false`) with a
deterministic `code` value and non-empty `remediation[]`.

Each adapter may be disabled per-project by setting `enabled = false` under the corresponding
`[import.<adapter>]` block in `config.toml`.  Disabled adapters refuse immediately with
`code: "adapter-disabled"` and a remediation showing the config stanza to re-enable.
## Requirements
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
The system SHALL treat import as a grounding operation that writes only to import-related local relations. Importers MUST write to `imported_term`, `reference`, or `prefix` as appropriate. Repeated import of the same source identity MUST be idempotent rather than duplicating imported state.

For idempotence, each importer SHALL derive a deterministic `canonical_source_id` with these normalization rules:
- OLS: lowercase trimmed ontology prefix
- Wikidata `--entity`: uppercase trimmed QID
- Wikidata `--sparql`: SHA-256 hash of normalized query text where normalization removes full-line and trailing `#...` comments, normalizes line endings to `\n`, collapses contiguous whitespace to a single ASCII space, and trims leading/trailing whitespace
- OpenAlex `--work`: normalized DOI (trimmed, lowercased, with leading `doi:` and `https://doi.org/` removed before hashing/identity)
- OpenAlex `--snapshot`: `realpath` plus SHA-256 of file bytes
- OBO/TTL/JSON-LD URL imports: normalized absolute URL (lowercased scheme/host, default port elision, dot-segment removal)
- Local file imports: `realpath` plus SHA-256 of file bytes

Equivalent source identities MUST map to the same `canonical_source_id`. For local-file and snapshot imports, implementations MUST treat content hash as the deduplication authority when path aliases (symlink/hardlink/case variants) differ.

#### Scenario: import populates import relations
- **WHEN** an import succeeds
- **THEN** the resulting grounded data is written into the local import relations rather than directly into coined `term` entities

#### Scenario: re-import is idempotent
- **WHEN** the caller imports the same canonical source identity twice
- **THEN** the second import does not duplicate previously imported rows for that source identity

#### Scenario: equivalent inputs map to same canonical source identity
- **WHEN** two importer inputs normalize to the same source identity (for example DOI case variation)
- **THEN** they produce the same `canonical_source_id` and idempotent write behavior

#### Scenario: path aliases deduplicate by content identity
- **WHEN** the same local file bytes are imported through two path aliases
- **THEN** the second import is deduplicated by content-hash identity

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

### Requirement: Import URL scheme and destination policy
The system SHALL apply URL scheme and destination safety checks to HTTP-backed imports before issuing requests.

- Supported schemes for network fetch are `http` and `https` only.
- Imports MUST refuse destinations resolving to loopback, link-local, multicast, or private-network address ranges (IPv4 and IPv6).
- If DNS resolution yields multiple addresses, the import MUST be refused when any resolved address is in a blocked class.

#### Scenario: loopback destination is refused
- **WHEN** a caller runs an HTTP-backed import pointing to a URL that resolves to `127.0.0.1`, `::1`, or localhost-equivalent
- **THEN** the command is refused before network fetch begins

#### Scenario: private-network destination is refused
- **WHEN** a caller runs an HTTP-backed import pointing to RFC1918 or ULA private address space
- **THEN** the command is refused before network fetch begins

#### Scenario: mixed DNS answer set is refused
- **WHEN** a URL resolves to multiple addresses and at least one address is in a blocked class
- **THEN** the command is refused before network fetch begins

#### Scenario: non-http scheme is refused
- **WHEN** a caller provides a URL with a non-HTTP(S) scheme
- **THEN** the command is refused before network fetch begins

### Requirement: Import safety refusal-code mapping
The system SHALL use deterministic error-code mapping for import safety refusals:
- Unsupported URL scheme or blocked resolved destination class SHALL use `unresolvable-uri`
- Missing or invalid import safety configuration required to evaluate policy SHALL use `config-missing`

All refusals SHALL include remediation describing a permitted endpoint or configuration correction.

#### Scenario: blocked destination uses unresolvable-uri
- **WHEN** an HTTP-backed importer blocks a destination because of scheme or resolved address class
- **THEN** the command returns `code: "unresolvable-uri"`

#### Scenario: policy configuration failure uses config-missing
- **WHEN** the importer cannot evaluate safety policy due to missing or invalid required configuration
- **THEN** the command returns `code: "config-missing"`
