## ADDED Requirements

### Requirement: ClaimView payload
The system SHALL return a `ClaimView` payload (envelope_kind: `"claim"`) containing: `id`, `entity_kind: "claim"`, `statement`, `status`, `confidence` (float | null; null when no LLM-authored value was provided), `atoms[]` (each with `idx`, `text`, `status`; atom-level `evidence[]` is stored but not inlined in ClaimView — use `dont why` for atom-level evidence detail), `hypotheses[]` (each with `idx`, `text`, `assessment` where `assessment = {supporting: string[], refuting: string[]}`), `evidence[]` (each with `source_uri`, `kind`, `supports`, `quote`), `depends_on[]`, `provenance`, `created_at`, `updated_at`, and `applicable_rules`. All array fields SHALL always be present (possibly empty), never omitted. Atom `status` SHALL be a three-value sub-lattice (`unverified`, `doubted`, `verified`) — atoms cannot become `stale`, `locked`, or `ignored`. The `applicable_rules` object SHALL be keyed by rule name with values discriminated by `rule_kind` (see applicable_rules contract). Future minor versions MAY add new fields to `ClaimView`; such additions SHALL be optional from the parser's perspective.

#### Scenario: ClaimView with atoms
- **WHEN** `dont show claim:X --json` is run on a claim with 3 atoms
- **THEN** the payload contains `atoms[]` with 3 entries, each carrying `idx`, `text`, and `status`

#### Scenario: ClaimView includes applicable rules
- **WHEN** a claim has a `lockable` rule that is not met
- **THEN** `applicable_rules.lockable` has `rule_kind: "gate"`, `met: false`, and `unmet` listing failing clauses

#### Scenario: ClaimView includes evidence array
- **WHEN** a claim has been dismissed with evidence
- **THEN** the payload contains `evidence[]` entries with `source_uri`, `kind`, `supports`, and optional `quote`

#### Scenario: ClaimView updated_at reflects latest transition
- **WHEN** a claim is trusted after creation
- **THEN** `updated_at` in the ClaimView is later than `created_at`

### Requirement: TermView payload
The system SHALL return a `TermView` payload (envelope_kind: `"term"`) containing: `id`, `entity_kind: "term"`, `curie`, `definition`, `kind_of[]`, `related_to[]`, `status`, `confidence` (float | null), `provenance`, `created_at`, and `applicable_rules`. `TermView` intentionally omits `updated_at` — term status transitions are tracked through the event history (see `dont why`).

#### Scenario: TermView includes kind_of hierarchy
- **WHEN** `dont show term:X --json` is run on a term with `kind_of: ["dont:Term"]`
- **THEN** the payload contains `kind_of: ["dont:Term"]`

#### Scenario: TermView includes applicable rules
- **WHEN** a term has an `unresolved-terms` gate rule that is met
- **THEN** `applicable_rules["unresolved-terms"]` has `rule_kind: "gate"`, `met: true`, `unmet: []`

### Requirement: EventView payload
The system SHALL return an `EventView` payload (envelope_kind: `"event"`) containing: `entity_id`, `tx`, `event_kind`, `at`, `author`, `reason` (nullable), `evidence_uri` (nullable), and `spawn_request_id` (nullable).

#### Scenario: EventView for a trust event
- **WHEN** a claim is trusted and the event is queried
- **THEN** the payload contains `event_kind: "trusted"`, `author`, `at`, and `reason`

#### Scenario: EventView for a spawn event
- **WHEN** a spawn request is issued and the event is queried
- **THEN** the payload contains `event_kind: "spawn-requested"` and `spawn_request_id`

### Requirement: SpawnRequest payload
The system SHALL return a `SpawnRequest` payload (envelope_kind: `"spawn_request"`) containing: `request_id` (prefixed `spawn:`), `request_kind` (one of `assume`, `overlook`, `guess`), `entity_id` (present for `assume` and `overlook`; absent for `guess` which targets a question, not an entity), `context` (with `clean: bool`, `prompt`, `allowed_tools[]`, `forbidden_tools[]`, `model_hint`, `max_tool_calls`), `return_to` (string identifying the originating harness, e.g. `"claude-code"`), `issued_at`, and `expires_at`. The `expires_at` SHALL be computed as `issued_at + config.harness.spawn_timeout_hours`. `allowed_tools[]` and `forbidden_tools[]` are open string arrays — parsers MUST NOT fail on unrecognised tool names.

#### Scenario: SpawnRequest for assume
- **WHEN** `dont assume claim:X --json` is run
- **THEN** the payload has `request_kind: "assume"` and `context.allowed_tools` includes `dont.dismiss`, `dont.trust`, `dont.show`, `web_search` and `context.forbidden_tools` includes `dont.conclude`, `dont.lock`, `dont.reopen`

#### Scenario: SpawnRequest for overlook
- **WHEN** `dont overlook claim:X --json` is run
- **THEN** the payload has `request_kind: "overlook"` and `context.prompt` instructs the subagent to assume the claim is wrong and try to explain why

#### Scenario: SpawnRequest for guess has no entity_id
- **WHEN** `dont guess "What is X?" --json` is run
- **THEN** the payload has `request_kind: "guess"` and `entity_id` is absent

#### Scenario: SpawnRequest carries expiry
- **WHEN** a spawn request is issued with `spawn_timeout_hours: 24`
- **THEN** `expires_at` is `issued_at` plus 24 hours

### Requirement: PrimeView payload
The system SHALL return a `PrimeView` payload (envelope_kind: `"prime"`) containing: `project` (name), `mode` (`"permissive"` or `"strict"`; parsers MUST handle unknown mode values gracefully), `status_counts` (map containing exactly the keys `unverified`, `doubted`, `verified`, `locked`, `stale`, `ignored` with integer counts), `rules` (keyed by severity `"strict"` and `"warn"` with rule name arrays), `ontologies[]` (each with `prefix` and `refreshed`), `blocking[]` (entries with `{id, statement, status}` for entities currently in `doubted` state), `pending_spawns` (count), `harness_mode` (boolean), and `invariants[]` (human-readable invariant summaries).

#### Scenario: PrimeView on fresh project
- **WHEN** `dont prime --json` is run on a just-initialized project
- **THEN** `status_counts` shows all zeros, `blocking` is empty, and `pending_spawns` is 0

#### Scenario: PrimeView reports blocking entities
- **WHEN** 3 claims are in `doubted` status
- **THEN** `blocking[]` contains 3 entries with `id`, `statement`, and `status: "doubted"`

#### Scenario: PrimeView reports harness mode
- **WHEN** `DONT_HARNESS=1` is set
- **THEN** `harness_mode` is `true`

#### Scenario: PrimeView reports active rules by severity
- **WHEN** a project has `unresolved-terms` as strict and `correlated-error` as warn
- **THEN** `rules.strict` contains `"unresolved-terms"` and `rules.warn` contains `"correlated-error"`

### Requirement: WhyView payload
The system SHALL return a `WhyView` payload (envelope_kind: `"why"`) containing: `entity` (full ClaimView or TermView), `history[]` (EventView entries, oldest first), `applicable_rules` (same structure as in ClaimView/TermView; SHALL be identical to `entity.applicable_rules` — both are present for consumer convenience), and `remediation[]` (per-unmet-rule suggestions as `{rule_name, command, description}`). On a fully-satisfied entity, `remediation` SHALL be an empty array. WhyView `remediation[]` entries are per-unmet-rule suggestions, distinct from the error-envelope `remediation[]` which exists only on refusals.

#### Scenario: WhyView with unmet lockable rule
- **WHEN** `dont why claim:X --json` is run and `lockable` is not met
- **THEN** `remediation[]` contains an entry with `rule_name: "lockable"` and a suggested command

#### Scenario: WhyView on fully satisfied entity
- **WHEN** `dont why claim:X --json` is run and all rules pass
- **THEN** `remediation` is an empty array and all `applicable_rules` show as met

#### Scenario: WhyView includes full event history
- **WHEN** a claim has 5 events in its history
- **THEN** `history[]` contains 5 EventView entries ordered oldest first

### Requirement: ClaimsList payload
The system SHALL return a `ClaimsList` payload (envelope_kind: `"claims"`) containing: `as_of` (RFC 3339 timestamp), `count` (integer), and `claims[]` (array of ClaimView, default sort order: `created_at` descending).

#### Scenario: ClaimsList with status filter
- **WHEN** `dont list --status verified --json` is run
- **THEN** the payload contains only claims with `status: "verified"` and `count` matches the array length

### Requirement: DoctorReport payload
The system SHALL return a `DoctorReport` payload (envelope_kind: `"doctor"`) containing: `cli_version`, `checks[]` (each with `name`, `status` in `{pass, warn, fail}`, and `detail`), and `summary` (counts of pass, warn, fail). Required check names include at minimum: `substrate`, `rules_compile`, `seed_snapshot`, `pending_spawns`, and `remediation_invariant`. Additional checks (e.g. `aux_linkml`, `aux_sparql`) SHALL be present when the corresponding tool is configured. Parsers MUST ignore unknown entries in `checks[]`. `dont doctor --strict` SHALL exit non-zero on any `warn` or `fail`; without `--strict`, only `fail` is non-zero.

#### Scenario: DoctorReport with all passing
- **WHEN** `dont doctor --json` is run on a healthy project
- **THEN** all `checks[]` have `status: "pass"` and `summary.fail` is 0

#### Scenario: DoctorReport strict mode on warn
- **WHEN** `dont doctor --strict` is run and one check has `status: "warn"`
- **THEN** the command exits non-zero

#### Scenario: DoctorReport reports missing auxiliary tool
- **WHEN** the `linkml` CLI is not on PATH
- **THEN** `checks[]` includes an entry with `name: "aux_linkml"` and `status: "warn"`

### Requirement: ExamplesList payload
The system SHALL return an `ExamplesList` payload (envelope_kind: `"examples"`) containing `examples[]` where each entry has `title`, `summary`, and `transcript[]` — a list of `{command, envelope}` pairs representing a worked example session.

#### Scenario: ExamplesList contains worked examples
- **WHEN** `dont examples --json` is run
- **THEN** the payload contains at least one example with title, summary, and a non-empty transcript

### Requirement: SchemaDoc payload
The system SHALL return a `SchemaDoc` payload (envelope_kind: `"schema_doc"`) containing `schema_name`, `schema_version`, and `json_schema` (a full JSON Schema document, draft 2020-12). `dont schema` with no argument SHALL return `envelope_kind: "empty"` with a `hints[]` list of available schema names.

#### Scenario: SchemaDoc for claim
- **WHEN** `dont schema claim --json` is run
- **THEN** the payload contains `schema_name: "claim"` and a valid JSON Schema document

#### Scenario: schema with no argument lists names
- **WHEN** `dont schema --json` is run with no argument
- **THEN** the response has `envelope_kind: "empty"` and `hints[]` listing available schema names

### Requirement: Applicable rules gate/flag contract
Each entry in `applicable_rules` SHALL be discriminated by a `rule_kind` field (namespaced per `dont-data-model` Kind disambiguation). Two rule kinds exist in v0.3: `"gate"` (the rule gates a transition, payload: `{rule_kind, met: bool, unmet: [string]}` where `unmet` lists failing clauses) and `"flag"` (the rule flags a condition without gating, payload: `{rule_kind, flagged: bool, detail?: string}`). New rule kinds SHALL require a minor `envelope_version` bump. Parsers MUST default-branch on unknown `rule_kind` values.

#### Scenario: gate rule that is not met
- **WHEN** a claim's `lockable` rule requires 3 hypotheses but only 1 exists
- **THEN** `applicable_rules.lockable` has `rule_kind: "gate"`, `met: false`, `unmet: ["needs >=3 hypotheses; has 1"]`

#### Scenario: flag rule that is not flagged
- **WHEN** a claim's evidence comes from independent sources
- **THEN** `applicable_rules["correlated-error"]` has `rule_kind: "flag"`, `flagged: false`

#### Scenario: unknown rule kind uses default branch
- **WHEN** a parser encounters an `applicable_rules` entry with an unknown `rule_kind`
- **THEN** it handles it through a default branch rather than failing

### Requirement: Suggest-term search behaviour
`dont suggest-term "<string>"` SHALL search (1) the local `term` table, (2) the local `imported_term` table, and (3) optionally, enabled import sources via their HTTP APIs. It SHALL return a ranked list. The tool SHALL NOT require `suggest-term` to have been run before `define` — the forcing function is soft — but the orientation block and `prime` output SHALL recommend it.

#### Scenario: suggest-term finds no matches
- **WHEN** `dont suggest-term "intrinsic curvature tensor" --json` is run on a fresh project
- **THEN** the response has `envelope_kind: "empty"`

#### Scenario: suggest-term finds a coined term
- **WHEN** `dont suggest-term "Ricci" --json` is run and `proj:RicciTensor` exists
- **THEN** the response includes `proj:RicciTensor` in the ranked results

#### Scenario: suggest-term searches imported terms
- **WHEN** `dont suggest-term "apoptosis" --json` is run and `GO:0006915` is in `imported_term`
- **THEN** the response includes `GO:0006915` in the ranked results

### Requirement: ConcludeInput schema
`ConcludeInput` SHALL accept: `statement` (required string), `atoms?` (string array), `refs?` (string array of URIs or CURIEs), `confidence?` (float 0.0-1.0), `depends_on?` (ClaimId array), `session_id?` (string), `author?` (AuthorString), `origin?` (string).

#### Scenario: conclude with atoms and refs
- **WHEN** `dont conclude "..." --atom "A" --atom "B" --ref proj:X --json` is run
- **THEN** the input is validated as `{statement, atoms: ["A","B"], refs: ["proj:X"]}`

#### Scenario: conclude with optional confidence
- **WHEN** `dont conclude "..." --confidence 0.8 --json` is run
- **THEN** the claim is created with `confidence: 0.8`

### Requirement: DefineInput schema
`DefineInput` SHALL accept: `curie` (required string), `doc` (required string), `kind_of?` (string array of CURIEs), `related_to?` (string array of CURIEs), `attribute?` (AttrSpec array), `author?` (AuthorString).

#### Scenario: define with kind_of reference
- **WHEN** `dont define proj:X --doc "..." --kind-of dont:Term --json` is run
- **THEN** the input is validated with `curie: "proj:X"` and `kind_of: ["dont:Term"]`

### Requirement: TrustInput schema
`TrustInput` SHALL accept: `entity_id` (required EntityId), `reason` (required string), `atom_idx?` (integer for doubting a single atom), `author?` (AuthorString).

#### Scenario: trust targeting a single atom
- **WHEN** `dont trust claim:X --reason "..." --atom 2 --json` is run
- **THEN** the input targets atom index 2 specifically

### Requirement: DismissInput schema
`DismissInput` SHALL accept: `entity_id` (required EntityId), `evidence` (required non-empty EvidenceSpec array where `EvidenceSpec = {uri, kind?, quote?, supports?}`), `atom_idx?` (repeatable integer array naming atoms to verify), `note?` (string), `author?` (AuthorString).

#### Scenario: dismiss with multiple atoms and evidence
- **WHEN** `dont dismiss claim:X --atom 0 --atom 1 --evidence uri1 --evidence uri2 --json` is run
- **THEN** atoms 0 and 1 are targeted and both evidence URIs are attached

### Requirement: Lifecycle verb input schemas
The system SHALL accept: `LockInput { entity_id, author? }` (entity must resolve to a claim; terms refuse), `ReopenInput { entity_id, author? }`, `IgnoreInput { entity_id, reason, author? }`, `VerifyEvidenceInput { entity_id, timeout_seconds? }`.

#### Scenario: lock refuses terms
- **WHEN** `dont lock term:X --json` is run
- **THEN** the command refuses with `wrong-entity-kind`

#### Scenario: ignore requires reason
- **WHEN** `dont ignore claim:X --json` is run without a reason
- **THEN** the command refuses because `reason` is required

### Requirement: Spawn verb input schemas
The system SHALL accept: `AssumeInput { entity_id, model_hint?, max_tool_calls? }`, `OverlookInput { entity_id, model_hint?, max_tool_calls? }`, `GuessInput { question, n?, temperature? }`.

#### Scenario: assume with model hint
- **WHEN** `dont assume claim:X --model-hint verifier --json` is run
- **THEN** the spawn request carries `context.model_hint: "verifier"`

#### Scenario: guess with multiple candidates
- **WHEN** `dont guess "What is X?" --n 3 --json` is run
- **THEN** the spawn request asks for 3 diverse candidates

### Requirement: ImportInput schema
`ImportInput` SHALL carry at minimum `source` (required string identifying the importer: `obo`, `ols`, `wikidata`, `openalex`, `bioregistry`, `jsonld`, `ttl`, `linkml`). Additional fields vary by importer and are defined per-importer (see future import spec). The system SHALL validate the `source` value against the known importer set and refuse with `code: "usage"` on unknown importers.

#### Scenario: import with known source
- **WHEN** `dont import ols GO --json` is run
- **THEN** the input is validated with `source: "ols"` and importer-specific fields

#### Scenario: import with unknown source
- **WHEN** `dont import unknown-source --json` is run
- **THEN** the command refuses with `code: "usage"`

### Requirement: EntityId and AuthorString type conventions
`EntityId` SHALL be `claim:<ULID>` or `term:<ULID>`. `AuthorString` SHALL be `<actor-kind>:<id>` as defined in the author identity format (see `dont-data-model`; this requirement defers to `dont-data-model` as the single source of truth for AuthorString and SHALL NOT duplicate its constraints). Cardinality notation: `T[]` is a possibly-empty array; `T[]` in a field marked required (no `?`) is non-empty.

#### Scenario: EntityId accepts claim prefix
- **WHEN** an input contains `entity_id: "claim:01HX05A9K8VP"`
- **THEN** validation accepts it as a valid EntityId

#### Scenario: EntityId rejects unprefixed ULID
- **WHEN** an input contains `entity_id: "01HX05A9K8VP"`
- **THEN** validation rejects it because the prefix is missing

#### Scenario: required array must be non-empty
- **WHEN** `DismissInput.evidence` is provided as an empty array
- **THEN** validation rejects it because required arrays must be non-empty

### Requirement: Forward-compatibility for payload and input shapes
Future minor versions MAY add new fields to output payload types (ClaimView, TermView, etc.); such additions SHALL be optional from the parser's perspective and MUST NOT require parser changes to remain functional. Parsers MUST ignore unknown fields. Optional input schema fields (marked `?`) MUST NOT become required except in a major version bump. All undocumented fields in payload types MUST be treated as optional by consumers.

#### Scenario: new output field does not break old parser
- **WHEN** a new minor version adds a field `tags[]` to ClaimView
- **THEN** parsers that do not recognise `tags` continue to function without error

#### Scenario: optional input field remains optional across minor versions
- **WHEN** `ConcludeInput.confidence` is optional in v0.3
- **THEN** it remains optional in v0.3.x and v0.4; making it required needs a major version bump
