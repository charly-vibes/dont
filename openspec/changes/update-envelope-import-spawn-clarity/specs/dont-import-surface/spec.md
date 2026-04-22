## MODIFIED Requirements

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

## ADDED Requirements

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
