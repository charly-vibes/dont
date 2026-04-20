# Change: add dont envelope specs

## Why

The envelope contract, error taxonomy, and CLI shell conventions are the machine-parseable surface that every consumer — harness, MCP adapter, shell script — depends on. These are currently specified only in the monolithic `dont-spec-v0_3_2.md` (§10.2–10.7). Extracting them as focused OpenSpec capabilities makes the contract testable, versioned, and independently evolvable.

## What Changes
- Add a capability for the output envelope contract: versioning, field semantics, `envelope_kind` discriminator, identity/format conventions, and forward-compatibility rules.
- Add a capability for the error envelope and error-code taxonomy: `ErrorResult` shape, remediation invariant, exit codes, and the complete v0.3.2 error-code set.
- Add a capability for CLI shell conventions: universal flags, colour/terminal awareness, stdin piping, shell completions, and the help surface.

## Deferred
- Core payload types (`ClaimView`, `TermView`, `PrimeView`, etc.) — these are data-model shapes, not envelope infrastructure
- Input schemas (`ConcludeInput`, `DefineInput`, etc.) — these belong with the data model
- Derived commands (`guess`, `assume`, `overlook`, `suggest-term`) — these are higher-level orchestrations
- Spawn protocol and rule engine — separate operational concerns

## Traceability
- `dont-envelope` is sourced from sections 10.2 and 10.3 of `dont-spec-v0_3_2.md`
- `dont-errors` is sourced from sections 10.5 and 10.7.1 of `dont-spec-v0_3_2.md`
- `dont-cli-surface` is sourced from sections 10.7.2–10.7.6 of `dont-spec-v0_3_2.md`

## Impact
- Affected specs: `dont-envelope`, `dont-errors`, `dont-cli-surface`
- Affected docs: `dont-spec-v0_3_2.md`, `openspec/project.md`
- Affected workflow: future payload-type and data-model specs can reference these envelope contracts rather than restating them
