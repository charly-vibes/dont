## Context

Most remaining monolith content is now isolated into focused changes. Import is the next coherent batch: it defines how `dont` brings external vocabulary and references into local relations without invoking an LLM. Section 15 also carries the one importer with materially different behaviour, `dont import linkml`, whose subprocess dependency and lossy lowering need their own boundary.

## Goals
- Capture the common import command contract separately from any one adapter, including strict declarative idempotence and rule bypass.
- Capture LinkML-specific behaviour as a dedicated capability because it has unique failure modes and explicit lossy lowering tiers.
- Preserve the distinction between HTTP-backed importers (enforcing default hardcoded rate limits) and local-file/subprocess importers.

## Non-Goals
- Specify the full importer configuration schema in `config.toml`
- Specify transport concerns outside the import command family
- Specify all downstream uses of imported rules beyond the adapter contract itself

## Decisions
- **Two capabilities, not one**: most importers share a common command contract, while LinkML is exceptional enough to deserve its own capability.
- **No-LLM contract is normative**: import adapters MUST be strictly deterministic and mechanistic. They MUST NOT use LLMs for summarization, translation, or mapping.
- **Hardcoded Rate Limit**: A hardcoded default HTTP rate limit (e.g. 5 req/sec) MUST apply globally to all HTTP importers to prevent DoS, but it CAN be overridden via `config.toml`.
- **Strict Idempotence**: All import operations MUST be fully declarative and perform a replace-or-upsert sync based on source identity, removing stale/deleted terms on re-import.
- **Rule Bypass**: `dont import` commands MUST bypass project methodology rules entirely. Imports CANNOT fail because a project rule (like `unresolved-terms`) was violated by the external ontology.
- **LinkML tiers are first-class semantics**: flattened-without-warning, imported-with-warning, and refused-without-partial-import are key operator expectations and should be testable requirements.
- **Doctor Integration**: `dont doctor` MUST perform a pre-flight environment check for the external Python `linkml` binary, emitting a warning if not found.

## Source Mapping
- `dont-import-surface`: §15 importer command list, idempotence target relations, rate limiting, and auxiliary-tool dependency notes
- `dont-linkml-import`: §15 LinkML adapter scope, generated-rule note, and unsupported-feature refusal semantics

## Risks / Trade-offs
- The common import surface may seem thin compared to LinkML.
  - Mitigation: keep shared guarantees together so future adapters can reference them.
- LinkML details could drift into implementation specifics.
  - Mitigation: specify observable behaviour and feature classes, not subprocess plumbing internals beyond what the operator sees.

## Open Questions
- None remain for the high-level boundary of these two capabilities; semantics resolved by design interview.
