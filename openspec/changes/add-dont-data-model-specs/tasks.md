## 1. Extract data model capability
- [ ] 1.1 Write `dont-data-model` spec: entity structure (id, kind, attributes, history)
- [ ] 1.2 Include storage semantics (datom shape, event-sourcing, transactions, concurrency, signal handling)
- [ ] 1.3 Include core relations (entity, attribute, event, evidence, depends_on)
- [ ] 1.4 Include kind disambiguation (entity_kind, event_kind, envelope_kind)
- [ ] 1.5 Include canonical event_kind list with forward-compatibility
- [ ] 1.6 Include atom model and atom-completion gate
- [ ] 1.7 Include import relations (imported_term, reference, prefix) and CURIE collision semantics
- [ ] 1.8 Include five MVP primitives (attribute, derived_class, enum, prefix, rule)

## 2. Extract payload types capability
- [ ] 2.1 Write `dont-payload-types` spec: ClaimView shape with atoms and applicable_rules
- [ ] 2.2 Include TermView, EventView, and SpawnRequest shapes
- [ ] 2.3 Include PrimeView and WhyView shapes
- [ ] 2.4 Include list/collection shapes (ClaimsList) and diagnostic shapes (DoctorReport, ExamplesList, SchemaDoc)
- [ ] 2.5 Include applicable_rules gate/flag discriminator contract
- [ ] 2.6 Include suggest-term search behaviour and return shape
- [ ] 2.7 Include all input schemas (ConcludeInput through ImportInput) with cardinality and type conventions
- [ ] 2.8 Include AuthorString and EntityId shape definitions

## 3. Validate and review
- [ ] 3.1 Run `openspec validate add-dont-data-model-specs --strict`
- [ ] 3.2 Run Rule-of-5 review on both specs
