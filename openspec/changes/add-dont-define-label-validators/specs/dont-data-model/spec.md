## MODIFIED Requirements

### Requirement: Term-specific attributes
Terms SHALL carry these attributes: `curie` (string, the term's compact URI), `definition` (string, prose definition), `label` (string, optional; the SK11 type-text — a singular indefinite noun phrase suitable for use in olog diagrams), `kind_of[]` (array of CURIEs referencing parent terms), `related_to[]` (array of CURIEs referencing related terms), `status` (lattice value per `dont-status-lifecycle`), and `provenance` (structured object). `kind_of[]` and `related_to[]` create edges traversed by `stale-cascade` and `dangling-definition` rules.

Seed entries (populated during `dont init`) that omit `label` in the seed YAML suppress the verb-level label-shape validators during seed installation only. Suppression is limited to the `dont init` seed pass; subsequent `dont define` calls on the same CURIE are subject to normal validator behaviour. The seed is pre-locked and is not round-tripped through `define`, so no `term-shape-*` refusal fires during seed installation regardless of whether `label` is present.

#### Scenario: term carries curie and definition
- **WHEN** a term is defined with `dont define proj:RicciTensor --doc "..."`
- **THEN** the stored term has `curie: "proj:RicciTensor"` and `definition` matching the provided text

#### Scenario: term carries label when supplied
- **WHEN** a term is defined with `dont define proj:RicciTensor --label "a Ricci tensor" --doc "..."`
- **THEN** the stored term has `label: "a Ricci tensor"` alongside `curie` and `definition`

#### Scenario: label is absent when not supplied
- **WHEN** a term is defined without `--label`
- **THEN** the stored term has no `label` attribute; the field is optional

#### Scenario: seed entry without label suppresses shape checks
- **WHEN** `dont init` installs seed terms from YAML entries that omit `label`
- **THEN** no `term-shape-*` refusal fires for those entries during installation

#### Scenario: kind_of creates traversable edges
- **WHEN** a term has `kind_of: ["dont:Term"]`
- **THEN** the `stale-cascade` rule can traverse from the parent term to this term
