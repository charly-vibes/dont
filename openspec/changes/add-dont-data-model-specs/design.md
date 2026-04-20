## Context

The monolithic spec interleaves storage architecture (§4.2), entity model (§4.3), relation schemas (§5.2-5.3), MVP primitives (§6), view shapes (§10.4), and input schemas (§10.6). These serve two distinct audiences: implementers of the storage layer (data model) and consumers of the wire format (payload types). The prior decomposition established envelope contract, error taxonomy, and CLI surface as the machine-parseable infrastructure layer. This change extracts the data shapes those layers carry.

## Goals
- Capture what is stored (entities, relations, event kinds, primitives) as a standalone testable capability
- Capture what is on the wire (view shapes, input schemas) as a separate capability that references the data model
- Preserve the Clojure/Datomic-style "attributes are first-class, classes are queries" philosophy normatively
- Include the suggest-term contract since its return shape and search behaviour are payload-type concerns

## Non-Goals
- Specify the CozoDB substrate choice as normative — that is an implementation decision; the spec captures the abstract storage semantics
- Specify derived commands (guess, assume, overlook) — those are orchestration over the data model
- Specify the rule engine — rules reference the data model but are a separate concern

## Decisions
- **Two capabilities, not one**: The data model (what's stored) and payload types (what's on the wire) change independently. A storage migration might change relation shapes without altering view types; a new CLI command adds a view type without changing the data model.
- **Storage semantics, not CozoDB specifics**: The data model spec captures the datom shape, event-sourcing invariants, transaction semantics, and signal handling as abstract requirements. "CozoDB" appears in rationale, not in normative requirements. This allows future substrate changes without spec rewrites.
- **Atom-completion gate lives in data model**: The gate is a stored-state invariant (when atoms reach verified, the claim auto-promotes). It references the status lattice (`dont-status-lifecycle`) and is referenced by verb specs (`dont-cli-core`).
- **Input schemas paired with view types**: Input and output shapes are the same concern boundary — "what does the wire look like?" Keeping them in one capability means consumers have a single reference for JSON Schema generation.
- **suggest-term in payload types**: Its contract is about search behaviour and return shape, not about storage. It searches across coined terms and imported terms but does not write.

## Source Mapping
- `dont-data-model`: §4.2 (storage: datom shape, event-sourcing, transactions, concurrency, signals), §4.3 (entities: id, kind, attributes, history), §5.2 (core relations: entity, attribute, event, evidence, depends_on; kind disambiguation; canonical event_kind list; atom model; atom-completion gate), §5.3 (import relations: imported_term, reference, prefix; CURIE collision), §6 (MVP primitives: attribute, derived_class, enum, prefix, rule)
- `dont-payload-types`: §10.1 (suggest-term), §10.4 (ClaimView, TermView, EventView, SpawnRequest, PrimeView, WhyView, ClaimsList, DoctorReport, ExamplesList, SchemaDoc; applicable_rules with gate/flag kinds), §10.6 (input schemas: ConcludeInput through ImportInput; cardinality notation; AuthorString and EntityId shapes)

## Risks / Trade-offs
- The data model spec is large (entity model, relations, atoms, imports, primitives). Splitting further would fragment cross-references between atoms and core relations.
  - Mitigation: Use clear requirement grouping within the spec.
- Payload types include ~12 distinct view types. This is a large single spec.
  - Mitigation: Each view type is a separate requirement with its own scenarios; consumers can navigate by requirement name.
- The atom-completion gate straddles data model and CLI verbs. Placing it in data model means `dont-cli-core` must reference it.
  - Mitigation: Cross-reference established; `dont-cli-core` already references `dont-status-lifecycle` for status transitions.

## Open Questions
- Should `applicable_rules` shape (gate/flag discriminator) be in data model or payload types? Currently in payload types since it's a view concern.
