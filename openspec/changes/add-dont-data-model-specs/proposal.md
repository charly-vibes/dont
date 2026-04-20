# Change: add dont data model and payload type specs

## Why

The data model (entity structure, core relations, import relations, MVP primitives) and the payload types (view shapes returned by commands, input schemas accepted by commands) are currently specified only in `dont-spec-v0_3_2.md` (sections 4.2-4.3, 5.2-5.3, 6, 10.1, 10.4, 10.6). Extracting them as focused OpenSpec capabilities makes the stored-shape and wire-shape contracts testable, versioned, and independently evolvable. Downstream specs (harness integration, spawn protocol, import) depend on these shapes.

## What Changes
- Add a capability for the data model: entity structure, CozoDB storage semantics, core relations (entity, attribute, event, evidence, depends_on), import relations (imported_term, reference, prefix), and the five MVP primitives (attribute, derived_class, enum, prefix, rule).
- Add a capability for payload types: all view types (ClaimView, TermView, EventView, SpawnRequest, PrimeView, WhyView, ClaimsList, DoctorReport, ExamplesList, SchemaDoc), input schemas (ConcludeInput through ImportInput), and the suggest-term command contract.

## Deferred
- Spawn-request protocol (assume/overlook/guess orchestration, harness detection, timeouts) — separate operational concern
- Methodology rules (Datalog rule system, shipped rules, severity table) — separate concern
- Project layout (config.toml schema, directory structure) — infrastructure
- Import adapters (OBO, OLS, Wikidata, etc.) — separate operational concern
- Self-teaching surface (orientation block, worked examples, how-to guides) — documentation/UX

## Traceability
- `dont-data-model` is sourced from sections 4.2, 4.3, 5.2, 5.3, and 6 of `dont-spec-v0_3_2.md`
- `dont-payload-types` is sourced from sections 10.1, 10.4, and 10.6 of `dont-spec-v0_3_2.md`

## Impact
- Affected specs: `dont-data-model`, `dont-payload-types` (both new)
- Cross-references: `dont-status-lifecycle` (status lattice), `dont-envelope` (envelope contract), `dont-errors` (error codes), `dont-cli-core` (verb definitions)
- Affected workflow: future harness-integration, spawn-protocol, and rule-engine specs can reference these data shapes rather than restating them
