## 1. Extract data model capability
- [x] 1.1 Write `dont-data-model` spec: entity structure (id, kind, attributes, history)
- [x] 1.2 Include storage semantics (datom shape, event-sourcing, transactions, concurrency) without OS signal/retry logic
- [x] 1.3 Include core relations (entity, attribute, event, evidence, depends_on)
- [x] 1.4 Include kind disambiguation (entity_kind, event_kind)
- [x] 1.5 Include closed canonical event_kind list with explicit versioned expansion rules
- [x] 1.6 Include atom model and atom-completion gate as a persisted invariant
- [x] 1.7 Include import relations (imported_term, reference, prefix) and CURIE shadowing semantics
- [x] 1.8 Include closed set of five MVP primitives (attribute, derived_class, enum, prefix, rule)

## 2. Extract payload types capability
- [x] 2.1 Write `dont-payload-types` spec: ClaimView shape with atoms and separate trace assessments
- [x] 2.2 Include TermView, EventView, and SpawnRequest passive shapes
- [x] 2.3 Include PrimeView and WhyView shapes
- [x] 2.4 Include list/collection shapes (ClaimsList) and diagnostic shapes (DoctorReport, ExamplesList, SchemaDoc)
- [x] 2.5 Include applicable_rules gate/flag discriminator contract as a computed view only
- [x] 2.6 Include suggest-term search behaviour mapping over both local and imported terms
- [x] 2.7 Include all input schemas (ConcludeInput through ImportInput) defining shape/arrays, not behaviour
- [x] 2.8 Include normative AuthorString and EntityId shape definitions

## 3. Validate and review
- [ ] 3.1 Run `openspec validate add-dont-data-model-specs --strict`
- [ ] 3.2 Run Rule-of-5 review on both specs
