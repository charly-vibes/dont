## ADDED Requirements

### Requirement: LinkML adapter shell-out and lowering contract
The system SHALL implement `dont import linkml <schema.yaml>` as a subprocess-backed adapter over the LinkML CLI. The adapter MUST shell out to LinkML tooling to derive intermediate forms, lower the result into local import relations, and MAY additionally generate Datalog rules for imported constraints.

#### Scenario: linkml import lowers to local relations
- **WHEN** the caller imports a supported LinkML schema
- **THEN** the adapter lowers the schema into local import relations usable by `dont`

#### Scenario: linkml import may generate rules
- **WHEN** imported LinkML constraints can be represented as local rules
- **THEN** the adapter may emit generated Datalog rules as part of the import result

### Requirement: Lossy-by-design adapter boundary
The system SHALL treat LinkML import as lossy by design rather than as a round-tripping implementation of full LinkML semantics. Projects needing full LinkML fidelity MUST use LinkML directly rather than relying on `dont` to preserve all semantics.

#### Scenario: grounding rather than round-tripping
- **WHEN** an operator uses `dont import linkml`
- **THEN** the adapter's contract is to ground useful vocabulary and constraints for `dont`
- **AND** not to preserve every LinkML semantic feature losslessly

### Requirement: Flattened feature tier
The system SHALL flatten certain LinkML features without warning because they are expected structural lowerings. `is_a` inheritance MUST become a transitive `kind_of` chain, mixins MUST be flattened into the inheriting class's attribute set, and `slot_usage` refinements MUST be expanded into per-class attribute predicates.

#### Scenario: inheritance becomes kind_of chain
- **WHEN** a LinkML schema uses `is_a`
- **THEN** the import result represents that inheritance as a traversable `kind_of` chain

#### Scenario: mixins flatten silently
- **WHEN** a LinkML schema uses mixins
- **THEN** the mixin attributes are flattened into the inheriting class without producing a warning solely for that transformation

### Requirement: Warning-tier feature handling
The system SHALL import certain LinkML features approximately and warn about that approximation. `permissible_values` enums, string patterns, value ranges, and minimum-cardinality constraints MUST be translated into local rule representations when possible, and the import result MUST record warnings naming each approximated feature per source.

#### Scenario: enum import warns
- **WHEN** the schema contains `permissible_values`
- **THEN** the adapter imports the constraint approximately
- **AND** records a warning naming that feature in the source schema

#### Scenario: pattern or range import warns
- **WHEN** the schema contains string-pattern or value-range constraints
- **THEN** the adapter lowers them into local rule forms where possible
- **AND** emits warnings so consumers know the translation is approximate

### Requirement: Refusal-tier unsupported features
The system SHALL refuse schemas containing unsupported LinkML features that cannot be safely approximated. Expressions requiring SPARQL evaluation, reified slots, and custom `python_class` injections MUST trigger `linkml-unsupported-feature` and list the offending constructs.

#### Scenario: unsupported feature refuses import
- **WHEN** the schema uses reified slots or SPARQL-evaluated expressions
- **THEN** the import fails with code `linkml-unsupported-feature`
- **AND** the error identifies the unsupported constructs

### Requirement: No partial import on refusal
The system SHALL avoid partial state when unsupported LinkML features are encountered. If refusal-tier features are present, the adapter MUST not import a partial subset of the schema.

#### Scenario: unsupported schema leaves no partial state
- **WHEN** `dont import linkml` refuses because of an unsupported feature
- **THEN** none of the schema's terms or generated rules are imported into the local store
