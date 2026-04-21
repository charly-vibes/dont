## ADDED Requirements

### Requirement: Entity structure
All stored objects SHALL be entities. Each entity SHALL have an `id` (ULID, prefixed per kind: `claim:`, `term:`, `event:`, `evidence:`, `project:`), a `kind` attribute (`claim`, `term`, `hypothesis`, `evidence`, `project`, ...), a set of attribute assertions, and a history of events. Class membership SHALL be recognised by predicate match, not stored as a fact — "Is this entity an X?" is a rule evaluation, not a lookup. Attributes SHALL be first-class and globally defined; they are not owned by classes. The `event:`, `evidence:`, and `project:` prefixes are internal storage identifiers; only `claim:` and `term:` are externally-addressable as `EntityId` values in CLI verbs and input schemas (see `dont-payload-types`). Parsers MUST use a default branch for unknown `entity_kind` values; adding a new entity kind requires a minor-version envelope bump.

#### Scenario: entity has prefixed ULID identifier
- **WHEN** a new entity is created
- **THEN** it receives a ULID prefixed by its kind (e.g. `claim:01HX05A9K8VP`)

#### Scenario: entity kind is an attribute not a class declaration
- **WHEN** a consumer queries "Is entity X a RicciTensor?"
- **THEN** the system evaluates a recognition predicate, not a stored class-membership fact

#### Scenario: attributes are globally scoped
- **WHEN** an attribute `statement` is defined
- **THEN** it is available to any entity, not bound to a single kind

#### Scenario: unknown entity kind uses default branch
- **WHEN** a parser encounters an entity with an `entity_kind` value not in its known set
- **THEN** it handles the entity through a default branch rather than failing

#### Scenario: project entity is reserved for project-wide audit
- **WHEN** the system needs to record a project-wide event such as `mode-changed`
- **THEN** it uses a reserved internal `project:` entity rather than attaching the event to a claim or term

### Requirement: Datom-based event-sourced storage
The system SHALL store all facts as datoms — immutable atomic facts in the shape `(entity, attr, value, tx, assert_bit)` — where `tx` is a monotonically increasing transaction number and `assert_bit` distinguishes assertion from retraction. Every CLI invocation SHALL be its own transaction; there is no batching across invocations. Within a transaction, event ordering SHALL be ULID-sorted.

#### Scenario: fact is stored as a datom
- **WHEN** a claim is concluded
- **THEN** the system writes datoms with entity, attribute, value, transaction number, and assertion bit

#### Scenario: each CLI invocation is one transaction
- **WHEN** two `dont` commands run sequentially
- **THEN** each produces a distinct transaction with a higher transaction number than the previous

#### Scenario: events within a transaction are ULID-ordered
- **WHEN** a single transaction writes multiple events
- **THEN** the events are ordered by their ULID within the transaction

#### Scenario: attribute update produces assertion and retraction datoms
- **WHEN** a claim's status changes from `unverified` to `doubted`
- **THEN** a retraction datom (`assert_bit=0`) for the old status and an assertion datom (`assert_bit=1`) for the new status are written in the same transaction

### Requirement: Snapshot isolation and concurrency
The system SHALL use snapshot isolation: concurrent readers see a consistent snapshot and writers serialise. When parallel `dont` calls produce a write conflict, the system SHALL retry with exponential backoff (up to 3 attempts, 50 ms base). After the third retry exhausts, the call SHALL terminate with a non-zero exit status and emit an error envelope with `code: "db-locked"`, `unmet_clauses[]` naming the transaction that held the lock, and `remediation[]` suggesting `dont doctor` and re-running the command (see `dont-errors`).

#### Scenario: concurrent readers see consistent state
- **WHEN** two read-only `dont` commands run concurrently
- **THEN** each sees a consistent snapshot and neither blocks the other

#### Scenario: write conflict retries with backoff
- **WHEN** a write conflicts with another in-flight transaction
- **THEN** the system retries up to 3 times with exponential backoff from 50 ms base

#### Scenario: write conflict succeeds on retry
- **WHEN** a write conflicts with another transaction but the second retry succeeds
- **THEN** the command exits 0 and the transaction commits normally

#### Scenario: exhausted retries produce db-locked error
- **WHEN** all 3 retry attempts fail
- **THEN** the command exits non-zero with `code: "db-locked"`, `unmet_clauses[]` naming the blocking transaction, and `remediation[]` suggesting `dont doctor` and re-running the command

### Requirement: Signal handling
The system SHALL handle `SIGINT` and `SIGTERM` by cleanly rolling back any in-flight transaction: no partial event is committed, and the process exits with code 130 (SIGINT) or 143 (SIGTERM). Signals received during the read-only preamble (argument parsing, config load, pending-spawn expiry sweep) SHALL cause a clean exit with no store mutation. `SIGPIPE` (closed downstream reader) SHALL be treated as normal termination with exit code 0. Signals received during network I/O inside `dont verify-evidence` or `dont import` SHALL cancel the in-flight request; any rows written for URIs already resolved before the signal SHALL remain committed. `SIGKILL` cannot be intercepted; the append-only log guarantees the store is never corrupted — at worst a transaction is lost, never half-applied.

#### Scenario: SIGINT during transaction rolls back cleanly
- **WHEN** SIGINT is received during a write transaction
- **THEN** the transaction is discarded, no partial event is committed, and the process exits 130

#### Scenario: SIGPIPE is normal termination
- **WHEN** SIGPIPE is received (e.g. from `head -n 1` in a pipeline)
- **THEN** the process exits with code 0

#### Scenario: SIGTERM during transaction rolls back cleanly
- **WHEN** SIGTERM is received during a write transaction
- **THEN** the transaction is discarded, no partial event is committed, and the process exits 143

#### Scenario: signal during read-only preamble exits cleanly
- **WHEN** SIGINT is received during argument parsing or config load
- **THEN** the process exits cleanly with no store mutation

#### Scenario: signal during verify-evidence preserves completed checks
- **WHEN** SIGINT is received during `dont verify-evidence` after 3 of 5 URIs have been checked
- **THEN** the 3 completed `evidence_checked` rows remain committed and the process exits 130

#### Scenario: SIGKILL does not corrupt store
- **WHEN** SIGKILL terminates the process mid-transaction
- **THEN** the append-only log guarantees no half-applied state; at worst the in-flight transaction is lost

### Requirement: Time-travel query support
The system SHALL support reconstructing state at any past transaction by querying at a specific transaction number. Full bitemporality (transaction-time + valid-time) SHALL be available in the storage substrate for future features but the v0.3 CLI surface SHALL use transaction-time only.

#### Scenario: query at a past transaction
- **WHEN** a consumer queries state at transaction 42
- **THEN** the system returns the state as it was after transaction 42 committed

#### Scenario: valid-time is substrate-available but not CLI-exposed
- **WHEN** a consumer uses the v0.3 CLI
- **THEN** no valid-time query surface is available; every fact is stamped with wall-clock `at` only

### Requirement: Core relations
The system SHALL maintain these logical relations derived from datoms:
- `entity { id, entity_kind, created_at, created_by }`
- `attribute { entity_id, name, value, tx }`
- `event { id, entity_id, event_kind, at, author, reason?, evidence_uri?, spawn_request_id? }`
- `evidence { id, entity_id, source_uri, kind, supports, quote? }`
- `depends_on { entity_id, dep_id }`

Both `evidence` and `depends_on` SHALL key on `entity_id` (not `claim_id`) because `dismiss` targets both claims and terms and `stale-cascade` traverses term-to-term dependencies. `depends_on` rows for terms SHALL be synthesised from `kind_of`/`related_to` attributes at rule-evaluation time, not written directly by a verb; the relation's shape admits both explicit and synthesised rows.

#### Scenario: evidence keyed on entity not claim
- **WHEN** evidence is attached to a term via `dismiss`
- **THEN** the `evidence` relation stores it with `entity_id` pointing to the term

#### Scenario: depends_on supports term-to-term edges
- **WHEN** a term has a `kind_of` reference to another term
- **THEN** a `depends_on` edge exists between them for stale-cascade traversal

### Requirement: Kind disambiguation
The system SHALL namespace "kind" to avoid overloading: `entity_kind` for what an entity is (`claim`, `term`, `event`, `evidence`, `project`), `event_kind` for what happened (see canonical event kind list), `envelope_kind` for the envelope's payload-type discriminator (see `dont-envelope`), and `rule_kind` for the applicable-rules discriminator (`gate`, `flag`; see `dont-payload-types`).

#### Scenario: entity_kind distinguishes stored object types
- **WHEN** a claim entity is queried
- **THEN** its `entity_kind` is `"claim"`

#### Scenario: event_kind distinguishes event types
- **WHEN** a trust event is recorded
- **THEN** its `event_kind` is `"trusted"`

### Requirement: Canonical event kind list
The v0.3 system SHALL emit exactly these 14 event kinds: `created`, `concluded`, `defined`, `trusted`, `dismissed`, `locked`, `ignored`, `stale-cascaded`, `stale-restored`, `spawn-requested`, `spawn-resolved`, `spawn-timeout`, `evidence-checked`, `mode-changed`. All event kinds SHALL be lower-kebab-case per the naming convention in `dont-envelope`. Additions SHALL require a minor-version envelope bump and MUST be documented in the changelog. Parsers MUST treat unknown event kinds as ignorable (forward compatibility) — unknown events SHALL appear in `history[]` output but SHALL NOT cause parser failure.

#### Scenario: complete event kind set
- **WHEN** the system processes events
- **THEN** it recognises exactly the 14 canonical event kinds listed for v0.3

#### Scenario: unknown event kind is ignorable
- **WHEN** a parser encounters an event kind not in the v0.3 set
- **THEN** it treats the event as ignorable rather than failing

#### Scenario: new event kind requires envelope version bump
- **WHEN** a new event kind is introduced
- **THEN** it requires a minor-version envelope bump and changelog documentation

### Requirement: Claim-specific attributes
Claims SHALL carry these attributes: `statement` (string), `status` (lattice value per `dont-status-lifecycle`), `confidence` (float | null; null when no LLM-authored value was provided), `provenance` (structured object), `atoms[]` (repeated attribute rows, not JSON blobs), `hypotheses[]` (repeated attribute rows; each carries `{idx, text, assessment}` where `assessment = {supporting: string[], refuting: string[]}`), and `refs[]` (repeated attribute rows). `atoms[]`, `hypotheses[]`, and `refs[]` SHALL be stored as repeated attribute rows so that rules can reason over individual items. The `lockable` rule (see future rules spec) requires at least 3 assessed hypotheses to gate `lock`.

#### Scenario: atoms stored as individual rows
- **WHEN** a claim is concluded with 3 atoms
- **THEN** each atom is stored as a separate attribute row, not as a JSON blob

#### Scenario: confidence is uncalibrated
- **WHEN** a claim has `confidence: 0.78`
- **THEN** consumers treat this as the LLM's stated number, not a calibrated probability

### Requirement: Term-specific attributes
Terms SHALL carry these attributes: `curie` (string, the term's compact URI), `definition` (string, prose definition), `kind_of[]` (array of CURIEs referencing parent terms), `related_to[]` (array of CURIEs referencing related terms), `status` (lattice value per `dont-status-lifecycle`), and `provenance` (structured object). `kind_of[]` and `related_to[]` create edges traversed by `stale-cascade` and `dangling-definition` rules.

#### Scenario: term carries curie and definition
- **WHEN** a term is defined with `dont define proj:RicciTensor --doc "..."`
- **THEN** the stored term has `curie: "proj:RicciTensor"` and `definition` matching the provided text

#### Scenario: kind_of creates traversable edges
- **WHEN** a term has `kind_of: ["dont:Term"]`
- **THEN** the `stale-cascade` rule can traverse from the parent term to this term

### Requirement: Atom model
An atom SHALL be a sub-statement carrying `{idx, text, status, evidence[]}` where `status` uses a three-value sub-lattice (`unverified`, `doubted`, `verified`) — atoms cannot become `stale`, `locked`, or `ignored` (those states apply only at the entity level per `dont-status-lifecycle`). Atom-level `evidence[]` is stored but MAY be projected differently in view payloads (see `dont-payload-types` ClaimView). An atom SHALL transition to `verified` when a `dismiss` call names it via `--atom <idx>`. The `--atom` flag SHALL be repeatable on a single `dismiss` call so that one body of evidence can verify multiple atoms without issuing N separate commands.

#### Scenario: atom has its own status
- **WHEN** a claim with 3 atoms has atom 0 dismissed but atoms 1 and 2 undismissed
- **THEN** atom 0 has status `verified` while atoms 1 and 2 remain `unverified`

#### Scenario: single dismiss verifies multiple atoms
- **WHEN** `dont dismiss --atom 0 --atom 1 --evidence <uri>` is run
- **THEN** both atom 0 and atom 1 transition to `verified` with the provided evidence

### Requirement: Atom-completion gate
A claim SHALL reach `verified` in exactly one of two ways: (1) the claim has no declared atoms and a `dismiss` targets the claim with sufficient evidence, or (2) the claim has declared atoms and every declared atom has reached `verified` — the whole claim's `verified` transition is then automatic on the last atom's transition. A whole-claim `dismiss` (no `--atom` flags) on a claim that has declared atoms with any atom still `unverified` or `doubted` SHALL be refused with error code `atoms-incomplete`. A `trust` call MAY target a single atom via `--atom <idx>`; doubting any atom SHALL cascade the parent claim to `doubted`.

#### Scenario: auto-promotion on last atom verified
- **WHEN** the last unverified atom of a claim is dismissed
- **THEN** the whole claim automatically transitions to `verified` without an explicit whole-claim dismiss

#### Scenario: whole-claim dismiss refused when atoms incomplete
- **WHEN** `dont dismiss claim:X --evidence <uri>` is run without `--atom` flags and atom 1 is still `unverified`
- **THEN** the command is refused with `code: "atoms-incomplete"`

#### Scenario: doubting an atom cascades to parent claim
- **WHEN** `dont trust claim:X --atom 0 --reason "..."` is run
- **THEN** atom 0 transitions to `doubted` and the parent claim also transitions to `doubted`

### Requirement: Import relations
The system SHALL maintain import relations separate from core entities: `imported_term { curie, label, definition, xrefs, source, imported_at }`, `reference { uri, title, authors, year, source, imported_at }`, and `prefix { prefix, uri_base, canonical, imported_at }`. Imports SHALL populate reference material; they are not entities in the core store and SHALL NOT be targetable by doubt/dismiss verbs. `imported_term` and `term` SHALL be queried together when checking CURIE resolution.

#### Scenario: imported term is not targetable by trust
- **WHEN** a consumer tries to `trust` an imported term
- **THEN** the command is refused because imported terms are not core entities

#### Scenario: CURIE resolution checks both tables
- **WHEN** a claim references CURIE `GO:0008150`
- **THEN** resolution searches both the `term` table and the `imported_term` table

### Requirement: CURIE collision semantics
When the same CURIE exists in both `term` (coined) and `imported_term` (imported), the coined term SHALL shadow the imported one: the project's definition is authoritative for rule evaluation and `dismiss` decisions. Re-importing a source SHALL NOT overwrite a coined term sharing its CURIE; the import operation SHALL record a warning. To replace a coined term with an import, the workflow is: `trust <term-id>` (doubt the coined term), re-import, the coined term remains `doubted` for audit while fresh claims resolve through `imported_term`.

#### Scenario: coined term shadows imported term
- **WHEN** both `term` and `imported_term` have CURIE `proj:X`
- **THEN** rule evaluation and dismiss decisions use the coined term's definition

#### Scenario: re-import does not overwrite coined term
- **WHEN** `dont import ols GO` is re-run and a coined term shares a CURIE with an imported one
- **THEN** the coined term is unchanged and the import logs a warning

### Requirement: Five MVP primitives
The system SHALL recognise exactly five schema-level primitives and SHALL require written justification for adding a sixth:
1. **`attribute`** — `{ ident, value_type, cardinality, predicate?, doc }` where `value_type` is one of `string`, `int`, `float`, `bool`, `ref`, `tuple`, `enum`, `expr` and `predicate` is an optional Datalog fragment or reference to an external verifier tool
2. **`derived_class`** — `{ ident, defining_attributes, extra_predicates, doc }` — a named recognition query; not inheritance
3. **`enum`** — `{ ident, values[], doc }`
4. **`prefix`** — `{ prefix, uri_base, canonical }`
5. **`rule`** — `{ name, body, severity: warn|strict, doc }`

Explicitly not primitives: inheritance, mixins, `slot_usage`, identifier prefixes as a separate concept from `prefix`, class-owns-slot relationships.

#### Scenario: exactly five primitives
- **WHEN** the system's schema is queried
- **THEN** it recognises exactly five primitive kinds: attribute, derived_class, enum, prefix, rule

#### Scenario: adding a sixth primitive requires justification and version bump
- **WHEN** a contributor proposes a sixth primitive
- **THEN** the pull request must demonstrate that no combination of existing primitives expresses the capability being added, and the addition requires a minor-version bump and changelog entry

#### Scenario: derived_class is recognition not inheritance
- **WHEN** a derived class query matches an entity
- **THEN** the entity is recognised as a member; multiple derived classes may apply simultaneously

#### Scenario: inheritance is not a primitive
- **WHEN** a consumer expects class inheritance
- **THEN** the system does not provide it; membership is by predicate match only

### Requirement: Author identity format
Authors SHALL be identified by a string `<actor-kind>:<id>` where `actor-kind` is one of `human`, `llm`, `tool`, or `ci`, and `id` is an opaque stable identifier within that kind. Parsing SHALL split on the first `:` only — the `id` portion may itself contain colons. Empty `id` is invalid; `actor_kind` outside the four-value set is invalid. The `actor_kind` set is closed by design; extending it requires a minor-version bump and a parser upgrade gate. The convention is shared with `beads` and `wai` for cross-tool audit correlation.

#### Scenario: author string with colons in id
- **WHEN** author is `tool:github-actions:prod`
- **THEN** parsing yields `actor_kind = "tool"` and `id = "github-actions:prod"`

#### Scenario: empty id is rejected
- **WHEN** author is `human:`
- **THEN** validation rejects it as invalid

#### Scenario: all four valid actor kinds are accepted
- **WHEN** author strings `human:alice`, `llm:claude-3`, `tool:ci-bot`, and `ci:github-actions` are validated
- **THEN** all four are accepted

#### Scenario: unknown actor kind is rejected
- **WHEN** author is `bot:gpt-4`
- **THEN** validation rejects it because `bot` is not in the allowed set `{human, llm, tool, ci}`

### Requirement: Confidence semantics
The `confidence` field on claims SHALL be stored as authored by the LLM, uncalibrated. The system SHALL NOT attempt to Platt-scale or post-process confidences in v0.3. Consumers treating confidences as probabilities SHALL read them as the author's stated number, not as a calibrated estimate.

#### Scenario: confidence stored without transformation
- **WHEN** an LLM concludes a claim with confidence 0.78
- **THEN** the stored confidence is exactly 0.78, not a calibrated or scaled value

#### Scenario: out-of-range confidence is rejected
- **WHEN** a claim is concluded with `confidence: 1.3` (outside 0.0-1.0)
- **THEN** the command is refused with `code: "schema-mismatch"`

### Requirement: Provenance tracking
Every entity SHALL carry a `provenance` object recording at minimum `author` (AuthorString) and `origin` (string). Claims MAY additionally carry `model`, `session_id`, and `context_hash` in provenance. Every status transition SHALL record author, timestamp, and reason or evidence.

#### Scenario: claim provenance includes model
- **WHEN** a claim is concluded by `llm:claude-opus-4.7`
- **THEN** provenance includes `author`, `origin: "llm"`, and `model: "claude-opus-4.7"`

#### Scenario: every transition records author and timestamp
- **WHEN** a claim is trusted
- **THEN** the event records `author`, `at` (timestamp), and `reason`
