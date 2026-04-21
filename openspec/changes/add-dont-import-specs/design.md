## Context

Most remaining monolith content is now isolated into focused changes. Import is the next coherent batch: it defines how `dont` brings external vocabulary and references into local relations without invoking an LLM. Section 15 also carries the one importer with materially different behaviour, `dont import linkml`, whose subprocess dependency and lossy lowering need their own boundary.

## Goals
- Capture the common import command contract separately from any one adapter
- Capture LinkML-specific behaviour as a dedicated capability because it has unique failure modes and semantics
- Preserve the distinction between HTTP-backed importers and local-file/subprocess importers

## Non-Goals
- Specify the full importer configuration schema in `config.toml`
- Specify transport concerns outside the import command family
- Specify all downstream uses of imported rules beyond the adapter contract itself

## Decisions
- **Two capabilities, not one**: most importers share a common command contract, while LinkML is exceptional enough to deserve its own capability.
- **No-LLM contract is normative**: import is grounding infrastructure, not reasoning; keeping this explicit prevents accidental coupling to harness logic.
- **Rate limits stay aligned with verify-evidence**: the shared network politeness contract is part of the externally visible behaviour, even though the exact implementation is deferred.
- **LinkML tiers are first-class semantics**: flattened-without-warning, imported-with-warning, and refused-without-partial-import are key operator expectations and should be testable requirements.

## Source Mapping
- `dont-import-surface`: §15 importer command list, idempotence target relations, rate limiting, and auxiliary-tool dependency notes
- `dont-linkml-import`: §15 LinkML adapter scope, generated-rule note, and unsupported-feature refusal semantics

## Risks / Trade-offs
- The common import surface may seem thin compared to LinkML.
  - Mitigation: keep shared guarantees together so future adapters can reference them.
- LinkML details could drift into implementation specifics.
  - Mitigation: specify observable behaviour and feature classes, not subprocess plumbing internals beyond what the operator sees.

## Open Questions
- Whether future non-LinkML adapters will need their own capability splits once they gain substantial adapter-specific semantics.
