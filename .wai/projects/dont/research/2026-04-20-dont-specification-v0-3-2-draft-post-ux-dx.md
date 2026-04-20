# `dont` — specification (v0.3.2 draft, post-UX/DX-review patches)

**Status:** design draft, not implemented. v0.3.2 applies targeted fixes from a UX/DX evaluation (Layer 3 interface heuristics and Layer 5 Diataxis documentation) against v0.3.1 (§21 "v0.3.2 patches"). No new core features and no change to the four-verb lattice; the patch-pass closes CLI convention gaps (exit-code taxonomy, universal `--help`, signal handling, stdin composability, shell completions, short flags, `NO_COLOR`), adds documentation modalities the prior drafts under-served (a sequenced tutorial and three how-to guides), and specifies a few previously-implicit behaviours (rate-limiting and backoff in network-touching commands, prose-linting of error messages). Sections marked *[revised]* differ materially from v0.1; *[new]* are additions; *[unchanged]* carry over intact; *[patched]* received targeted edits from the v0.2 Rule-of-5 review; *[v0.3]* were revised to close substrate hedging, split the lifecycle verbs out of the four-verb core, make `dismiss` deterministic, and close load-bearing open questions; *[v0.3.1]* received the previous review-pass patches; *[v0.3.2]* received the present UX/DX-pass patches. A summary of v0.3 changes is in §21; v0.3.1 patches follow; v0.3.2 patches follow those.

---

## 1. Purpose *[revised]*

`dont` is a **forcing function** for autonomous LLM harnesses. It is a per-project command-line tool that an LLM calls directly, as a peer tool alongside harness-provided memory (`beads`) and workflow (`wai`) tools. Its single job is to interrupt the LLM's default behaviour of confidently asserting unchecked claims, and to give the LLM a structured, mechanical path to earn the right to assert. The name is imperative: `dont` — *don't* assert what you have not grounded.

The problem it addresses, distilled:

- LLMs do not reliably self-correct. Lift comes from external signal.
- Once a claim enters the context window, the autoregressive substrate makes it cheaper to defend than to retract.
- Durable knowledge-producing institutions (peer review, CI gates, the nuclear two-person rule) solve this by separating generator from evaluator and by making retraction a first-class event.

`dont` implements that separation as a minimal CLI with four verbs, an append-only event log, and a refusal protocol whose error messages *are the user interface*. When the LLM tries to assert something it has not adequately grounded, `dont` refuses with a structured remediation the LLM can act on without human involvement. When the LLM asks for verification, `dont` emits a structured instruction for the harness to spawn a clean-context subagent whose only terminal actions are `dismiss` (with evidence) or `trust` (with reason).

**The user is the LLM.** Humans may inspect `dont`'s store after the fact for audit, but they are not the target of the CLI. Human-mode output exists as a courtesy; `--json` is the product.

**The tool grows its vocabulary with use.** `dont` does not ship a domain ontology. It ships a tiny seed vocabulary (§6) and forces the LLM to coin or import every domain-specific term it uses. A term coined by the LLM is itself a claim and travels through the same doubt machinery as any other claim.

**How a harness orchestrates the trio.** `beads` owns memory (what was done, what is outstanding). `wai` owns workflow (the shape of the work: specs, phases, gates). `dont` owns epistemic discipline (what is claimed, what is grounded, what is locked). A typical turn: the harness loads `beads` context at session start, consults `wai` for the current workflow step, and invokes `dont` whenever the LLM's output would otherwise commit a claim to memory or action. The three tools do not share code, configuration, or a runtime; they share conventions (per-project directory, install-once binary, structured JSON, author-string identity) and the author-string is the shared key that lets post-hoc audit cross tool boundaries.

---

## 2. What `dont` is not *[revised]*

- Not a task tracker. That is `beads`.
- Not a workflow spec engine. That is `wai`.
- Not a general-purpose ontology editor. Protégé, LinkML, and OWL tooling exist for that; `dont` grows a project-local working vocabulary through use and supports one-way import from LinkML/OBO/etc. as a convenience.
- Not a knowledge base to be queried as a source of truth. The store exists to enforce doubt; reading it back is secondary.
- Not a bug catcher in its own right. It is a *checkpoint* at which external verifiers (SymPy, pytest, Lean, CAS sidecars) may be consulted; it owns none of them.
- Not an LLM wrapper. The tool does not call LLMs in its primary mode; it emits spawn-request payloads for the harness to execute.

---

## 3. Design principles and invariants *[v0.3]*

The v0.2 draft merged directional design principles with testable behavioural commitments into a single ten-item list. This revision separates the two because they carry different weight: principles guide design judgment but are not falsifiable; invariants are things an implementation either does or does not do, and tests can check.

### 3.1 Design principles (directional)

1. **The forcing is the product.** Every other feature — the store, the schema, the spawn protocol — is machinery in service of making refusal credible and remediation concrete.
2. **Refusals teach, not punish.** The error surface is the LLM's primary teacher: a good refusal conveys what went wrong and what to try next in one envelope. Invariant 3.2.5 gives the testable floor; this principle is the design attitude that goes beyond it (wording, ordering, and choice of remediation candidates).
3. **`dont` delegates, does not embed.** Commands that need fresh reasoning (`assume`, `overlook`, `guess`) emit spawn requests for the harness to fulfill in a clean-context subagent. Direct LLM calls are a `--direct` fallback for CI and shells without spawn support.
4. **The LLM is the user.** The CLI surface, the error messages, the orientation block, the hint system — all are written for an LLM reader first. Human-readable rendering is secondary and will not be invested in beyond correctness.
5. **Tools are independent.** `dont`, `beads`, and `wai` share conventions (install-once binary, per-project directory, agent-addressed documentation, author identity strings) but no code and no configuration. A harness invokes each independently.

### 3.2 Invariants (testable behavioural commitments)

1. **Nothing is deleted.** All state transitions append to an immutable event log. Doubt is an event, not a deletion.
2. **`conclude` and `define` are always unverified.** No flag, no import path, no privileged source bypasses this. `verified` is reached only through `dismiss` with evidence.
3. **Terms are first-class entities.** A CURIE coined through `define` enters the store as a doubtable entity, subject to the same status lattice as a claim. The vocabulary the LLM builds is itself data in the store.
4. **Imports are not claims.** Data brought in from external ontologies, registries, or papers populates separate relations and is not doubtable through the normal verbs. If an import is wrong, re-import; do not doubt.
5. **Every error envelope carries non-empty `remediation[]`.** An error without a remediation is a bug. Enforced by schema validation; checked by `dont doctor --strict` and in CI.
6. **Structured output is the contract.** Every command supports `--json` with a versioned, stable envelope (§10.2). Human output is a rendering of the same data; the two must not drift within a major `envelope_version`.

---

## 4. Architectural decisions *[revised]*

### 4.1 Language: Rust *[v0.3]*

Single-binary distribution, sub-50 ms cold start, embeddable store. Ontology-heavy work (SHACL, OWL, SPIRES-style extraction) stays out-of-process behind subprocess sidecars.

Rust in particular (over Go, which meets the single-binary and cold-start targets equally well) because (a) the rule engine and event-log materialisation benefit from zero-cost abstractions over typed algebra, (b) the `cozo` crate (§4.2) gives embedded Cozo access as a direct Rust library without FFI or subprocess overhead, and (c) the pattern-match-heavy shape of the CLI surface (envelope dispatch, refusal construction, rule-result aggregation) benefits from exhaustive-match checking. This decision is load-bearing and settled for v0.3.

**Single-binary caveat.** The core binary ships as a single static artifact. Certain import adapters require auxiliary tools installed separately: `dont import linkml` shells out to the Python `linkml` CLI (§15). `dont doctor` reports auxiliary-tool availability. Projects that do not use those importers need nothing beyond the `dont` binary itself.

### 4.2 Storage: CozoDB with event-sourced entities *[v0.3]*

The v0.2 draft deferred this choice, offering SQLite as a working assumption and leaving CozoDB as a candidate. v0.3 commits: the substrate is **CozoDB**, embedded via the `cozo` crate (minimum supported version: 0.7; APIs below that are not supported because of Validity-type and transaction-API breakage below 0.7), backed by RocksDB for on-disk persistence.

The argument for Cozo over SQLite, restored from v0.1 and reaffirmed:

- **Datoms over rows.** Events are naturally expressed as datoms — an immutable atomic fact tagged with a transaction and an assertion/retraction bit. Cozo's native triple-with-validity shape matches the event log's shape directly; in SQL this would require either an EAV table schema or repeated column-oriented tables per attribute.
- **Datalog at query time.** The §13 rules are Datalog; Cozo evaluates them natively. In a SQL substrate these would need to be either (a) hand-translated to SQL per rule, losing composability, or (b) interpreted by an in-process Datalog engine over SQL-backed facts, adding a layer. Cozo removes the layer.
- **Time-travel for free.** Reconstructing state at a past transaction (§4.2 invariant "Reconstruct state at a past time for audit") is a built-in query modifier in Cozo (`@ <tx>`), not application code.

**What v0.3 does not expose.** Cozo supports full bitemporality (transaction-time + valid-time). The v0.3 CLI surface uses **transaction-time only**: every fact is stamped with the wall-clock `at` of its event and treated as asserted from that moment forward until retracted. Valid-time remains available in the substrate for future CLI features (e.g. "this claim was true between dates A and B regardless of when we wrote it down") without a schema migration. Full bitemporal query surfaces are out of scope for v0.3 (§17).

**Required operations, all Cozo-native:**

- Append an event (one or more datoms within a transaction).
- Compute current status of any claim/term from its event history (recursive Datalog walk over `(entity, attr, value, tx, assert_bit)`).
- Query claims/terms by current status (indexed).
- Reconstruct state at a past time for audit (`@ <tx>`).

**Transaction and concurrency semantics.** Each CLI invocation is its own transaction. No batching across invocations. Cozo (RocksDB-backed) uses snapshot isolation: concurrent readers see a consistent snapshot, writers serialise. When a harness issues parallel `dont` calls in a single turn, write conflicts surface as `db-locked` errors (§10.5); the default remediation is retry with exponential backoff (up to 3 attempts, 50 ms base). After the third retry exhausts, the call terminates with a non-zero exit status and emits an `error` envelope with `code: "db-locked"`, `unmet_clauses[]` naming the transaction that held the lock, and `remediation[]` suggesting (a) `dont doctor` (to check for a stale writer) and (b) re-running the original command. The transaction is *not* retried automatically beyond the three built-in attempts. Transaction (`tx`) numbers increase monotonically in commit order. Within a transaction, event ordering is ULID-sorted. This is enough for single-session LLM harnesses; multi-user concurrent editing remains out of scope (§17).

**Signal handling.** `SIGINT` and `SIGTERM` received during a transaction cause a clean rollback: any in-flight write is discarded, no partial event is committed, and the process exits with code 130 (`128 + SIGINT`) or 143 (`128 + SIGTERM`) respectively. Because the substrate is append-only and every CLI invocation is a single transaction, there is no half-written state to recover — either the transaction committed in full before the signal, or it did not happen at all. Signals received during the read-only preamble (argument parsing, config load, pending-spawn expiry sweep) exit cleanly with no store mutation. Signals received during network I/O inside `dont verify-evidence` (§9A.4) or `dont import` (§15) cancel the in-flight request; any `evidence_checked` rows written for URIs already resolved before the signal remain committed, and the process exits 130/143. `SIGPIPE` (closed downstream reader) is treated as normal termination with exit code 0 — typical for pipelines that `head -n 1` a `--json` stream. No other signals are specially handled; `SIGKILL` is unrecoverable by definition but the append-only log guarantees the store is not corrupted — at worst a transaction is lost, never half-applied.

### 4.3 Data model: entities with kind, attributes, and history *[revised]*

All stored objects are **entities**. Each entity has:

- an `id` (ULID, prefixed per kind: `claim:`, `term:`, `event:`, `evidence:`),
- a `kind` attribute (`claim`, `term`, `hypothesis`, `evidence`, ...),
- a set of attribute assertions,
- a history of events (`created`, `concluded`, `defined`, `trusted`, `dismissed`, `locked`, `ignored`, and the rest; see §5.2 for the canonical enumeration).

Classes are not declared; membership is *recognised* by predicate match. "Is this entity a `gr:RicciTensor`?" is a rule evaluation, not a stored fact. This follows the Clojure/Datomic-style modelling discussion from the design pass: attributes are first-class and classes are queries.

The minimum-viable primitives the tool understands are described in §6; the seed vocabulary shipped with the binary is described in §7.

### 4.4 Distribution and initialisation *[patched, v0.3.1]*

Single static binary. `curl -fsSL https://.../install.sh | bash`. Per-project: `dont init [--strict]` creates `.dont/`, writes `.dont/AGENTS.md`, installs the seed vocabulary (§7), and runs `dont sync-docs` to inject the managed block into root-level agent docs.

`dont init` defaults to **permissive mode** (§8). `dont init --strict` starts the project in strict mode. Mode can be changed later via `config.toml`; the data shape is identical between modes.

**Re-invocation policy.** `dont init` on a directory that already contains `.dont/` refuses with error code `already-initialised` and `remediation[]` pointing at `dont doctor` (to inspect the existing project) and `dont config` (reserved verb, not in v0.3) for mode changes. There is no `dont init --force` in v0.3; re-initialisation would silently overwrite the seed snapshot and is deliberately unavailable. Harnesses that idempotently call `dont init` at session start are expected to tolerate `already-initialised` as a non-error.

**Author identity.** Authors are identified by a string `<actor-kind>:<id>` where `actor-kind` is one of `human`, `llm`, `tool`, or `ci`, and `id` is an opaque stable identifier within that kind: `human:sasha`, `llm:claude-opus-4.7`, `tool:github-actions`, `ci:buildkite-42`. **Parsing rule:** split on the *first* `:` only. The `id` portion is opaque and may itself contain colons (`tool:github-actions:prod` has `actor_kind = "tool"` and `id = "github-actions:prod"`). Empty `id` is invalid; `actor_kind` outside the four-value set is invalid. `dont` does not validate or enroll authors beyond this shape check; stability is the contributor's responsibility. The convention is shared with `beads` and `wai` so that post-hoc audit can correlate events across tools, but no shared registry exists and none is planned for v0.3.

**Seed snapshotting.** `dont init` snapshots the binary's embedded seed vocabulary (§7) into `.dont/seed/dont-seed.yaml`. From that moment the on-disk seed is authoritative for the project. Upgrading the `dont` binary does not migrate the project seed. Seed migration is deferred to a future `dont migrate-seed` command (§17). This closes the version-drift risk without introducing an active migration surface in v0.3.

---

## 5. Data model *[revised]*

### 5.1 Status lattice *[v0.3]*

**Serialisation convention.** In this spec's prose, statuses appear with a leading colon (`:unverified`, `:verified`) as a visual cue that they are status values, not English nouns. **In JSON envelopes and stored values the colon is omitted** — `"status": "verified"`. A single canonical form in serialisation; the colon is a prose-only convention.

```
              ┌──── trust ────┐
              ▼               │
          :doubted           (from any non-locked
              │               state except :stale)
              │ dismiss
              │ (+evidence)
              ▼
:unverified ──┴──▶ :verified ──── lock ──────────────▶ :locked (terminal)
              ▲         │          (lockable rule
   dismiss    │         │           must be met)
 (+evidence)  │         └── trust ──▶ :doubted
              │
          :stale ◀── stale-cascade ── (any non-:locked entity
              │                        whose dependency is doubted)
              │
              ├── auto ──▶ previous non-stale status
              │           (all dependencies resolve to :verified)
              │
              ├── trust ──▶ :doubted
              │            (leave the stale track
              │             on own merits)
              │
              └── reopen ──▶ :unverified
                            (manual bypass of the cascade;
                             reconsider on own merits)
```

Full transition table:

| From         | Verb / trigger             | To          | Notes                                        |
|--------------|----------------------------|-------------|----------------------------------------------|
| (none)       | `conclude` / `define`      | `:unverified` | Entry only. Strict mode refuses ungrounded. |
| `:unverified`| `trust --reason`           | `:doubted`  |                                              |
| `:unverified`| `dismiss --evidence`       | `:verified` | `unresolved-terms` rule must be clear; atom-completion gate applies (§5.2). |
| `:verified`  | `trust --reason`           | `:doubted`  |                                              |
| `:verified`  | `lock`                     | `:locked`   | `lockable` rule must be met. See §9A.1.      |
| `:doubted`   | `dismiss --evidence`       | `:verified` | Atom-completion gate applies (§5.2).         |
| `:stale`     | (dependencies clear)       | prior non-stale status | Automatic.                          |
| `:stale`     | `reopen`                   | `:unverified` | Manual bypass of the cascade. See §9A.2.   |
| `:stale`     | `trust --reason`           | `:doubted`  | Leaves the stale track on own merits.        |
| any non-locked | a dependency → `:doubted` | `:stale`   | Via `stale-cascade` rule.                    |
| any non-locked | `ignore --reason`        | `:ignored`  | Terminal escape valve. See §9A.3.            |
| `:locked`    | any                        | refused     | Terminal.                                    |
| `:ignored`   | any                        | refused     | Terminal.                                    |

Invariants:

- Claims and terms enter only via `conclude`/`define` and only `:unverified`.
- `:locked` and `:ignored` are both terminal.
- `:stale` is auto-cascade; `stale-cascade` extends across **claim→claim**, **claim→term**, and **term→term** via `kind_of` / `related_to`. When term B is doubted, every entity (claim or term) that references B cascades to `:stale`.
- Seed vocabulary terms (§7) enter `:locked` on `dont init` and require an explicit unlock operation to doubt (reserved for future use).
- Every transition records author, timestamp, and reason/evidence.

### 5.2 Core relations *[v0.3]*

Cozo-native shape (datoms stored as `(entity, attr, value, tx, assert_bit)`; the following schematic presents the logical relations derived from those datoms):

```
entity       { id, entity_kind, created_at, created_by }
attribute    { entity_id, name, value, tx }
event        { id, entity_id, event_kind, at, author,
               reason?, evidence_uri?, spawn_request_id? }
evidence     { id, entity_id, source_uri, kind, supports, quote? }
depends_on   { entity_id, dep_id }
```

**Scope of `evidence` and `depends_on`.** Both relations key on `entity_id`, not `claim_id`, because `dismiss` accepts both claims and terms as targets (§9.4) and because `stale-cascade` traverses term→term dependencies (§5.1, §13). In practice, `depends_on` rows for terms are synthesised from `kind_of` / `related_to` attributes at rule-evaluation time rather than written directly by a verb, but the relation's shape admits both. The change from v0.3's `claim_id` keying closes the contradiction between the data model and §9.4's "applies equally to claims and terms."

**Kind disambiguation.** "Kind" is namespaced in v0.3 to avoid the three-way overloading present in earlier drafts:

- **`entity_kind`** — what an entity *is*: `claim`, `term`, `evidence`.
- **`event_kind`** — what happened: see canonical list below.
- **`envelope_kind`** — the envelope's payload-type discriminator (§10.2): `claim`, `claims`, `term`, `term_list`, `event`, `events`, `spawn_request`, `spawn_requests`, `rule`, `rule_result`, `prime`, `why`, `doctor`, `examples`, `schema_doc`, `empty`, `error`.

**Canonical `event_kind` list.** No ellipsis; this is the complete set for v0.3:

```
created, concluded, defined, trusted, dismissed,
locked, ignored, stale_cascaded, stale_restored,
spawn_requested, spawn_resolved, spawn_timeout,
evidence_checked, mode_changed
```

Additions require a minor-version envelope bump and MUST be documented in the changelog. Parsers MUST treat unknown event kinds as ignorable (forward compatibility).

Claim-specific attributes: `statement`, `status`, `confidence`, `provenance`, `atoms[]`, `refs[]`.
Term-specific attributes: `curie`, `definition`, `kind_of[]`, `related_to[]`, `provenance`.

`atoms[]` and `refs[]` are stored as repeated attribute rows, not JSON blobs, so that rules can reason over individual atoms.

**Confidence.** Stored as authored by the LLM, uncalibrated. v0.3 does not attempt to Platt-scale or otherwise post-process confidences; consumers treating confidences as probabilities should read them as the author's stated number, not as a calibrated estimate. A future `dont calibrate` command may ingest historical `assume` outcomes and emit a scaling function; deferred (§17, §18).

**Atoms.** An *atom* is a sub-statement produced by decomposing a claim's top-level `statement` into independently-checkable propositions. Each atom carries `{idx, text, status, evidence[]}` where `status` mirrors the claim lattice (`:unverified` / `:doubted` / `:verified`). An atom transitions to `:verified` when a `dismiss` call names it via `--atom <idx>` (repeatable; one flag per atom index to verify) and the call's `--evidence` values are recorded against each named atom. The `--atom` flag is repeatable on a single `dismiss` call so that a single body of evidence (e.g. one authoritative paper) can verify several atoms at once without issuing N separate commands. When absent, the dismiss targets the whole claim (subject to the atom-completion gate below).

**Atom-completion gate (authoritative).** A claim as a whole reaches `:verified` in exactly one of two ways:

1. The claim has no declared atoms, and a `dismiss` call targets the claim with sufficient evidence.
2. The claim has declared atoms, and every declared atom has reached `:verified` — either through one or more `dismiss` calls that each name one or more atoms via `--atom <idx>`, or through a mix of per-atom and multi-atom calls. The whole claim's `:verified` transition is then automatic on the last atom's `:verified` transition; an explicit whole-claim `dismiss` is not required.

A whole-claim `dismiss` (no `--atom` flags) on a claim that has declared atoms with any atom still `:unverified` or `:doubted` is **refused** with error code `atoms-incomplete` (§10.5). This is the single authoritative statement of the rule; §9.4 references this section rather than duplicating.

A `trust` call may target the whole claim or a single atom via `--atom <idx>`; doubting any atom cascades the parent claim to `:doubted`. Atoms exist to surface the Chain-of-Verification pattern (§20.2, Dhuliawala et al.) at the schema layer so rules can fire at atom granularity.

### 5.3 Import relations

```
imported_term   { curie, label, definition, xrefs, source, imported_at }
reference       { uri, title, authors, year, source, imported_at }
prefix          { prefix, uri_base, canonical, imported_at }
```

Imports populate reference material; they are not entities in the core store and cannot be doubted through the normal verbs. `imported_term` and `term` (LLM-coined) are queried together when checking CURIE resolution, but they have different lifecycles.

**CURIE collision.** When the same CURIE is present in both `term` (coined) and `imported_term` (imported), the coined term *shadows* the imported one: the project's definition is authoritative for rule evaluation and `dismiss` decisions. Re-importing a source does not overwrite a coined term sharing its CURIE; the import operation records a warning in `imports/manifests/`. To replace a coined term with an import, the workflow is: `trust <term-id>` (doubting the coined term) → re-import → the coined term remains `:doubted` in the store for audit, and fresh claims resolve through `imported_term`.

---

## 6. Minimum-viable primitives *[new]*

The tool recognises exactly these concepts at the schema level. They are the vocabulary an LLM needs to participate in the forcing function; additions require written justification in the pull request that introduces them, demonstrating that no combination of existing primitives expresses the capability being added.

**`attribute`** — a named relation with a value. `{ ident, value_type, cardinality, predicate?, doc }`. `value_type` is one of `string`, `int`, `float`, `bool`, `ref`, `tuple`, `enum`, `expr`. `predicate` is a Datalog fragment or a reference to an external verifier tool. Attributes are globally defined; they are not owned by classes.

**`derived_class`** — a named recognition query. `{ ident, defining_attributes, extra_predicates, doc }`. A derived class is the set of entities matching its predicate. Not inheritance — recognition. Multiple derived classes may apply to the same entity simultaneously.

**`enum`** — a discrete value set. `{ ident, values[], doc }`.

**`prefix`** — a CURIE prefix mapping. `{ prefix, uri_base, canonical }`.

**`rule`** — a Datalog rule that fires during transitions. `{ name, body, severity: warn|strict, doc }`.

Five primitives. The v0.1 draft had a larger implied schema surface; this draft commits to these five as the upper bound and expects additions to be justified one at a time.

**Explicitly not primitives:** inheritance, mixins, `slot_usage`, identifier prefixes as a separate concept from `prefix`, class-owns-slot relationships. These are the constructs whose absence was the point of the schema redesign.

---

## 7. Seed vocabulary *[new]*

`dont init` installs a ten-term seed vocabulary with prefix `dont:`. Every seed term is `:locked` from creation. Domain projects build their vocabulary on top of this seed using `define`.

*Structural:*

- `dont:Entity` — the root of everything the store holds.
- `dont:Claim` — an assertion in the claim store.
- `dont:Term` — a coined definition in the project vocabulary.
- `dont:Evidence` — material cited in `dismiss`.

*Relations:*

- `dont:kind_of` — "X is a kind of Y" for the purposes of rule matching. Deliberately looser than `rdfs:subClassOf`; carries no OWL semantics.
- `dont:related_to` — "X is related to Y without a stronger claim."
- `dont:defined_as` — attaches a prose definition to a term.

*Epistemic:*

- `dont:Hypothesis` — a claim offered for consideration. Weaker commitment than a `:unverified` claim.
- `dont:Retraction` — the event kind recording a `trust` transition.

*External anchoring:*

- `dont:external_ref` — attaches a URI or CURIE pointing at something outside the store (paper, file, commit).

Terms explicitly **not** in the seed: `owl:Thing`, `rdfs:subClassOf`, `skos:definition` (pull in unwanted semantics); provenance/author terms (these are event attributes, not vocabulary); confidence or probability terms (too theory-laden); domain-specific terms (that is what `define` and `import` are for).

Import adapters may provide one-way mappings from `owl:Thing`/`rdfs:subClassOf`/etc. into the seed terms for projects bringing in external vocabularies.

**Seed file shape.** `.dont/seed/dont-seed.yaml` is the snapshotted-at-init form. On `dont init`, the binary writes this file and then issues one synthetic `defined` event per entry against a fresh `term` entity, followed by a `locked` event. The file is read-only thereafter (seed migration is deferred, §17). The shape is:

```yaml
version: "0.3"
prefix: dont
terms:
  - curie: dont:Entity
    doc: "The root of everything the store holds."
    kind_of: []
    related_to: []
    seed_locked: true

  - curie: dont:Claim
    doc: "An assertion in the claim store."
    kind_of: [dont:Entity]
    related_to: []
    seed_locked: true

  - curie: dont:kind_of
    doc: "'X is a kind of Y' for rule-matching purposes."
    kind_of: []
    related_to: []
    seed_locked: true

  # ...seven more entries covering Term, Evidence, related_to, defined_as,
  #    Hypothesis, Retraction, external_ref (§7 enumerates all ten).
```

`seed_locked: true` is always the case in v0.3 (all ten seed terms are `:locked` on init); the field is explicit so that future versions can ship non-locked seed entries without a schema change. Non-seed terms (LLM-coined) are not stored in YAML — they live in the event log and are queried via `dont vocab`.

---

## 8. Modes *[new]*

`dont` operates in one of two modes, set at `init` and changeable in `config.toml`. The **invariant** is the same in both modes: no `:verified` entity may reference an unresolved CURIE. The modes differ only in *when* the check runs.

### 8.1 Permissive mode (default)

- `conclude` accepts claims whose CURIEs do not resolve. The claim enters `:unverified` with an `:ungrounded-term` marker per offending CURIE listed in `warnings[]`.
- A rule (`unresolved-terms`) blocks `dismiss` until every CURIE in the claim resolves.
- `dont prime` and `dont list --status unverified` surface pending-grounding work prominently so the LLM is prompted to return and complete it.
- The LLM can make progress on a first call; it cannot shortcut the grounding requirement when it matters (at `:verified`).

### 8.2 Strict mode

- The `ungrounded` rule fires at `conclude` time, refusing the claim.
- Nothing enters the store until every CURIE resolves.
- Use case: mature projects with a settled vocabulary, or domains where even `:unverified` storage of ungrounded claims is undesirable.

### 8.3 Constraint that binds both modes

`define` **always** forbids references to undefined terms, regardless of mode. A definition with an unresolved parent or related-to reference is not a term; it is an intention to define one. This is the one place where permissive mode is not permissive: the vocabulary skeleton must be built coherently or not at all.

Rationale: if definitions could dangle, the bootstrap problem becomes unbounded — the LLM could define `A` in terms of `B`, `B` in terms of `C`, and never resolve anything. Forbidding dangling references forces each `define` call to either reference existing terms or introduce a self-contained term (no `--kind-of`, no `--related-to`).

**Cyclic definitions.** Tightly-coupled paired concepts (A `kind-of` B and B `kind-of` A) cannot be declared in a single pair of `define` calls because forward references are forbidden. The supported pattern is: (1) `define A` self-contained; (2) `define B --kind-of A`; (3) `trust A` with a reason such as "pairing with B not yet declared"; (4) redefine A via a second `define A --kind-of B` — the CURIE's history accumulates both definitions in event order, and the latest non-doubted `defined` event is current.

Expected state after step 4:

| Entity | Status | Events (in order) | Current `kind_of` |
|--------|--------|-------------------|-------------------|
| A (`term:01…A`) | `:unverified` | `created`, `defined` (self-contained), `trusted` (step 3), `defined` (with `--kind-of B`) | `[B]` (from the latest non-doubted `defined` event) |
| B (`term:01…B`) | `:unverified` | `created`, `defined` (with `--kind-of A`) | `[A]` |

The second `defined` event on A resets A's status to `:unverified` (a fresh definition is treated as a new assertion that has not yet been dismissed). Both terms now reference each other; both are `:unverified`; subsequent `dismiss --evidence` calls promote them to `:verified` independently. This is intentionally clunky: cyclic taxonomies are rarely what an LLM actually wants, and the friction is part of the forcing function.

### 8.4 Mode migration *[v0.3]*

A project in permissive mode can switch to strict at any time by editing `config.toml`. Existing `:verified` claims already satisfy strict's precondition by the invariant above. Existing `:unverified` claims with unresolved CURIEs become visible as work-to-do but do not retroactively fail. The data shape is unchanged; only the gating of *new* `conclude` calls changes.

**Mode change as event.** On `dont init` (establishing initial mode) and on every subsequent mode change detected at command start (via `config.toml` diff), the tool emits a `mode_changed` event with attributes `{old_mode, new_mode, author, at}`. The event is attached to a synthetic project-entity (`entity_kind: "project"`) so that audit can answer "when did this project get stricter?" without scanning configuration history. Mode changes are not refusable — they are recorded, not gated.

---

## 9. Primary CLI: four verbs *[v0.3]*

The core four are `conclude`, `define`, `trust`, `dismiss`. These are the verbs that implement the epistemic lattice — the moves the LLM makes to introduce claims and terms and to drive them through doubt. The **lifecycle verbs** (`lock`, `reopen`, `ignore`) that operate *around* the lattice — terminal promotion, cascade bypass, escape valve — live separately in §9A. This split was made in v0.3 after an evaluation pass flagged the previous single §9 as both non-singular ([ISO 29148] singularity) and structurally non-conforming to its own "four verbs" title.

### 9.0 A note on verb naming *[v0.3]*

Two of the four verbs invert ordinary English usage and readers should brace for it before reading the individual specifications.

- `dont trust` *doubts* an entity. Read it as "submit for trust review" — I do not trust this enough to let it stand, challenge it.
- `dont dismiss` *verifies* an entity. Read it as "dismiss the doubt" — the doubt is what is being dismissed, not the claim.

The inversion is deliberate. The tool's stance is that assertion is the suspect act and grounding is the earning. In that frame, "trusting" a claim is the move that requires justification (a non-hedge reason) and "dismissing" it is the move that requires evidence. The verbs name what the LLM is asking the system to do with its own uncertainty, not its English disposition toward the claim. Renaming to `doubt`/`affirm` was considered and rejected: `doubt` is already the tool's ambient posture (everything starts `unverified`, so a verb for it would be redundant) and `affirm` collides with "assert" — the LLM reflex the tool is built to interrupt.

The `lockable` gate, the `stale-cascade` rule, and the lifecycle verbs (§9A) all read naturally even with this inversion. `conclude` and `define` retain ordinary meanings.

### 9.1 `dont conclude`

```
dont conclude "<statement>"
        [--atom "<sub-statement>"]*
        [--ref <uri|curie>]*
        [--confidence <0..1>]
        [--depends-on <claim-id>]*
        [--session <id>]
        [--author <id>]
        [--json]
```

Creates a `claim` entity with `status = :unverified`. Records atoms, refs, dependencies. Emits a `:concluded` event. Returns a `ClaimView` (§10.3).

In strict mode: refuses if any CURIE in statement or atoms does not resolve.
In permissive mode: accepts with `warnings[]` listing unresolved CURIEs; `unresolved-terms` rule will later block `dismiss`.

**Duplicate-statement policy.** `conclude` does not deduplicate. Two `conclude` calls with identical `<statement>` create two distinct claim entities with distinct IDs. This is a deliberate choice: the LLM may legitimately re-assert a claim after reasoning about it further, and the two occurrences carry separate provenance and session metadata. An opt-in rule (`duplicate-statement`, warn-level) may be authored per project to flag near-duplicates; it does not ship by default. The responsibility for deduplication lies with the caller, typically via `dont list --status unverified` before a re-conclude.

### 9.2 `dont define` *[new verb]*

```
dont define <curie>
        --doc "<prose definition>"          # required
        [--kind-of <parent-curie>]*
        [--related-to <other-curie>]*
        [--attribute <attr-spec>]*
        [--author <id>]
        [--json]
```

Creates a `term` entity with `status = :unverified`, CURIE `<curie>`, and the given definition, parent terms, related terms, and attribute specifications. Emits a `:defined` event.

**Always** refuses if any referenced CURIE (parent, related, attribute) does not resolve. Regardless of mode.

Terms promote to `:verified` the same way claims do — `dismiss` with evidence. Evidence for a term might be a link to an authoritative source (a paper establishing the concept, a section of a specification, a reviewer's approval). Terms can be `:doubted` and `:stale`-cascaded like claims; when a term is doubted, every claim referencing it cascades to `:stale`.

**Duplicate-CURIE policy.** `define` on a CURIE that already exists in the project's `term` table is **not** a duplicate: it is a redefinition, handled as described in §8.3 (cyclic definitions). The second `define` appends a new `defined` event; the latest non-doubted event is current. If the caller's intent is to create a fresh unrelated term, they must choose a different CURIE. Collision with an `imported_term` is resolved by shadowing (§5.3).

### 9.3 `dont trust` *[v0.3]*

```
dont trust <entity-id>
        --reason "<text>"                    # required
        [--atom <idx>]
        [--author <id>]
        [--json]
```

Retracts the current status of a claim or term, asserts `:doubted`, records reason. Triggers `stale-cascade` on dependents. Refuses if the target is `:locked` or `:ignored`.

**Reason validation (verb-level, v0.3).** `trust` performs two checks at the verb level, independent of the rule system:

- `reason-required` — refuses with error code `reason-required` if `--reason` is missing or empty.
- `reason-not-hedge` — refuses with error code `reason-not-hedge` if the reason consists only of hedge patterns (`"might be wrong"`, `"not sure"`, `"probably incorrect"`, `"just a hunch"` and configurable siblings) without a specific defect. The check uses a hedge pattern list in `config.toml` under `[trust.hedges]` and can be extended per project.

These were previously expressed as a `vague-reason` rule (§13) in v0.2, but mixing a "warn-by-default" rule with a hard verb-level refusal produced the inconsistency flagged in the v0.3 evaluation. v0.3 makes the split explicit: verb-level refusals for presence and hedge-only content (deterministic, non-overridable at the rule layer), and optional rule-layer sibling checks for softer signals (see §13).

### 9.4 `dont dismiss` *[v0.3]*

```
dont dismiss <entity-id>
        --evidence <uri|curie>               # required, repeatable
        [--atom <idx>]*                      # repeatable; each named atom
                                             #   is verified by the evidence
        [--quote "<excerpt>"]
        [--note "<text>"]
        [--author <id>]
        [--json]
```

Transitions the target to `:verified`, records evidence rows, emits a `dismissed` event. Target semantics:

- No `--atom`: targets the whole claim or term. For a claim with declared atoms, the atom-completion gate (§5.2) applies and will refuse a whole-claim dismiss if any atom is not yet verified.
- One or more `--atom <idx>`: targets the named atoms. The provided `--evidence` values are recorded against each named atom. When the last remaining unverified atom transitions to `:verified`, the whole claim auto-promotes to `:verified` (§5.2). Intermediate calls that verify only a subset of atoms leave the whole claim at its current status and emit `dismissed` events on the atoms alone.

**Deterministic refusal conditions (v0.3).** `dismiss` performs only local, network-free checks. All refusals are reproducible without external I/O:

- `no-evidence` — no `--evidence` flag was supplied.
- `claim-locked` / `term-locked` — the target is `:locked`.
- `claim-ignored` / `term-ignored` — the target is `:ignored`.
- `unresolvable-uri` — an evidence value is malformed: not a valid URI (no scheme, or an ill-formed CURIE), or a CURIE whose prefix is not registered in the project's `prefix` table. This is a *shape* check, not a liveness check.
- `rule-not-met` with `unresolved-terms` — the target references CURIEs that do not resolve in either `term` or `imported_term`.
- `atoms-incomplete` — the target claim has declared atoms and a whole-claim `dismiss` was attempted while one or more atoms are still `:unverified` or `:doubted`. To clear this, issue per-atom `dismiss --atom <idx>` calls first; the whole-claim transition then happens automatically (§5.2). This refusal does not apply when `--atom <idx>` is supplied.

**Evidence liveness is a separate command.** `dismiss` does **not** hit the network. A URI that is well-formed but returns HTTP 404, times out, or is served from a dead host will still clear `dismiss`. Liveness checking is the job of `dont verify-evidence` (§9A.4), run on demand or on a schedule. This separation keeps `dismiss` deterministic and reproducible, preserves the sub-50 ms cold-start target, and makes refusals debuggable without a network.

Applies equally to claims and terms. The semantics of what counts as evidence differ (a claim's evidence is usually a paper or experiment; a term's evidence is usually a definitional authority), but the verb is the same.

---

## 9A. Lifecycle verbs *[v0.3]*

Three verbs operate *around* the four-verb lattice rather than within it: `lock` promotes a verified entity to terminal, `reopen` escapes a stale cascade, `ignore` drops an entity out of the verification game entirely. A fourth — `verify-evidence` — exists because v0.3 made `dismiss` deterministic; evidence liveness is its own concern and its own command.

These are not derived commands (they write to the event log and change status). They are lifecycle verbs: primary actions, but not part of the four-verb core. In v0.2 `lock` and `reopen` were fused into a single `reopen --lock` verb; v0.3 splits them on singularity grounds (one verb = one capability).

### 9A.1 `dont lock` *[v0.3.1]*

```
dont lock <claim-id>
        [--author <id>]
        [--json]
```

Promotes a `:verified` **claim** to `:locked` (terminal) if the `lockable` rule is met (§13: `:verified` status, ≥3 assessed hypotheses, ≥2 independent supporting evidence items). This is the only path to `:locked` for non-seed claims.

**Scope: claims only.** In v0.3.1, `lock` applies to claims but **not** to terms. The `lockable` rule requires ≥3 assessed hypotheses, and terms have no `hypotheses[]` slot in the schema. Seed terms (§7) enter `:locked` directly on `dont init`; LLM-coined terms remain revisable across the lattice (`:unverified` ↔ `:doubted` ↔ `:verified`) but cannot be terminally locked. A target argument that resolves to a term is refused with `wrong-entity-kind`; the refusal's `remediation[]` points at `dont define` (for redefinition) or at seed-level inclusion (out of scope for v0.3).

Refuses with:

- `rule-not-met` — `lockable` is not satisfied. The refusal's `unmet_clauses[]` lists every failing condition; `remediation[]` suggests the concrete next step (typically `dont overlook` to generate hypotheses, or `dont assume` to gather independent evidence).
- `claim-not-verified` — the target is not in `:verified` status. Lock only applies to `:verified`; other statuses use other verbs.
- `claim-locked` — already locked. No-op refusal for idempotency.
- `wrong-entity-kind` — the target ID resolves to a term. Lock on terms is not supported in v0.3.1.

### 9A.2 `dont reopen`

```
dont reopen <entity-id>
        [--author <id>]
        [--json]
```

Manual cascade bypass: transitions a `:stale` entity to `:unverified`, so the LLM can reconsider it on its own merits even after a dependency was doubted. Useful when the cascade is judged over-broad: the claim's content may still be sound even if a dependency is in flux.

Refuses with `claim-not-stale` / `term-not-stale` on any non-`:stale` status. The `trust`/`dismiss` path is the right tool for `:unverified` and `:doubted`; this verb exists specifically to opt out of the `stale-cascade`.

Applies to claims and terms. Does not apply to `:locked` or `:ignored` (both terminal).

### 9A.3 `dont ignore`

```
dont ignore <entity-id>
        --reason "<text>"                    # required
        [--author <id>]
        [--json]
```

Moves a claim or term to `:ignored` — a terminal state outside the verification lattice. Ignored entities are excluded from rule evaluation and from `stale-cascade`, but remain in the event log for audit and remain queryable via `dont list --status ignored`.

The primary use case is **permanently unresolvable references**: an imported ontology retires a CURIE, a cited paper is withdrawn, a dataset disappears from its host. The `:unverified` claim is then stuck — it cannot reach `:verified` via `dismiss` because `unresolved-terms` will never clear. `ignore` is the escape valve.

Secondary use: the LLM realises an early claim was misframed and wants to abandon it without polluting the active store. `ignore` is preferable to deletion (nothing is deleted; invariant 3.2.1) and clearer than leaving the claim `:doubted` indefinitely.

For cases where the content is still valid but the vocabulary drifted, the alternative workflow is to `define` a local replacement CURIE and issue a fresh `conclude` whose ref has been substituted; the original stays `:unverified` and can be `ignore`d afterward.

Refuses with `reason-required` if `--reason` is missing or empty. Subject to the same hedge check as `trust` (§9.3).

### 9A.4 `dont verify-evidence` *[new in v0.3]*

```
dont verify-evidence <entity-id>
        [--timeout <seconds>]                # default 10
        [--json]
```

Walks the evidence URIs attached to a claim or term and records their liveness as of now. For each URI: HTTPS GET (or HEAD where servers support it), status recorded; CURIE-form evidence is resolved through the prefix table to a URI and then fetched. DOI-form evidence (`doi:…`) is resolved via `https://doi.org/` with redirect-following.

Emits one `evidence_checked` event per URI with attributes `{uri, status_code, resolved, checked_at, timed_out?}`. Does **not** change the entity's status. Malformed URIs are surfaced as `evidence-malformed` warnings; liveness failures (HTTP ≥ 400) as `evidence-stale` warnings.

**Timeout behaviour.** Each URI fetch is bounded by `--timeout` (default 10 s). On timeout for a given URI: `status_code` is `null`, `resolved` is `false`, and `timed_out` is `true`; an `evidence-stale` warning is attached to the envelope naming the URI. The overall command always returns partial results — a timeout on one URI does not abort the command or skip remaining URIs. The envelope's top-level `ok` remains `true` if at least one URI's liveness was recorded; it is `false` only on a structural failure (no URIs at all, target not found). Aggregate timeout budget across a many-URI verification is not bounded by `--timeout`; callers needing a wall-clock ceiling should invoke `verify-evidence` per-target.

**Rate limiting and backoff.** `verify-evidence` is the one lifecycle verb that emits outbound network traffic. To avoid hammering hosts cited repeatedly in the store (a common case: a dozen claims citing the same paper on `doi.org` or the same section of an ontology on `ebi.ac.uk`), the client enforces: (a) at most 4 concurrent outbound requests per invocation, configurable via `config.toml` under `[verify_evidence.concurrency]`; (b) a per-host token bucket that caps sustained request rate at 2 requests/second/host with a burst of 4, resetting every 5 seconds; (c) exponential backoff on `429 Too Many Requests` and `503 Service Unavailable` responses, honouring the `Retry-After` header where present, up to 3 retries per URI before recording a `evidence-stale` warning and moving on. These defaults are conservative on purpose — `dont verify-evidence` is usually run offline or in CI, not in a tight loop — and can be tightened for users running batch verification against their own ontology servers. Requests emit a `User-Agent` header of the form `dont/<cli_version> (+https://.../dont)` so that receiving hosts can identify the tool if they need to contact the operator.

Rationale for separating this from `dismiss`: in v0.2 `dismiss` refused on "unresolvable evidence URIs," which introduced non-determinism (the same call could succeed one minute and fail the next depending on network conditions or remote-server behaviour), tied the tool's sub-50 ms cold-start target to the slowest remote host, and made refusals difficult to reason about offline. v0.3 makes `dismiss` deterministic and gives liveness its own verb. Projects that care about live references can run `dont verify-evidence` in CI; the tool reports stale evidence without failing the dismissal history.

The command is read-only with respect to status but writes events (the liveness history). It is the one lifecycle verb that hits the network.

---

## 10. Derived commands and envelope *[revised, patched]*

Orchestrations over the four primitives. In harness mode (default when `DONT_HARNESS` is set or when invoked through an LLM tool-use channel), commands that need new reasoning emit **spawn requests**; they do not call an LLM. In `--direct` mode they call the configured provider directly.

```
dont guess "<question>" [--n 3] [--temperature 0.7] [--json]
dont assume <entity-id> [--json]
dont overlook <entity-id> [--json]

dont list [--status <s>] [--as-of <ts>] [--json]
dont vocab  [--status <s>] [--as-of <ts>] [--json]
dont show <entity-id> [--history] [--json]
dont why  <entity-id> [--json]

dont suggest-term <string>       [--json]     # [new] searches imported + coined

dont rules [list | show <n> | add <file.dl> | test <n>] [--json]
dont spawns [--pending] [--json]
dont prime [--json]
dont explain <rule-name> [--json]
dont examples
dont help [<command>]
dont --version
dont doctor
dont schema [envelope|claim|term|event|spawn-request|...]
dont sync-docs
dont completions <shell>
dont import <source> ...
```

Command summaries (the self-teaching surface from v0.1 §11 carries over intact; what follows is the one-liner per command, sufficient for implementers):

- **`guess`** — spawn-request for *n* diverse candidate answers to a question, with self-consistency aggregation. The Wang et al. (§20.2) pattern. Emits `kind: "spawn-request"`; does not write to the claim store on its own.
- **`assume`** — spawn-request for clean-context independent verification of a specific claim. The Dhuliawala et al. Chain-of-Verification pattern. The subagent's only terminal actions are `dismiss` or `trust`.
- **`overlook`** — spawn-request for premortem-style adversarial challenge: the subagent assumes the claim is wrong and tries to explain why. Inspired by Klein (§20.3).
- **`list`** — lists **claims** by status / as-of. The default scope. `list --all` includes terms.
- **`vocab`** — lists **terms** by status / as-of. The narrower counterpart of `list`; equivalent to `list --kind term`.
- **`show <id>`** — full payload for one entity; `--history` includes the event timeline.
- **`why <id>`** — the claim/term plus its events plus all rules currently applicable, with remediation for any unmet.
- **`prime`** — project orientation for the LLM at session start (shape in §10.5 `PrimeView`).
- **`explain <rule>`** — prose explanation of a rule, its severity, and how to satisfy it.
- **`examples`** — canonical worked examples; see §11.2.
- **`help [<cmd>]`** — agent-addressed help; every error message also carries `remediation[]`.
- **`doctor`** — diagnostics: substrate reachable, rules compile, ontologies fresh.
- **`schema <name>`** — print the JSON Schema for one envelope or payload type.
- **`schema`** (no arg) — list schema names.
- **`rules`** — list / show / add / test rule files.
- **`spawns --pending`** — outstanding spawn requests and their age.
- **`sync-docs`** — rewrite the managed block in `AGENTS.md` / `CLAUDE.md` / etc.
- **`completions <shell>`** — print shell-completion script for `bash`, `zsh`, `fish`, `powershell`, or `elvish` (§10.7.5).
- **`import <source>`** — see §15.

### 10.1 `dont suggest-term` *[new]*

Before coining a new term, the LLM should run `dont suggest-term "<rough concept>"`. The command searches:

1. The local `term` table (terms already coined in this project).
2. The local `imported_term` table (terms imported from OLS, LinkML schemas, etc.).
3. Optionally, enabled import sources via their HTTP APIs.

Returns a ranked list. The LLM is then expected to either pick an existing term or, if nothing matches, proceed with `define`. The forcing function here is soft — the tool does not require `suggest-term` to have been run before `define` — but the orientation block and the `prime` output recommend it, and a future rule (`unsearched-before-coining`, opt-in) could enforce it for projects that want stronger vocabulary hygiene.

### 10.2 Envelope *[v0.3]*

All machine-parseable output across the CLI and MCP surfaces follows the same envelope. This section is the contract.

```json
{
  "envelope_version": "0.2",
  "cli_version": "0.3.0",
  "ok": true,
  "envelope_kind": "claim",
  "data": { },
  "hints": [
    { "command": "dont assume claim:01HX05", "description": "spawn independent verification" }
  ],
  "warnings": [],
  "meta": {
    "duration_ms": 14,
    "tx": 82,
    "request_id": null
  }
}
```

Fields:

- `envelope_version` — the envelope *schema* version, independent of the CLI's own version. Stable within a major; minor versions may add fields but MUST NOT remove or rename. Starts at `"0.2"` in v0.3 because the v0.2 spec already committed to the envelope shape; this version field was the stale `dont_version: "0.1"` in the v0.2 draft and is renamed and reset here to close [COHR-3.2].
- `cli_version` — the CLI binary's semver. Independent of the envelope schema. Present for troubleshooting; parsers should not branch on it.
- `ok` — `true` for success, `false` for refusal or error. On `false`, `data` is the `ErrorResult` shape (§10.5) and `envelope_kind` is `"error"`.
- `envelope_kind` — discriminator for `data`. Canonical list for envelope_version 0.2:

  ```
  claim, claims, term, term_list, event, events,
  spawn_request, spawn_requests, rule, rule_result,
  prime, why, doctor, examples, schema_doc,
  empty, error
  ```

  Parsers MUST have a default branch for unknown values (forward compatibility; new payload types in future minor versions will not break old parsers).

- `data` — the typed payload.
- `hints` — ordered array of `{command, description}`. Safe to ignore; useful for agents. Not carried on errors (use `remediation[]` inside `ErrorResult`).
- `warnings` — rule flags triggered during the operation, `{rule_name, entity_id?, message, suggested_remediation?}`. Non-refusing rule hits, malformed-but-non-blocking inputs, liveness stale signals.
- `meta` — execution metadata. `tx` is the transaction id that applied any mutations (`null` for read-only commands). `request_id` is populated when the command resolves a pending spawn.

In `--json` mode the envelope is the only thing on stdout. Human logging goes to stderr.

### 10.3 Identity and format conventions

- **Claim IDs** are ULIDs prefixed `claim:` — `claim:01HX05A9…`. Lexicographically sortable, timestamp-embedded.
- **Spawn request IDs** prefixed `spawn:`.
- **Rule names**, **event kinds**, **status values**: lower-kebab-case strings.
- **Timestamps**: RFC 3339 in UTC — `2026-04-18T14:20:00Z`.
- **Validity**: `[timestamp, assertion_bool]`.

### 10.4 Core payload types *[v0.3]*

All examples below use v0.3 serialisation conventions: no colons on status values; `status_counts` not `state` in `PrimeView`; `rule_name` not `rule` in refs; `entity_kind` where disambiguation matters.

**`ClaimView`** (`envelope_kind: "claim"`):

```json
{
  "id": "claim:01HX05A9K8VP",
  "entity_kind": "claim",
  "statement": "CRISPR-Cas9 causes off-target edits in human cells",
  "status": "verified",
  "confidence": 0.78,
  "atoms": [
    {"idx": 0, "text": "CRISPR edits DNA", "status": "verified"},
    {"idx": 1, "text": "off-target edits have been observed", "status": "verified"}
  ],
  "hypotheses": [
    {"idx": 0, "text": "...", "assessment": {"supporting": [], "refuting": []}}
  ],
  "evidence": [
    {"source_uri": "doi:10.1126/science.abc1234", "kind": "paper",
     "supports": true, "quote": null}
  ],
  "depends_on": ["claim:01HWZ3…"],
  "provenance": {
    "author": "human:sasha",
    "origin": "llm",
    "model": "claude-opus-4.7",
    "session_id": "s-84c…",
    "context_hash": "sha256:…"
  },
  "created_at": "2026-04-15T09:12:00Z",
  "updated_at": "2026-04-16T14:20:00Z",
  "applicable_rules": {
    "lockable":         {"kind": "gate", "met": false, "unmet": ["needs >=3 hypotheses; has 1"]},
    "correlated-error": {"kind": "flag", "flagged": false},
    "ungrounded":       {"kind": "flag", "flagged": false}
  }
}
```

Notes: atom `status` is a lattice value (`unverified` / `doubted` / `verified`), matching the claim's own `status` field. The prior `verified: true` boolean shorthand was dropped in v0.3 for consistency; atoms can be `doubted` independently of the parent.

The `applicable_rules` values are discriminated by a `kind` field. Two kinds exist in v0.3.1:

- `"kind": "gate"` — the rule gates a transition (`lockable` gates `lock`; `unresolved-terms` gates `dismiss`). Payload: `{kind: "gate", met: bool, unmet: [string]}` where `unmet` is a (possibly empty) list of failing clauses, human-readable.
- `"kind": "flag"` — the rule flags a condition without gating any transition (`correlated-error`, `ungrounded` in permissive mode). Payload: `{kind: "flag", flagged: bool, detail?: string}`.

New rule kinds require a minor envelope-version bump. Parsers MUST default-branch on unknown `kind` values.

**`TermView`** (`envelope_kind: "term"`):

```json
{
  "id": "term:01HX07…",
  "entity_kind": "term",
  "curie": "proj:RicciTensor",
  "definition": "A symmetric (0,2) tensor encoding intrinsic curvature of a Riemannian manifold.",
  "kind_of": ["dont:Term"],
  "related_to": ["proj:MetricTensor"],
  "status": "unverified",
  "confidence": null,
  "provenance": { "author": "llm:claude-opus-4.7", "origin": "llm" },
  "created_at": "2026-04-18T09:00:00Z",
  "applicable_rules": {
    "unresolved-terms": {"kind": "gate", "met": true, "unmet": []},
    "dangling-definition": {"kind": "gate", "met": true, "unmet": []}
  }
}
```

**`EventView`** (`envelope_kind: "event"`):

```json
{
  "entity_id": "claim:01HX05…",
  "tx": 42,
  "event_kind": "trusted",
  "at": "2026-04-15T09:34:00Z",
  "author": "human:sasha",
  "reason": "atom 3 conflates correlation with causation",
  "evidence_uri": null,
  "spawn_request_id": null
}
```

**`SpawnRequest`** (`envelope_kind: "spawn_request"`): the structured subagent instruction (§12).

```json
{
  "request_id": "spawn:01HX0A3K8P",
  "request_kind": "assume",
  "entity_id": "claim:01HX05…",
  "context": {
    "clean": true,
    "prompt": "You are an independent verifier…",
    "allowed_tools": ["dont.dismiss", "dont.trust", "dont.show", "web_search"],
    "forbidden_tools": ["dont.conclude", "dont.lock", "dont.reopen"],
    "model_hint": "verifier",
    "max_tool_calls": 20
  },
  "return_to": "claude-code",
  "issued_at": "2026-04-18T14:20:00Z",
  "expires_at": "2026-04-19T14:20:00Z"
}
```

`request_kind` is one of `assume`, `overlook`, `guess`. `expires_at` is computed as `issued_at + config.harness.spawn_timeout_hours`; on expiry the spawn surfaces via `spawn_timeout` events (§12).

**`PrimeView`** (`envelope_kind: "prime"`):

```json
{
  "project": "research-demo",
  "mode": "permissive",
  "status_counts": {
    "unverified": 23, "doubted": 11, "verified": 10,
    "locked": 3, "stale": 0, "ignored": 2
  },
  "rules": {"strict": ["lockable", "ungrounded"], "warn": ["correlated-error"]},
  "ontologies": [
    {"prefix": "GO", "refreshed": "2026-04-15T00:00:00Z"},
    {"prefix": "HPO", "refreshed": "2026-04-15T00:00:00Z"}
  ],
  "blocking": [
    {"id": "claim:01HWY9…", "statement": "…", "status": "doubted"}
  ],
  "pending_spawns": 2,
  "harness_mode": true,
  "invariants": [
    "conclude always creates unverified",
    "dismiss requires --evidence and is deterministic",
    "locked and ignored are terminal"
  ]
}
```

**`WhyView`** (`envelope_kind: "why"`): claim or term, full event timeline, applicable rules, and remediation for every unmet rule. Used by `dont why <id>`.

```json
{
  "entity": { /* ClaimView or TermView, as appropriate */ },
  "history": [ /* EventView, oldest first */ ],
  "applicable_rules": {
    "lockable": {"kind": "gate", "met": false,
                 "unmet": ["needs >=3 hypotheses; has 1",
                          "needs >=2 independent supporting evidence items; has 1"]},
    "correlated-error": {"kind": "flag", "flagged": false}
  },
  "remediation": [
    {"rule_name": "lockable",
     "command": "dont overlook claim:01HX05",
     "description": "spawn premortem to generate and assess 2 more hypotheses"},
    {"rule_name": "lockable",
     "command": "dont assume claim:01HX05",
     "description": "spawn independent verification to add a second evidence item"}
  ]
}
```

`remediation[]` entries at the `WhyView` top level are **per-unmet-rule** suggestions, distinct from the error-envelope `remediation[]` (§10.5) which exists only on refusals. On a fully-satisfied claim or term, `why`'s `remediation` is an empty array and `applicable_rules` shows all rules passing.

**`ClaimsList`** (`envelope_kind: "claims"`):

```json
{
  "as_of": "2026-04-18T14:20:00Z",
  "count": 47,
  "claims": [ /* ClaimView */ ]
}
```

**`DoctorReport`** (`envelope_kind: "doctor"`): output of `dont doctor`. Checks that the binary is healthy, the substrate is reachable, rules compile, auxiliary tools for import are available, and pending spawns are not past their expiry.

```json
{
  "cli_version": "0.3.0",
  "checks": [
    {"name": "substrate",      "status": "pass", "detail": "cozo 0.7, 124 MB, 3.4k events"},
    {"name": "rules_compile",  "status": "pass", "detail": "7/7 shipped, 0 user"},
    {"name": "seed_snapshot",  "status": "pass", "detail": ".dont/seed/dont-seed.yaml present"},
    {"name": "aux_linkml",     "status": "warn", "detail": "linkml CLI not found; dont import linkml will fail"},
    {"name": "aux_sparql",     "status": "pass", "detail": "reachable"},
    {"name": "pending_spawns", "status": "pass", "detail": "2 pending, oldest 4h"},
    {"name": "remediation_invariant", "status": "pass", "detail": "all error schemas require remediation[]"}
  ],
  "summary": {"pass": 6, "warn": 1, "fail": 0}
}
```

Each check has `status` in `{pass, warn, fail}`. `dont doctor --strict` exits non-zero on any `warn` or `fail`; without `--strict`, only `fail` is non-zero.

**`ExamplesList`** (`envelope_kind: "examples"`): output of `dont examples`. A curated list of worked example transcripts bundled with the binary; each entry is a `{title, summary, transcript[]}` record where `transcript[]` is a list of `{command, envelope}` pairs.

```json
{
  "examples": [
    {
      "title": "Seven-call productive session",
      "summary": "define → conclude → spawn-verify → dismiss → lock",
      "transcript": [ /* {command, envelope}, ... */ ]
    }
  ]
}
```

The canonical transcript for the above is §11.2.

**`SchemaDoc`** (`envelope_kind: "schema_doc"`): output of `dont schema <n>`. A JSON Schema document (draft 2020-12) for one envelope or payload type.

```json
{
  "schema_name": "claim",
  "schema_version": "0.2",
  "json_schema": { /* a full JSON Schema document */ }
}
```

`dont schema` with no argument returns `envelope_kind: "empty"` with a `hints[]` list of available schema names.

Full JSON Schemas for each type are shipped with the binary; `dont schema <n>` prints them.


### 10.5 Error envelope *[v0.3]*

On refusal or error: `ok: false`, `envelope_kind: "error"`, `data` takes the `ErrorResult` shape:

```json
{
  "code": "no-evidence",
  "message": "dismiss requires at least one --evidence URI",
  "rule_name": null,
  "spec_ref": "§9.4",
  "entity_id": "claim:01HX05…",
  "unmet_clauses": [
    {"clause": "at least one --evidence flag", "fix": "--evidence <uri>"}
  ],
  "remediation": [
    {"command": "dont assume claim:01HX05",
     "description": "spawn clean-context verification to surface evidence"},
    {"command": "dont dismiss claim:01HX05 --evidence <uri>",
     "description": "supply evidence directly"}
  ]
}
```

Fields:

- `code` — stable lowercase-kebab error identifier. The set is open; see below.
- `message` — human-readable one-liner.
- `rule_name` — the §13 rule that refused, or `null` for verb-level validators that are not rules (`no-evidence`, `reason-required`, `reason-not-hedge`, `atoms-incomplete`, etc.). Closes the v0.2 overloading of this field identified in the evaluation pass.
- `spec_ref` — pointer to the spec section governing the refusal. Useful for debugging; not a substitute for `rule_name`.
- `entity_id` — the target of the refused operation, when applicable.
- `unmet_clauses[]` — structured list of failing conditions, each with a `fix` snippet.
- `remediation[]` — **required, non-empty array** of `{command, description}` pairs. Invariant 3.2.5 commits the tool to this: an error without remediation is a bug. `dont doctor --strict` verifies the schema enforces this.

**Error-code completeness.** Known v0.3.2 codes include: `no-evidence`, `reason-required`, `reason-not-hedge`, `atoms-incomplete`, `claim-not-found`, `term-not-found`, `claim-locked`, `term-locked`, `claim-ignored`, `term-ignored`, `claim-not-verified`, `term-not-verified`, `claim-not-stale`, `term-not-stale`, `rule-not-met`, `wrong-entity-kind`, `already-initialised`, `unresolvable-uri`, `schema-mismatch`, `db-locked`, `config-missing`, `spawn-not-found`, `spawn-expired`, `linkml-unsupported-feature`, `usage`, `internal`. Warning codes (attached to `warnings[]`, not to `ok: false`): `evidence-malformed`, `evidence-stale`. The `usage` code is emitted on malformed CLI arguments (exit `2`, §10.7.1); `internal` on unexpected tool failure (exit `4`). Both carry `remediation[]` — `usage` pointing at `dont help <cmd>`, `internal` pointing at `dont doctor` and at issue-reporting.

**Scope of `rule-not-met`.** This code is the generic refusal for any failure originating from the §13 rule system (`lockable`, `unresolved-terms`, `ungrounded` in strict mode, `dangling-definition`, `correlated-error` if promoted to strict per-project). Readers must consult the `rule_name` field to identify the specific rule that refused. Verb-level validators that are not rules (`no-evidence`, `reason-required`, `reason-not-hedge`, `atoms-incomplete`, `wrong-entity-kind`) have their own dedicated codes and do not use `rule-not-met`; their `rule_name` is `null`.

The set is **open** — new codes may be added in minor envelope versions. Parsers MUST have a default branch for unknown codes and MUST NOT fail-closed on them. New codes are not a breaking change within a major `envelope_version`.

### 10.6 Input schemas *[v0.3]*

CLI args map 1:1 to structured payloads. In MCP mode, tools accept the payload directly. Both are validated against the same JSON Schema.

```
ConcludeInput       = {statement: string,
                       atoms?: string[],
                       refs?: string[],             # URIs or CURIEs
                       confidence?: float,          # 0.0–1.0
                       depends_on?: ClaimId[],
                       session_id?: string,
                       author?: AuthorString,
                       origin?: string}

DefineInput         = {curie: string,
                       doc: string,
                       kind_of?: string[],          # CURIEs
                       related_to?: string[],       # CURIEs
                       attribute?: AttrSpec[],
                       author?: AuthorString}

TrustInput          = {entity_id: EntityId,
                       reason: string,
                       atom_idx?: int,              # single; doubt one atom
                       author?: AuthorString}

DismissInput        = {entity_id: EntityId,
                       evidence: EvidenceSpec[],    # non-empty
                       atom_idx?: int[],            # repeatable; names atoms
                                                    #   to verify in this call
                       note?: string,
                       author?: AuthorString}
# where EvidenceSpec = {uri: string, kind?: string,
#                       quote?: string, supports?: bool}

LockInput           = {entity_id: EntityId, author?: AuthorString}
                      # v0.3.1: entity_id must resolve to a claim; terms refuse
ReopenInput         = {entity_id: EntityId, author?: AuthorString}
IgnoreInput         = {entity_id: EntityId, reason: string, author?: AuthorString}
VerifyEvidenceInput = {entity_id: EntityId, timeout_seconds?: int}

AssumeInput         = {entity_id: EntityId, model_hint?: string, max_tool_calls?: int}
OverlookInput       = {entity_id: EntityId, model_hint?: string, max_tool_calls?: int}
GuessInput          = {question: string, n?: int, temperature?: float}
ImportInput         = {source: string, ...}        # varies by importer
```

Cardinality notation: `T[]` is a possibly-empty array; `T[]` in a field marked `required` (no `?`) is non-empty. `AuthorString` is the `<actor-kind>:<id>` shape specified in §4.4. `EntityId` is `claim:<ULID>` or `term:<ULID>`.

The field name `entity_id` replaces v0.2's `claim_id` in every non-claim-specific input, closing the ambiguity where terms are also valid targets for `trust`, `dismiss`, `lock`, `reopen`, and `ignore`.

### 10.7 CLI conventions *[v0.3.2]*

The prior drafts specified the JSON envelope (§10.2–§10.6) thoroughly but left the shell-facing surface under-specified. This subsection is the contract for everything the shell sees: exit codes, universal flags, short flags, stdin behaviour, colour, and shell completions. All of the following applies uniformly across every `dont` subcommand unless a subcommand's own section explicitly overrides.

#### 10.7.1 Exit codes

Every `dont` invocation exits with one of the following codes. Harnesses scripting around `dont` can branch on the exit code alone without parsing the envelope, though the envelope always carries richer detail when emitted.

| Exit | Meaning | Envelope | Typical cause |
|------|---------|----------|---------------|
| `0` | Success | `ok: true` (or none on `--version` / completions) | Command completed as intended. |
| `1` | Refusal | `ok: false`, `envelope_kind: "error"` | A rule refused, a verb-level validator tripped, or `dont doctor --strict` found a `warn`/`fail` check. This is a *legitimate outcome* — the LLM is expected to read `remediation[0].command` and retry. |
| `2` | Usage error | `ok: false`, `envelope_kind: "error"`, `code: "usage"` | Malformed arguments, unknown flag, unknown subcommand. The error envelope is emitted but `remediation[]` points at `dont help <cmd>` rather than a domain action. |
| `3` | Substrate or configuration error | `ok: false`, `envelope_kind: "error"`, `code` in {`db-locked`, `config-missing`, `schema-mismatch`, `spawn-not-found`, `spawn-expired`, `linkml-unsupported-feature`, `already-initialised`} | The store could not be reached, configuration is incomplete, or an auxiliary tool is missing. Harness-recoverable in principle but not by retrying the same command without operator action. |
| `4` | Internal error | `ok: false`, `envelope_kind: "error"`, `code: "internal"` | A bug. The envelope's `message` includes a short diagnostic and a hint to file an issue. Should not happen in normal operation; `dont doctor` may help. |
| `130` | Interrupted (`SIGINT`) | none | User pressed Ctrl-C; see §4.2 signal handling. |
| `143` | Terminated (`SIGTERM`) | none | Killed by supervisor; see §4.2. |

**Rationale.** The boundary between `1` (refusal) and `3` (substrate) is the pivot a harness most needs: exit `1` means "the LLM can make progress by reading `remediation[]`"; exit `3` means "stop asking the LLM, check configuration or re-run `dont doctor`." Exit `2` is distinguished from `1` because usage errors are not domain events; they should not be surfaced to the LLM-user as refusals to act on.

**Doctor-mode exception.** `dont doctor --strict` exits `1` when any check is `warn` or `fail`. This preserves the convention "exit `1` = something the caller should attend to, envelope says what." Without `--strict`, exit `1` occurs only on `fail` checks.

#### 10.7.2 Universal flags

Every subcommand accepts the following flags. They are parsed before subcommand-specific flags and never conflict with them.

| Flag | Short | Effect |
|------|-------|--------|
| `--help` | `-h` | Print the same content as `dont help <this-cmd>` and exit `0`. Always routes to stdout. Present on every subcommand and on bare `dont`. |
| `--version` | | Print `cli_version` and `envelope_version` in a single line (`--json` prints a minimal `envelope_kind: "version"` record) and exit `0`. |
| `--json` | `-j` | Emit the structured envelope on stdout (§10.2) in place of human rendering. Human logging moves to stderr. |
| `--plain` | | Force uncoloured, unformatted human output. Useful in logs, CI, and pipes. Mutually exclusive with `--json`; if both are set, `--json` wins. |
| `--author <id>` | `-a` | Override the author string for this invocation (§4.4); default is derived from `$DONT_AUTHOR` or `$USER`. |
| `--direct` | | Direct-mode opt-out from harness detection (§12.1). |

**Short-flag conflicts.** Subcommand-specific short flags are chosen to not collide with the universal set above. The only widely reused short flag per-command is `-r` for `--reason` (on `trust`, `ignore`) and `-e` for `--evidence` (on `dismiss`). The full list is emitted by `dont help <cmd>` and by the shell-completion generator.

#### 10.7.3 Colour and terminal awareness

Human-mode output is coloured by default when stdout is a terminal. The tool honours the [`NO_COLOR`](https://no-color.org) convention: if `NO_COLOR` is set to any non-empty value, or if stdout is not a terminal, or if `--plain` is passed, all output is uncoloured. This decision is independent of `--json` (which is uncoloured by definition).

`CLICOLOR_FORCE=1` forces colour even when stdout is redirected; it exists for CI log collectors that preserve ANSI sequences intentionally.

#### 10.7.4 Stdin input for bulk operations

Several verbs accept entity IDs from stdin when the ID argument is given as `-`. This is the upstream half of the pipeline the JSON envelope already provides on stdout; it lets a harness compose multi-command workflows without shell loops.

```
dont list --status doubted --json \
  | jq -r '.data.claims[].id' \
  | dont show - --json
```

Commands that accept `<entity-id>` as `-` read one ID per line from stdin and invoke the command once per line. Each invocation emits its own envelope on stdout (one per line when `--json` is set, i.e. NDJSON). Empty lines are skipped; invalid IDs produce an error envelope for that line and processing continues. The exit code after a stdin run is:

- `0` if every line succeeded;
- `1` if any line was refused (`ok: false` with domain code) but none errored structurally;
- `2` or higher if a structural error occurred on any line (usage, substrate, internal).

Verbs that accept stdin in v0.3.2: `show`, `why`, `trust` (requires `--reason`), `dismiss` (requires `--evidence`), `ignore` (requires `--reason`), `lock`, `reopen`, `verify-evidence`. Verbs that do *not* accept stdin: `conclude`, `define` (each takes domain content, not just an ID). `dont list` and `dont vocab` ignore stdin; they are sources, not sinks.

#### 10.7.5 Shell completion

`dont completions <shell>` prints a shell-completion script to stdout for the named shell. Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`. The output is the shape each shell expects (e.g. bash's `complete -F` script, zsh's `#compdef` file, fish's `complete -c` form). Installation:

```
# bash
dont completions bash > /etc/bash_completion.d/dont
# zsh
dont completions zsh > "${fpath[1]}/_dont"
# fish
dont completions fish > ~/.config/fish/completions/dont.fish
```

Completions cover: subcommands, universal flags, subcommand flags, enum-valued flags (statuses, modes, shell names), and — where cheap — dynamic completion of entity IDs by querying the local store for recent IDs matching the current prefix. Dynamic completion is gated on a fast path (≤ 10 ms budget); completions degrade to static on slow stores rather than blocking the prompt.

#### 10.7.6 Help surface

`dont help` and `dont help <cmd>` are the primary agent-addressed help surface (§11). `--help` on any subcommand routes to the same content as `dont help <cmd>` for that command. The tutorial at §11.3, the how-to guides at §11.4, and the reference at §§5, 9, 9A, 10, 13, 14 are the three Diataxis modalities the CLI points at. `dont help --tutorial` prints the §11.3 walkthrough; `dont help --howto <topic>` prints a §11.4 guide. `dont help --topics` lists both.

---

## 11. Self-teaching and harness-integration surface *[mostly unchanged]*

The self-teaching surface from v0.1 (three-part errors, `dont prime`, `dont why`, `dont explain`, `dont examples`, `dont ignore`, contextual hints, self-correcting refusals, agent-addressed docs, `dont help`/`dont doctor`/`dont schema`) carries over with two changes:

- Every reference to "a claim" or "the claim" applies equally to terms.
- `.dont/AGENTS.md` is now primary; root-level managed blocks remain but are shorter, because the LLM reads `dont prime --json` at session start.

The managed block template (updated for v0.3):

```markdown
<!-- dont-managed:start -->
## Claim and term management — `dont`

This project uses `dont` to track knowledge claims and coined terms with enforced doubt.

- Run `dont prime --json` at session start.
- Full usage: [`.dont/AGENTS.md`](./.dont/AGENTS.md) or `dont help`.
- Core four verbs: `dont conclude`, `dont define`, `dont trust`, `dont dismiss`.
- Lifecycle verbs: `dont lock`, `dont reopen`, `dont ignore`, `dont verify-evidence`.
- `assume` / `overlook` / `guess` emit spawn requests.
- Prefer `--json` for parsing.

This block is managed by `dont`. Edits inside the markers will be overwritten on `dont sync-docs`.
<!-- dont-managed:end -->
```

The spawn-request protocol is reframed in §12.

### 11.1 Orientation block for the LLM *[v0.3]*

The minimum-viable orientation prompt, updated for v0.3:

```
You have access to `dont`, a tool that forces grounding before you
can assert claims. You are the user; always pass --json.

Core four verbs:
  dont.conclude — introduce a claim (one declarative sentence,
                  decomposed into atoms). Enters unverified.
  dont.define   — coin a vocabulary term. Requires a prose definition.
                  Parent/related references must already resolve.
  dont.trust    — challenge a claim or term with a specific,
                  non-hedge reason.
  dont.dismiss  — verify a claim or term with at least one
                  well-formed evidence URI. Deterministic and
                  network-free.

Lifecycle verbs (around the lattice):
  dont.lock           — promote verified → locked (terminal) when
                        the lockable rule is met.
  dont.reopen         — escape a stale cascade: stale → unverified.
  dont.ignore         — drop a permanently unresolvable entity out
                        of the verification game (terminal).
  dont.verify-evidence — check liveness of evidence URIs. Separate
                        from dismiss; this is the only verb that
                        hits the network.

Mode: permissive (default) or strict.
  - Permissive: `conclude` accepts unresolved CURIEs with warnings;
    `dismiss` requires them resolved.
  - Strict: `conclude` refuses unresolved CURIEs up front.
  - `define` forbids unresolved references in both modes.

On refusal:
  Read data.remediation[0].command and run it. Then retry.
  Do not guess reformulations. Do not apologise.

On a spawn-request envelope:
  Invoke your harness's subagent mechanism with context.prompt and
  allowed_tools. Do not perform the verification yourself.

Before coining a new term: run `dont suggest-term "<concept>"` first.

For full docs: `dont help <cmd>` or `.dont/AGENTS.md`.
For a sequenced first-session walkthrough: `dont help --tutorial` (§11.3).
For goal-oriented recipes: `dont help --howto <topic>` (§11.4).
```

### 11.2 Worked example *[new, patched]*

A minimal end-to-end trace showing the seven calls a typical session makes. Each call shows the invocation, the envelope kind returned, and the relevant state change. Errors and retries elided for brevity.

**Setup.** Fresh project, permissive mode, `beads` and `wai` also present.

**Call 1 — Prime.**
```
$ dont prime --json
```
Returns `envelope_kind: "prime"` with zero-state `PrimeView`. The LLM now knows the project mode, what rules are active, and what (nothing, at this point) is blocking.

**Call 2 — Search vocabulary before coining.**
```
$ dont suggest-term "intrinsic curvature tensor" --json
```
Returns `envelope_kind: "empty"` (no matches in coined `term`, no matches in `imported_term`). The LLM is free to coin.

**Call 3 — Define a term.**
```
$ dont define proj:RicciTensor \
    --doc "A symmetric (0,2) tensor encoding intrinsic curvature of a Riemannian manifold." \
    --kind-of dont:Term \
    --json
```
Returns `envelope_kind: "term"` with a `TermView`, status `unverified`. The `kind-of` reference resolves (`dont:Term` is seed-locked), so `dangling-definition` does not fire.

**Call 4 — Conclude a claim.**
```
$ dont conclude "The Ricci tensor contracts the Riemann curvature tensor on its first and third indices." \
    --atom "Riemann tensor has four indices" \
    --atom "contraction on indices 1 and 3 yields a rank-2 tensor" \
    --atom "that rank-2 tensor is the Ricci tensor" \
    --ref proj:RicciTensor \
    --json
```
Returns `envelope_kind: "claim"` with a `ClaimView`, status `unverified`, three atoms `unverified`. `hints[]` suggests `dont assume` for independent verification.

**Call 5 — Spawn a verifier.**
```
$ dont assume claim:01HX05A9K8VP --json
```
Returns `envelope_kind: "spawn_request"`. The harness reads `context.prompt` and `context.allowed_tools` (= `dismiss`, `trust`, `show`, `web_search`) and spawns a clean-context subagent. The claim gets a `spawn_requested` event in its history.

**Call 6 — Subagent dismisses (in its clean context).**
```
$ dont dismiss claim:01HX05A9K8VP \
    --atom 0 --atom 1 --atom 2 \
    --evidence https://en.wikipedia.org/wiki/Ricci_curvature \
    --evidence doi:10.1007/978-0-387-74311-8 \
    --quote "Ric_{ij} = R^k_{ikj}" \
    --note "same evidence base verifies all three atoms; matches standard definition" \
    --json
```
The `--atom` flag is repeated once per declared atom (§9.4, §5.2), which verifies each named atom with the provided evidence in a single call. The atom-completion gate (§5.2) then auto-promotes the whole claim to `:verified` on the last atom's transition; an additional whole-claim `dismiss` is not needed and would be refused with `atoms-incomplete` if issued without `--atom` flags while any atom remained unverified. Evidence URIs are validated as well-formed only — no network call. Returns `envelope_kind: "claim"` with updated `ClaimView`, status `verified`, all three atoms `verified`. The original session sees a `spawn_resolved` event when it next polls `dont show` or `dont spawns`.

**Call 7 — Lock the verified claim.**
```
$ dont lock claim:01HX05A9K8VP --json
```
Refuses with `rule-not-met`: `lockable` requires ≥3 assessed hypotheses, and this claim declared none. The refusal's `remediation[]` suggests spawning an `overlook` round to generate and assess hypotheses before retrying. The LLM can accept that refusal (and leave the claim `verified`) or pursue the remediation.

That is the full shape of a productive session: define → conclude → spawn-verify → dismiss → consider locking. The forcing happens at every step.

### 11.3 First session: a teaching walkthrough *[v0.3.2]*

*This section is a Diataxis **tutorial**: sequenced, aimed at first contact, designed to build confidence through doing. Unlike §11.2 (a reference transcript of one canonical session), this walkthrough stops at every step to explain what the LLM is learning and why it is the right move at that stage. If you are an implementer, you may want to run through this as you build. If you are the LLM-user reading this at session start, treat it as a rehearsal.*

The scenario: a fresh project, no prior `dont` experience, one real claim you want to establish. You are the LLM; a harness is running.

**Step 1 — Orient yourself first, act second.**

Start with `dont prime --json`. Always. The reason you do this every session is not ceremony: it tells you what mode the project is in (permissive vs strict — the difference matters because permissive lets you `conclude` with unresolved CURIEs and strict refuses), which rules are active, and what work is already blocking. You cannot know the shape of a productive move without these facts.

```
$ dont prime --json
```

You should see an envelope with `envelope_kind: "prime"` and a `PrimeView` payload (§10.4). Read `mode`, `rules.strict`, and `blocking`. On a fresh project `blocking` is empty and counts are all zero — which is itself information: nothing is waiting for you, and the work ahead is greenfield.

**Step 2 — Before you coin, search.**

The project needs a term for the thing you want to talk about. Suppose it is "intrinsic curvature tensor." The novice move is to `define proj:CurvatureTensor` immediately. The disciplined move is:

```
$ dont suggest-term "intrinsic curvature tensor" --json
```

If the concept is already defined in the project's coined vocabulary, or imported from an ontology (OLS, LinkML, OpenAlex), you will find it and you reuse it. Coining a second term for the same concept is the single most common way LLM-coined vocabularies drift into incoherence, and this command exists to prevent exactly that.

On a fresh project this returns `envelope_kind: "empty"`. You are now free to coin.

**Step 3 — Coin the term (and notice what `define` refuses).**

```
$ dont define proj:RicciTensor \
    --doc "A symmetric (0,2) tensor encoding intrinsic curvature of a Riemannian manifold." \
    --kind-of dont:Term \
    --json
```

This call will succeed because `dont:Term` is seed-locked (§7) and therefore resolves. What would *not* succeed is `--kind-of proj:ManifoldTheory` if that term does not exist yet — `define` always refuses dangling references, in both modes (§8.3). This is deliberate: if definitions could dangle, your vocabulary would become a list of unfulfilled promises. Forcing each `define` to bottom out in already-resolved terms keeps the vocabulary skeleton coherent from the start.

Notice the return: `envelope_kind: "term"`, status `unverified`. The term exists, but it is not trusted. That is normal — everything enters unverified.

**Step 4 — State your claim in atoms.**

You want to claim: "The Ricci tensor contracts the Riemann curvature tensor on its first and third indices." This is a compound proposition. If you dump it into `dont conclude` as a single statement, the atom-completion gate (§5.2) has no way to reason over the parts. Instead, decompose:

```
$ dont conclude "The Ricci tensor contracts the Riemann curvature tensor on its first and third indices." \
    --atom "Riemann tensor has four indices" \
    --atom "contraction on indices 1 and 3 yields a rank-2 tensor" \
    --atom "that rank-2 tensor is the Ricci tensor" \
    --ref proj:RicciTensor \
    --json
```

The three atoms are independently checkable. Each enters `unverified`. The whole claim enters `unverified`. `hints[]` will suggest `dont assume` — this is the machine telling you the next move.

**Step 5 — Do not try to verify your own claim.**

The reflex is to dismiss the claim you just concluded, because you "know" it is true. Resist this. The tool's central thesis (§1, §20.1) is that LLMs cannot reliably self-correct; the lift comes from external signal. The move is to spawn a clean-context verifier:

```
$ dont assume claim:01HX05A9K8VP --json
```

This returns `envelope_kind: "spawn_request"`. Read `context.prompt` and `context.allowed_tools`. The harness — not you — starts a subagent with those exact tools. The subagent's *only* terminal actions are `dont dismiss` (if it finds the claim holds) or `dont trust` (if it finds reason to doubt). You do not perform the verification in the original session.

Your job in the original session, after emitting the spawn request, is to wait and to do unrelated work. The subagent's dismiss-or-trust will land as a `spawn_resolved` event; you see it next time you poll `dont show` or `dont spawns`.

**Step 6 — Learn from a refusal.**

Suppose the subagent dismisses the atoms, the whole claim auto-promotes to `verified` (§5.2 atom-completion gate), and you now want to lock it. You try:

```
$ dont lock claim:01HX05A9K8VP --json
```

This will refuse with `rule-not-met`, because `lockable` requires ≥ 3 assessed hypotheses and you have declared none. The error envelope's `remediation[0].command` will be something like `dont overlook claim:01HX05A9K8VP` — a spawn request for a premortem-style adversarial round that generates and assesses hypotheses.

**Read the remediation. Run it. Do not reformulate.** This is the single most important habit to form. A refusal without a remediation would be a bug (invariant 3.2.5); refusals *always* tell you what to do next. Your only decision is whether to pursue the remediation or accept the refusal (leaving the claim `verified` but not `locked`). Both are legitimate — locking is optional. What is not legitimate is to try different flag combinations on `lock` hoping one will slip past the rule.

**Step 7 — Internalise the loop.**

That is the shape of productive work in `dont`:

1. `prime` to orient.
2. `suggest-term` before coining.
3. `define` to extend the vocabulary.
4. `conclude` with atoms.
5. `assume` to spawn a verifier — never self-verify.
6. On refusal, read `remediation[0].command` and run it.
7. Only lock when the rule is met, and only if you care to.

Everything else in the spec is machinery that makes this loop work. You do not need to read §4 (substrate) or §13 (rule internals) to participate; you need to read your `--json` envelopes.

### 11.4 How-to guides *[v0.3.2]*

*This section is a set of Diataxis **how-to guides**: goal-oriented, minimal, assumes familiarity. Each answers "I want to accomplish X, give me the steps." For conceptual background, read the referenced reference sections; for a sequenced first pass, read §11.3. These guides are intentionally terse.*

#### 11.4.1 How to author a project-specific rule

Goal: add a rule that flags claims whose statement contains a banned phrase (e.g., "as everyone knows").

**Prerequisites.** Familiarity with Cozo Datalog (§4.2 reading list, §13). A text editor. Read-write access to `.dont/rules/`.

1. Create the rule file at `.dont/rules/no-appeal-to-common-knowledge.dl`:

   ```
   ?[claim_id, msg] := *event[claim_id, event_kind, _, _, _, _, _],
                       event_kind = "concluded",
                       *attribute[claim_id, "statement", statement, _],
                       is_in(statement, "as everyone knows"),
                       msg = "statement contains appeal to common knowledge"
   ```

2. Create the sibling `.md` with an English translation of what the rule does and how to satisfy it. This is not optional — `dont explain <rule>` reads this file.

3. Declare severity in `config.toml`:

   ```toml
   [rules]
   warn = ["correlated-error", "no-appeal-to-common-knowledge"]
   ```

   Severities are `warn` (envelope `warnings[]` entry, no refusal) or `strict` (refuses the transition with `rule-not-met` and `rule_name: "no-appeal-to-common-knowledge"`). Start with `warn` unless the rule is one the LLM should never be allowed to violate.

4. Test on a dry run:

   ```
   $ dont rules test no-appeal-to-common-knowledge --json
   ```

   This compiles the rule and runs it against the current store, returning matched entities without emitting events. If compilation fails, the error envelope names the line.

5. Commit `.dont/rules/no-appeal-to-common-knowledge.{dl,md}` and the `config.toml` change. The rule is active on the next `dont` invocation; no restart is required.

**Pitfalls.** Rules that traverse `depends_on` transitively need a recursive datalog clause — see `stale-cascade.dl` for the canonical shape. Rules that need data from outside the store must be declared as flags, not gates, because rule evaluation must remain offline and deterministic (same reasoning as §9.4 on `dismiss`).

#### 11.4.2 How to integrate `dont` into a new harness

Goal: wire `dont` into a harness that currently does not know about it, so that refusals route correctly and spawn requests are fulfilled.

**Prerequisites.** The harness can run shell commands and read their stdout as structured JSON. It has some subagent-spawn mechanism (if not, skip steps 5–6 and use `--direct`).

1. **Install.** `curl -fsSL https://.../install.sh | bash`. Verify with `dont --version`. No per-harness configuration is needed for the binary itself.

2. **Per-project init.** When the harness enters a project directory for the first time:

   ```
   $ dont init --json
   ```

   Treat `already-initialised` (exit `3`, `code: "already-initialised"`) as a non-error — it means the project is already set up.

3. **Signal harness mode.** Set `DONT_HARNESS=1` in the process environment before invoking any `dont` command. Without this, `dont` falls back to terminal-detection (§12.1) and may choose direct mode when you wanted harness mode. Setting the variable is the explicit, reliable signal.

4. **Session start.** On every new LLM session, have the LLM run `dont prime --json` as its first `dont` call. The orientation block (§11.1) should be in the LLM's system prompt. This is what §11.3 walks through.

5. **Handle envelopes.** On every `dont` call, parse the JSON envelope. Branch on `envelope_kind`:
   - `"error"`: the LLM should read `data.remediation[0].command` and run it. Do not ask the LLM to reformulate from scratch.
   - `"spawn_request"`: you — not the LLM — start a clean-context subagent with `data.context.prompt` and the exact `data.context.allowed_tools` list. No other tools. On the subagent's terminal action, record the `spawn_resolved` event by invoking the dismiss-or-trust it chose; `dont` will link it automatically via `request_id`.
   - Any other `envelope_kind`: pass `data` back to the LLM as context for its next turn.

6. **Spawn timeouts.** Poll `dont spawns --pending --json` periodically, or check on session wake; expired spawns auto-revert (§12.2) and will show up as `spawn_timeout` events. Your harness may want to surface these to the operator.

7. **Exit-code hygiene.** Branch on exit code independently of the envelope:
   - `0`: pass to next turn.
   - `1`: refusal — feed the envelope back to the LLM and let remediation-recovery run.
   - `2`: usage error — a harness bug; log and investigate.
   - `3`: substrate/config error — do not retry; run `dont doctor` and surface to operator.
   - `4`: internal error — log, file issue, retry once at most.

8. **Validate.** Run `dont doctor --strict`. All checks should pass. If `aux_linkml` is `warn`, either install the Python `linkml` CLI or accept that LinkML imports will refuse with `linkml-unsupported-feature` in this project.

The harness contract is otherwise transport-agnostic (§12). MCP is one option (`dont mcp`); direct CLI is the simpler default.

#### 11.4.3 How to recover a corrupted `.dont/` store

Goal: diagnose and (if possible) repair a `.dont/` directory that is not behaving correctly.

**Prerequisites.** A backup of `.dont/` before you start. Copy the directory aside before any repair attempt; the event log is append-only but RocksDB files themselves can be lost to filesystem corruption.

1. **Diagnose.**

   ```
   $ dont doctor --json
   ```

   Read each `checks[]` entry's `status` and `detail`. The most common failure modes and their signals:
   - `substrate: fail` — the RocksDB database cannot be opened. Usually a crashed writer left a lock file, or the storage medium is unhappy.
   - `rules_compile: fail` — one or more `.dl` files do not parse. The `detail` names the offending rule.
   - `seed_snapshot: fail` — `.dont/seed/dont-seed.yaml` is missing or unreadable. Without this, the project's seed vocabulary cannot be reconstructed after a reset.
   - `pending_spawns: warn` with old spawns — not corruption, but may indicate a harness that never completed.

2. **Substrate failure.** If RocksDB cannot be opened:

   ```
   $ ls -la .dont/db.cozo/
   ```

   Look for a stale `LOCK` file with no owning process. If found and you are certain no `dont` invocation is running, remove it and retry `dont doctor`. If RocksDB reports checksum or log-file errors, stop — this is beyond `dont`'s repair ability. Restore `.dont/` from your backup and replay any lost events manually via `dont conclude` / `dont define` / etc. (the author string and timestamps will not match the original; this is acceptable for continuity, not for audit fidelity).

3. **Rules-compile failure.** Open the offending `.dl` file. The compile error's `detail` names the line. Fix the syntax. Re-run `dont doctor`. If the rule is a recent addition that cannot be fixed quickly, move it to `.dont/rules/broken/<name>.dl.bak` (a non-standard directory that the rule loader ignores) and restart. The project runs without that rule until you can return to fix it.

4. **Seed-snapshot loss.** If `dont-seed.yaml` is missing but the store is intact, the seed terms are still in the event log as `defined` + `locked` events against their IDs. You can reconstruct the YAML by:

   ```
   $ dont vocab --status locked --kind seed --json
   ```

   Re-serialise the output into `.dont/seed/dont-seed.yaml` in the shape shown at §7. This is a manual operation in v0.3; a future `dont migrate-seed` would automate it (§17).

5. **Nothing worked.** The last-resort path is to initialise a fresh project elsewhere, export the event log from the broken project (read-only access to the RocksDB data is often still possible even when writes fail), and replay the events against the fresh store. This loses the original transaction-numbers and author metadata fidelity but recovers the claim and term content. Details beyond v0.3 scope; contact the project maintainers.

**Before you need this.** The single most useful habit is to keep `.dont/` under version control or in a rsync'd backup. The directory is small (RocksDB compacts aggressively), append-only growth is bounded by actual work, and the recovery-from-backup path is trivial: replace the directory, run `dont doctor`, proceed.

---

## 12. Spawn-request protocol *[v0.3]*

Unchanged in shape from v0.1; two load-bearing gaps closed in v0.3: harness-mode detection and spawn-timeout behaviour.

The protocol is *harness-agnostic*: any harness that can read structured output from a tool call and start a subagent can implement it. Concrete transports:

- **Direct CLI, harness reads stdout/stderr.** The structured envelope arrives on stdout as JSON; the harness parses it, spots `envelope_kind: "spawn_request"`, and invokes its own subagent mechanism.
- **MCP (optional).** A `dont mcp` server mode exposes the same commands as MCP tools; the envelope is the tool's return value. This is one transport among several, not the privileged surface.
- **Shell pipelines.** In `--direct` mode the tool calls a configured LLM provider itself; useful for CI and for harnesses without subagent support.

The harness contract (clean-context subagent, restricted tool set, terminal-only `dismiss`/`trust` callbacks, paired `spawn_requested` / `spawn_resolved` / `spawn_timeout` events) is unchanged in shape.

### 12.1 Harness-mode detection *[v0.3]*

The tool operates in one of two modes:

- **Harness mode (default).** Commands that need new reasoning emit `spawn_request` envelopes. The tool does not call an LLM.
- **Direct mode.** The tool calls the configured LLM provider directly (§14 `[llm]` block).

Detection, in order:

1. If `--direct` is passed on the command line, direct mode. Explicit opt-out wins.
2. Else if `DONT_HARNESS` is set in the environment to any non-empty value other than `0` or `false`, harness mode.
3. Else if the tool is invoked via `dont mcp` (the MCP server subcommand), harness mode.
4. Else if stdout is not a terminal (piped or redirected), harness mode. This catches harnesses that invoke `dont` as a generic shell tool without setting `DONT_HARNESS`.
5. Else direct mode. Interactive terminal use defaults to direct: a human at a shell wants answers, not spawn-request envelopes.

The `PrimeView.harness_mode` field reports the current mode so the LLM (or operator) can verify at session start.

### 12.2 Spawn-timeout behaviour *[v0.3]*

Each `spawn_request` carries an `expires_at` timestamp, computed at issuance as `issued_at + config.harness.spawn_timeout_hours`. Default is 24 hours (§14). A spawn is **pending** if no terminal `dismiss` or `trust` has arrived before `expires_at`.

On any `dont` invocation, the tool checks pending spawns for expiry as part of its startup sweep (O(pending_spawn_count), indexed on `expires_at`; typically sub-millisecond for realistic counts). For each expired spawn:

1. Emit a `spawn_timeout` event with attributes `{request_id, issued_at, expires_at, expired_at}` against the original entity.
2. Restore the entity to its pre-spawn status. For claims/terms that were `:unverified` at spawn time, this is a no-op in the status lattice but the `spawn_timeout` event is recorded.
3. Do not retry or re-spawn automatically. The LLM can re-issue `dont assume` / `dont overlook` explicitly if it still wants the verification.

`dont spawns --pending` lists active (non-expired) pending spawns sorted by age. `dont spawns --timed-out` lists expired spawns that have not had their original entity re-assumed since. `dont spawns --all` returns everything. Timed-out entries remain in the event log permanently (invariant 3.2.1 — nothing is deleted).

Errors related to spawns use codes `spawn-not-found` (the request_id is unknown) and `spawn-expired` (a terminal callback arrived after `expires_at`). On `spawn-expired`, the callback is recorded as a warning but the terminal action is still applied — the LLM should not lose a completed verification to clock-skew.

---

## 13. Methodology as rules *[v0.3]*

Rules live in `.dont/rules/*.dl` as Cozo Datalog (§4.2). Each has a sibling `.md` with its English translation. The substrate commitment in v0.3 removes the dual-format hedge from v0.2: there is exactly one rule format.

Rules shipped by default:

**`ungrounded`** — flags or refuses claims referencing unresolved CURIEs.

**`unresolved-terms`** — blocks `dismiss` on a claim whose CURIEs have not all resolved. Always strict. The invariant that permits permissive mode to be permissive at `conclude` time.

**`stale-cascade`** — on any `trust` transition, cascade every dependent to `:stale`. Dependency edges covered: claim→claim (via `depends_on`), claim→term (via `--ref` / CURIEs in statement or atoms), and term→term (via `kind_of` / `related_to`). Locked and ignored entities are immune.

**`lockable`** — `:verified`, ≥3 assessed hypotheses, ≥2 independent supporting evidence items. Precondition for `dont lock` (§9A.1).

**`correlated-error`** — flags claims whose only evidence shares a source with the author.

**`dangling-definition`** — refuses `define` calls whose `--kind-of` or `--related-to` references do not resolve. Always strict, in both modes. Enforces §8.3.

**Note on `vague-reason`.** The v0.2 `vague-reason` rule was promoted to a verb-level validator in v0.3 (see §9.3) — `reason-required` and `reason-not-hedge` now fire at the `trust` verb, not through the rule system. The rule file does not ship. Projects that want a softer, warn-level check on a separate dimension can author their own rule; the verb-level refusal remains unconditional.

Rule severity defaults:

| Rule                   | Permissive mode | Strict mode | Overridable |
|------------------------|-----------------|-------------|-------------|
| `unresolved-terms`     | strict          | strict      | no          |
| `dangling-definition`  | strict          | strict      | no          |
| `stale-cascade`        | strict (auto)   | strict (auto) | no        |
| `ungrounded`           | warn            | strict      | yes         |
| `correlated-error`     | warn            | warn        | yes         |
| `lockable`             | manual (on `lock`) | manual (on `lock`) | no |

`config.toml` overrides the "yes"-overridable rows on a per-project basis.

---

## 14. Project layout *[v0.3]*

```
.dont/
  db.cozo                   # CozoDB store (RocksDB-backed)
  config.toml
  AGENTS.md                 # canonical LLM-facing doc, owned by dont
  seed/                     # seed vocabulary, installed on init
    dont-seed.yaml
  vocab/                    # LLM-coined terms, one file per term or grouped
  rules/
    ungrounded.dl           ungrounded.md
    unresolved-terms.dl     unresolved-terms.md
    stale-cascade.dl        stale-cascade.md
    lockable.dl             lockable.md
    correlated-error.dl     correlated-error.md
    dangling-definition.dl  dangling-definition.md
    <user-added>.dl         <user-added>.md
  imports/
    manifests/              # import cursors and provenance
  sessions/                 # spawn-request logs, clean-context scratch
  schemas/                  # JSON Schema for envelope, claim, term, event, etc.
AGENTS.md                   # project root; contains dont-managed-block
CLAUDE.md                   # if present; also gets the managed block
```

`config.toml`:

```toml
[project]
name = "research-demo"
mode = "permissive"            # "permissive" | "strict"

[output]
default_format = "json"

[llm]
# Used only in --direct mode. Harness mode ignores this.
provider = "anthropic"
model = "claude-opus-4.7"

[harness]
managed_docs = ["AGENTS.md", "CLAUDE.md", ".cursorrules", ".aider.md"]
spawn_timeout_hours = 24

[rules]
strict = ["unresolved-terms", "dangling-definition", "stale-cascade"]
warn   = ["correlated-error"]
# `ungrounded` is auto-set by mode: strict in strict mode, warn in permissive.
# `vague-reason` is a verb-level validator on `trust`, not a rule (see §9.3).

[trust.hedges]
# Patterns rejected by the reason-not-hedge verb-level check (§9.3).
# Extend per-project if your domain has its own flavours of epistemic mush.
patterns = [
  "might be wrong",
  "not sure",
  "probably incorrect",
  "just a hunch",
  "feels off"
]

[storage]
# Cozo / RocksDB tuning. Defaults are fine for single-session harnesses.
busy_retry_attempts = 3
busy_retry_base_ms  = 50

[verify_evidence]
# Network politeness for `dont verify-evidence` (§9A.4).
concurrency         = 4     # concurrent URI fetches per invocation
per_host_rps        = 2     # sustained requests per second per host
per_host_burst      = 4     # token-bucket burst size
max_retries_429_503 = 3     # per URI on 429/503 before giving up
default_timeout_s   = 10

[import]
ols       = { enabled = true,  base = "https://www.ebi.ac.uk/ols4" }
wikidata  = { enabled = true,  endpoint = "https://query.wikidata.org/sparql" }
openalex  = { enabled = true,  base = "https://api.openalex.org" }
linkml    = { enabled = true }   # subprocess adapter, shells out to linkml CLI
```

---

## 15. Import *[v0.3]*

Small handlers that fetch over plain HTTP or read a file and project into local relations. No LLM involvement. No MCP.

```
dont import obo <path.owl|.obo|.ttl>
dont import ols <ontology-prefix>
dont import wikidata --entity <Qid> | --sparql <file.rq>
dont import openalex --work <doi> | --snapshot <path>
dont import bioregistry
dont import jsonld <file>
dont import ttl <file>
dont import linkml <schema.yaml>         # subprocess adapter, one-way, lossy
```

Writes to `imported_term`, `reference`, or `prefix`. Idempotent per source URI.

**Rate limiting.** Import adapters that fetch over HTTP (`ols`, `wikidata`, `openalex`, `obo` when given a URL) share the same per-host rate-limit policy as `dont verify-evidence` (§9A.4): at most 4 concurrent requests, 2 requests/second/host sustained, burst of 4, honouring `Retry-After` on `429` / `503`. These limits can be tightened per-importer in `config.toml` under the respective `[import.<name>]` block. Adapters reading from a local file (`--snapshot`, LinkML schema on disk) are not rate-limited. The `User-Agent` convention is the same.

**Auxiliary-tool dependency.** Most importers are pure-Rust (HTTP client + parser). `dont import linkml` is the exception: it shells out to the Python `linkml` CLI. If the tool is not on `$PATH`, the import command refuses with error code `config-missing` and a remediation suggesting `pip install linkml` (or the project's preferred equivalent). `dont doctor` reports availability as a `warn` check, not a `fail`, so the presence of LinkML imports in a project does not break `dont doctor` for users who do not use them.

**LinkML import scope.** The adapter shells out to `linkml` CLI commands (`gen-json-schema`, `gen-owl`) to produce intermediate forms, then lowers the intermediate into `imported_term` rows and (optionally) generates Datalog rules in `.dont/rules/imported-linkml-<n>.dl` from the SHACL output. The adapter is **lossy by design**. Three behaviours on encountering LinkML features:

- **Flattened, no warning.** Inheritance (`is_a`) becomes a transitive `kind_of` chain. Mixins are flattened into the inheriting class's attribute set. `slot_usage` refinements are expanded into per-class attribute predicates. These are expected transformations, not failures.
- **Imported with warning.** `permissible_values` enums, string patterns, value ranges, and minimum-cardinality constraints are imported as Datalog rules but the import operation records a warning naming each feature per source. Consumers should spot-check generated rules; automatic translation is approximate.
- **Refused.** Expressions requiring SPARQL evaluation, reified slots, and custom `python_class` injections are not supported. The import operation refuses with error code `linkml-unsupported-feature` listing the offending constructs and a remediation pointing at LinkML itself. No partial import — a schema with an unsupported feature is not imported at all, to avoid silent partial state.

Projects that need LinkML's full semantics should use LinkML directly; `dont`'s adapter is for grounding, not for round-tripping.

---

## 16. MCP interface *[unchanged]*

*Optional.* `dont mcp` runs the tool as an MCP server over stdio, exposing the same commands as MCP tools. Every tool returns the `Envelope` JSON as its result. This exists for harnesses that prefer MCP over direct CLI calls; it is not the privileged surface. Most harnesses invoking `dont` through a generic shell tool will not use MCP at all.

---

## 17. Out of scope for v0.3 *[v0.3]*

- Web UI.
- Multi-user concurrent editing and merge.
- OWL reasoning, SHACL validation (beyond what the LinkML import adapter produces as Datalog).
- SPIRES-style LLM extraction.
- Bitemporal query surfaces. The substrate (Cozo) supports full bitemporality; v0.3 exposes only transaction-time at the CLI layer. Valid-time queries are deferred.
- Seed-vocabulary migration tooling. `dont migrate-seed` is reserved for a future version (§4.4).
- Confidence calibration. Confidences are stored uncalibrated in v0.3 (§5.2).
- Authentication / authorization.
- Encryption at rest.
- A unified identity service across `dont`/`beads`/`wai`. A shared author-string convention is enough.

---

## 18. Open questions *[v0.3]*

The v0.3 revision closed the load-bearing open questions from v0.2 (substrate, harness detection, spawn timeout, envelope versioning, confidence calibration, mode-change event, LinkML import scope). What remains are genuinely deferred design choices whose resolution does not block a working v0.3.

1. **Rule language surface.** Raw Datalog vs a thin DSL over it. Start raw; add sugar when patterns repeat.
2. **Cascades on `dismiss`.** When a `:stale` entity's dependencies resolve, auto-revive or leave in place? Current behaviour: auto-revive (§5.1). The question is whether certain cascade patterns warrant staying stale — open for refinement with usage data.
3. **Evidence liveness sweep.** `dont verify-evidence` runs on-command (§9A.4). Whether to ship a `dont verify-evidence --all` nightly-sweep mode, and how to surface stale evidence in `dont prime` without alarm fatigue, is open.
4. **Streaming vs. buffered JSON.** NDJSON behind `--json-stream`, or cursor pagination for large `list`/`vocab` outputs.
5. **Spawn-request prompt stability.** Version the prompt template and log the version alongside the request; not yet specified.
6. **Seed vocabulary churn.** `dont init` snapshots the seed at init (§4.4). Whether to offer a `dont migrate-seed` command, and what merge semantics it uses when a project has coined terms that conflict with a newer seed, is open.
7. **`suggest-term` enforcement.** An opt-in rule `unsearched-before-coining` that requires a `suggest-term` call before a `define` — whether the tracking cost is justified by the vocabulary-hygiene benefit is open. The plumbing exists; the rule does not ship on by default.

---

## 19. Evaluation *[v0.3]*

v0.2's single "Evaluation questions" list mixed falsifiable criteria with open-ended design review. v0.3 splits the two: acceptance criteria are testable; review prompts are conversation starters. This avoids claiming a test where there is none, and preserves the review prompts as the valuable artefact they are.

### 19.1 Acceptance criteria (measurable)

Pass conditions for a v0.3 implementation to be considered complete. Each has a concrete test harness or procedure.

- **Refusal-recovery rate.** Against the test corpus of synthetic refusal scenarios shipped with the binary (`dont examples --refusal-corpus`), a subagent instantiated from the corresponding `remediation[0].command` reaches the expected terminal state (`:verified` for dismissible scenarios, refusal-acknowledgement for genuinely-blocked scenarios) in ≥ 80% of cases without human input. This bar is **frozen for the v0.3.x series**; any future revision raising the bar will do so in a numbered changelog entry and a dedicated test-corpus version, not by in-place tightening.
- **Spawn-protocol portability.** The same `SpawnRequest` envelope, replayed against two different harnesses (reference: Claude Code + one other that supports subagent spawning), produces equivalent terminal `dismissed` / `trusted` events — same entity, same evidence set, same status transition. Verified by running the spawn-protocol conformance suite against each harness.
- **Envelope stability.** No field is renamed or removed within a major `envelope_version`. Verified by a schema-diff CI check that fails the build on breaking changes. New fields and new `envelope_kind` / `event_kind` values are permitted in minor versions.
- **Determinism of `dismiss`.** A `dismiss` call against a given set of inputs produces the same terminal event (success or refusal) regardless of network state, DNS availability, or remote-host responsiveness. Verified by a `--offline` integration test mode that disables network access and runs the full dismissal suite.
- **Remediation invariant.** Every error envelope produced by the test suite has a non-empty `remediation[]`. Verified by a schema-level check in `dont doctor --strict` and by end-to-end CI.
- **Exit-code contract.** Every test-corpus invocation exits with a code drawn from the §10.7.1 table, and the code matches the envelope's `ok` and `envelope_kind` per the discipline there. Verified by a shell-level assertion harness that does not parse the JSON: it branches on exit code alone and flags any invocation whose exit code contradicts the envelope. *[v0.3.2]*
- **Universal `--help` coverage.** Every subcommand listed in §10, §9, and §9A responds to `--help` (and `-h`) with the same content as `dont help <cmd>` and exits `0`. Verified by a smoke-test that enumerates subcommands and asserts this shape. *[v0.3.2]*
- **Error-message prose linting.** Every shipped error message and every `remediation[].description` passes a Vale configuration bundled with the repository (`/.vale.ini`). The linter checks for imperative voice in remediations, absence of hedge patterns in messages, and absence of internal jargon that is not defined in Appendix A. CI fails on any violation. The goal is not aesthetic uniformity; it is that the LLM-user, for whom these strings are the primary teaching surface (§3.1.2, §11), receives consistent and actionable language. *[v0.3.2]*

### 19.2 Design-review prompts (open-ended)

Questions for periodic design review. Not acceptance criteria; answering them requires judgment, not a test run.

- Does the four-verb core (`conclude` / `define` / `trust` / `dismiss`) plus the lifecycle verbs (`lock` / `reopen` / `ignore` / `verify-evidence`) cover every state transition the workflow actually needs, or are there recurring patterns that suggest a missing verb?
- Is the permissive/strict mode distinction the right axis, or are there orthogonal axes (e.g. "evidence liveness required vs not") that deserve their own dimension?
- Are the ten seed-vocabulary terms enough, too many, or is something missing in practice?
- Is the refusal-with-remediation surface *expressive enough in new situations* — beyond the test corpus — that unscripted refusal scenarios are recoverable without human intervention?
- Does the §12 spawn protocol implement cleanly in `wai`, `beads`-adjacent harnesses, Claude Code, Cursor, and generic shell-based loops without meaningful variation?
- Does the LinkML import adapter's lossiness cause surprises in practice, or is the documented lossy behaviour acceptable to the target users?
- Can a researcher using a `wai`- or `beads`-backed harness complete a realistic session (coin terms, assert claims, verify through spawns, lock what deserves locking) without ever reading the spec?

---

## 20. References and learning material *[patched]*

This section gathers the prior work that underpins `dont`. Entries are grouped by what they illuminate and carry a one-line note on relevance to the tool. The goal is to let a reader (or contributor) understand the *why* of the design without reconstructing the argument from scratch. Where a paper has an arXiv preprint and a venue publication, the arXiv ID is listed for accessibility.

### 20.1 Why LLMs defend what they have said

The core behavioral problem `dont` exists to work around.

- **Perez et al.**, *Discovering Language Model Behaviors with Model-Written Evaluations*, arXiv:2212.09251, 2022. Establishes sycophancy and inverse scaling of honesty under RLHF — the statistical substrate of "defend earlier answers."
- **Sharma et al.**, *Towards Understanding Sycophancy in Language Models*, arXiv:2310.13548, ICLR 2024. Frontier models (Claude, GPT-4, LLaMA-2) flip correct answers under mild user pushback; preference models reward flattering-but-wrong replies.
- **Turpin et al.**, *Language Models Don't Always Say What They Think*, arXiv:2305.04388, NeurIPS 2023. Chain-of-thought is rationalization, not introspection. Directly motivates why `dont assume` spawns a clean-context subagent instead of asking the original session to reconsider.
- **Anthropic**, *Reasoning Models Don't Always Say What They Think*, March 2025. Same unfaithfulness persists in extended-CoT reasoning models; the problem is not solved by more thinking tokens.
- **Ranzato et al.**, *Sequence Level Training with Recurrent Neural Networks*, arXiv:1511.06732, 2015. The foundational account of exposure bias — models teacher-forced on ground truth at training but conditioned on their own samples at inference compound early errors.
- **Huang et al.**, *Large Language Models Cannot Self-Correct Reasoning Yet*, arXiv:2310.01798, 2023. Intrinsic self-correction of reasoning either fails or degrades performance without an external signal.
- **Stechly, Valmeekam & Kambhampati**, *On the Self-Verification Limitations of Large Language Models on Reasoning and Planning Tasks*, arXiv:2402.08115, 2024; and **Kamoi et al.**, *Evaluating LLMs at Detecting Errors in LLM Responses*, 2024. Empirical corroboration that self-verification on reasoning is unreliable; strong external verifiers produce almost all the real gains.

### 20.2 Methods that produce real verification

External-signal methods whose patterns `dont`'s derived commands and rules encode procedurally.

- **Wang et al.**, *Self-Consistency Improves Chain of Thought Reasoning in Language Models*, arXiv:2203.11171, 2022. Majority-voting across diverse samples. The pattern behind `dont guess --n`.
- **Dhuliawala et al.**, *Chain-of-Verification Reduces Hallucination in Large Language Models*, arXiv:2309.11495, 2023. Decompose → independently verify → synthesize. The canonical template behind `dont assume` and the factored-evidence part of the claim schema.
- **Lightman et al.**, *Let's Verify Step by Step*, arXiv:2305.20050, 2023. Process reward models beat outcome reward models; released PRM800K. The argument for rule-level (not just final-answer-level) gating.
- **Yao et al.**, *Tree of Thoughts: Deliberate Problem Solving with Large Language Models*, arXiv:2305.10601, 2023. Backtracking search over intermediate states. Relevant to branch-on-doubt workflows.
- **Shinn et al.**, *Reflexion: Language Agents with Verbal Reinforcement Learning*, arXiv:2303.11366, 2023. Works precisely because a real environment test exists — the same principle enforced by `dismiss --evidence`.
- **Du et al.**, *Improving Factuality and Reasoning in Language Models through Multiagent Debate*, arXiv:2305.14325, 2023. Independent agents critique each other; uncertain facts get filtered. Inspiration for the spawn-request contract.
- **Gou et al.**, *CRITIC: LLMs Can Self-Correct with Tool-Interactive Critiquing*, arXiv:2305.11738, 2023. Critique with external tools, not prose-only reflection.
- **Snell et al.**, *Scaling LLM Test-Time Compute Optimally*, arXiv:2408.03314, 2024. Best-of-N with a PRM beats longer CoT at equal compute. Guides when to spawn additional verifiers.
- **Madaan et al.**, *Self-Refine: Iterative Refinement with Self-Feedback*, arXiv:2303.17651, 2023. Useful for subjective tasks; less so for factual claims — a helpful boundary case.

### 20.3 Institutional analogs

The human systems that solved this problem over decades or centuries. Every `dont` primitive has a predecessor here.

- **Heuer**, *Psychology of Intelligence Analysis*, CIA Center for the Study of Intelligence, 1999. Chapter 8 introduces Analysis of Competing Hypotheses (ACH) — the direct ancestor of the `hypothesis` relation and the ≥3-hypothesis clause in `lockable`.
- **ODNI / CIA**, *A Tradecraft Primer: Structured Analytic Techniques*, 2009. Operational catalogue of SATs: key-assumptions check, devil's advocacy, Team A/Team B. Maps closely onto the derived commands.
- **Mellers, Hertwig & Kahneman**, *Do Frequency Representations Eliminate Conjunction Effects?*, *Psychological Science* 12, 2001, and the broader adversarial-collaboration methodology. Disputing scholars jointly design an experiment and pre-commit to what outcomes favor which position. The social ancestor of `overlook` + `reopen`.
- **Popper**, *The Logic of Scientific Discovery*, 1934/1959. Knowledge progresses by falsification, not confirmation. The epistemic underpinning of "doubt is a first-class event, not a deletion."
- **Lakatos**, *The Methodology of Scientific Research Programmes*, 1970. Progressive vs. degenerating problem shifts. The diagnostic for distinguishing genuine self-correction from post-hoc rationalization — the same distinction Turpin et al. draw at the model level.
- **Klein**, *Performing a Project Premortem*, *Harvard Business Review*, September 2007. Prospective hindsight — imagining the project has already failed — improves correct identification of failure causes by roughly 30% (Mitchell, Russo & Pennington, 1989).
- **Wason**, *On the failure to eliminate hypotheses in a conceptual task*, *Quarterly Journal of Experimental Psychology* 12, 1960; **Nickerson**, *Confirmation Bias: A Ubiquitous Phenomenon in Many Guises*, *Review of General Psychology* 2, 1998. Foundational human parallel to the LLM failure mode.
- **Fagan**, *Design and code inspections to reduce errors in program development*, *IBM Systems Journal* 15:3, 1976. Canonical separation of generator and evaluator. The engineering ancestor of `dont assume`.
- **Chambers**, *Registered Reports: A new publishing initiative*, *Cortex* 2014; **Nosek et al.**, *The preregistration revolution*, *PNAS* 2018. Lock the spec before outcomes are known. Science's answer to artifact locking.
- **Haynes et al.**, *A Surgical Safety Checklist to Reduce Morbidity and Mortality*, *NEJM* 360, 2009; **Gawande**, *The Checklist Manifesto*, 2009. Checklists as structured friction — the pattern behind rule gates.
- **Reason**, *Human Error*, 1990. The Swiss-cheese model; explains why single-layer verification is insufficient and why `dont` layers rules, events, and spawn audits.

### 20.4 Data-modeling foundations

Why datoms and not triples.

- **Hickey**, *The Datomic Information Model* (datomic.com documentation and accompanying talks). The datom — an immutable atomic fact tagged with a transaction and an assertion/retraction bit. The shape adopted at §4.3.
- **Snodgrass & Jensen**, *A Consensus Glossary of Temporal Database Concepts*, 1994; later codified in SQL:2011 temporal features. Formalises transaction-time vs. valid-time — the model Cozo's `Validity` type implements in a minimal form.
- **CozoDB documentation**, cozodb.org. Embedded Datalog with Validity-typed time travel; the store `dont` is built on.
- **XTDB documentation**, docs.xtdb.com. Bitemporal EDN-document DB; proves the shape scales. Considered and deferred as a deployment target because it is a JVM service rather than an embeddable binary.
- **Codd**, *A Relational Model of Data for Large Shared Data Banks*, *CACM* 13:6, 1970. The relational model that EAV-plus-time generalises; included for grounding, not for day-to-day reference.

### 20.5 Ontology and grounding infrastructure

The semantic-web plumbing `dont` imports from but does not reimplement.

- **Moxon et al.**, *LinkML: an open data modeling framework*, *GigaScience* 2025. One YAML compiles to JSON Schema, SHACL, OWL, Pydantic, ShEx. "Swagger for ontologies" — the candidate schema layer if rule authoring ever gains one.
- **Owen et al.**, *OLS4: a new Ontology Lookup Service for a growing interdisciplinary knowledge ecosystem*, *Nucleic Acids Research* 2024. The primary biomedical ontology registry; `dont import ols` is a thin REST client.
- **Hoyt et al.**, *Unifying the identification of biomedical entities with the Bioregistry*, *Scientific Data* 9, 2022. CURIE and prefix normalization — prevents the most common class of LLM hallucination in ontology workflows.
- **Monarch Initiative**, *Ontology Access Kit (OAK)*, github.com/INCATools/ontology-access-kit. Python library for local OBO traversal; the offline-mode reference.
- **Caufield et al.**, *Structured Prompt Interrogation and Recursive Extraction of Semantics (SPIRES)*, *Bioinformatics* 2024 (preprint at PMC10924283). Zero-shot LLM extraction guided by a LinkML schema plus ontology grounding. A future `dont assume` extension point.
- **Mungall et al.**, *CurateGPT: a flexible language-model assisted biocuration tool*, arXiv:2411.00046, 2024. Multi-agent ontology maintenance patterns.
- **W3C recommendations**: RDF 1.1, SPARQL 1.1, OWL 2, SHACL. The standards stack that `dont` can project into but deliberately does not treat as primary.

### 20.6 Adjacent tools and integration layers

What `dont` borrows from in shape.

- **Steve Yegge**, *Introducing Beads: A coding agent memory system*, steve-yegge.medium.com, October 2025; github.com/steveyegge/beads. The install-once-use-everywhere CLI pattern, the `.beads/` per-project convention, and the agent-memory framing.
- **Model Context Protocol specification**, modelcontextprotocol.io. The north-side surface `dont mcp` implements.
- **WAI**, github.com/charly-vibes/wai. Independent sibling tool; `dont` shares conventions (system-wide install, per-project directory, self-documenting surface, managed blocks) but not code.
- **Claude Code, Cursor, OpenCode** — harness implementations whose subagent-spawn mechanisms `dont` delegates to (§12).
- **Dependabot**, **pre-commit**, **Terraform-managed files** — the broader tradition of tools that own a delimited block inside human-edited config files, which `dont sync-docs` follows.

### 20.7 Suggested reading order for new contributors *[patched]*

For someone coming into the project cold, a minimum-viable curriculum:

1. Heuer 1999, chapter 8 on ACH — about twenty pages, and the most direct human precedent for `dont`'s methodology rules.
2. Turpin et al. 2023 and Sharma et al. 2024 — thirty minutes to understand why in-context reconsideration fails.
3. Dhuliawala et al. 2023 (CoVe) and Lightman et al. 2023 (PRM) — thirty minutes to understand what replaces it.
4. The CozoDB documentation, specifically the Validity / time-travel sections — the data model in under an hour.
5. Klein 2007 (premortem) and Mellers/Hertwig/Kahneman 2001 (adversarial collaboration) — the social dynamics `dont` tries to automate.

That is roughly half a day of reading. Enough to defend every non-obvious choice in this spec.

---

## Appendix A. Glossary *[v0.3]*

One-line definitions for the terms this spec introduces or reuses in a specific sense. Cross-references point to the section that treats each in full. Status values appear without the prose-only leading colon (see §5.1 serialisation convention).

- **Already-initialised** (§4.4) — error code refusing `dont init` on a directory that already contains `.dont/`; harnesses that call `init` idempotently at session start should tolerate this as a non-error.
- **Atom** (§5.2) — a sub-statement of a claim, independently checkable. Each atom has its own status in the lattice.
- **Atom-completion gate** (§5.2) — the rule that a claim with declared atoms reaches `verified` only when every atom reaches `verified`; whole-claim `dismiss` (no `--atom` flags) against a claim with unverified atoms is refused with `atoms-incomplete`. Multi-atom `dismiss` calls (`--atom` repeated) are the normal path.
- **Author string** (§4.4) — `<actor-kind>:<id>` identifier recorded on every event; `human:sasha`, `llm:claude-opus-4.7`, `tool:…`, `ci:…`.
- **Claim** (§5, §9.1) — a declarative assertion in the store; enters `unverified`, promotes via `dismiss`.
- **CLI version** (§10.2) — the `dont` binary's semver, independent of `envelope_version`.
- **Core four verbs** (§9) — `conclude`, `define`, `trust`, `dismiss`. The verbs that drive the epistemic lattice.
- **CURIE** — compact URI of the form `prefix:local`; the resolution target for `--ref` and for term parentage.
- **Derived class** (§6) — a named recognition query over attributes; membership is computed, not declared.
- **Derived command** (§10) — a command built on the primitives that emits spawn requests or reads state (`guess`, `assume`, `overlook`, `list`, `show`, `why`, `prime`, `explain`, etc.). Does not include the lifecycle verbs (§9A), which write to the event log.
- **Doubt** — the ambient epistemic state; the thing `trust` triggers and `dismiss` clears.
- **Entity kind** (§5.2) — what an entity *is*: `claim`, `term`, `evidence`, `project`. Distinct from `envelope_kind` and `event_kind`.
- **Envelope** (§10.2) — the stable JSON structure every `--json` response wraps its payload in.
- **Envelope kind** (§5.2, §10.2) — the envelope's payload-type discriminator (`claim`, `claims`, `term`, `event`, `error`, …).
- **Envelope version** (§10.2) — version of the envelope schema itself; stable within a major, independent of `cli_version`.
- **Event** (§4.2, §5.2) — an immutable record of one transition; the store is the event log, derived views are computed from it.
- **Event kind** (§5.2) — what the event records. Canonical v0.3.1 enumeration: `created`, `concluded`, `defined`, `trusted`, `dismissed`, `locked`, `ignored`, `stale_cascaded`, `stale_restored`, `spawn_requested`, `spawn_resolved`, `spawn_timeout`, `evidence_checked`, `mode_changed`. Additions require a minor envelope-version bump; see §5.2 for the single source of truth.
- **Evidence** (§7, §9.4) — a URI or CURIE that `dismiss` cites to earn `verified`; recorded per claim or term. Liveness is checked separately by `verify-evidence`.
- **Evidence liveness** (§9A.4) — whether an evidence URI is currently reachable. Checked by `dont verify-evidence`, not by `dismiss`, since v0.3.
- **Exit code** (§10.7.1) — the shell-facing status `dont` returns on termination; values `0` (success), `1` (refusal), `2` (usage), `3` (substrate/config), `4` (internal), `130` / `143` (signals). The first branch a harness inspects; the envelope refines. *[v0.3.2]*
- **Forcing function** (§1) — the design stance: shape the LLM's action space so wrong moves are syntactically blocked, not merely discouraged.
- **Harness** — the LLM execution environment that calls `dont` as a tool (Claude Code, Cursor, OpenCode, Claude.ai, custom).
- **Harness mode** (§12.1) — default operating mode where derived commands emit spawn requests rather than calling LLMs directly. Detection rules in §12.1.
- **Hedge pattern** (§9.3) — a string pattern (configured under `[trust.hedges]`) that disqualifies a `trust --reason` value from satisfying the `reason-not-hedge` verb-level check.
- **Hint** (§10.2) — `{command, description}` in the envelope; agent-safe suggestions for next action. Distinct from `remediation` (which appears inside error payloads).
- **Hypothesis** (§7) — a claim offered for consideration; weaker commitment than `unverified`. Counted by `lockable`.
- **Ignore** (§9A.3) — the lifecycle verb moving an entity to `ignored`.
- **Ignored** (§5.1, §9A.3) — a terminal state outside the lattice; for claims whose references became permanently unresolvable or that the LLM has explicitly abandoned.
- **Imported term** (§5.3) — a term pulled from an external ontology; lives in `imported_term`, is not doubtable via the normal verbs.
- **Invariant** (§3.2) — a testable behavioural commitment the spec makes about the tool; distinct from a design principle (§3.1).
- **Lifecycle verb** (§9A) — a verb that operates around the lattice: `lock`, `reopen`, `ignore`, `verify-evidence`. Separate from the core four.
- **Lock** (§9A.1) — the lifecycle verb promoting a claim from `verified → locked` when `lockable` is met. Applies to claims only in v0.3.1; non-seed terms cannot be locked (they have no `hypotheses[]` slot, and `lockable` is hypothesis-dependent). Seed terms enter `:locked` on `dont init` via a separate mechanism (§7).
- **Locked** (§5.1) — terminal status reached via `lock` on a `:verified` claim meeting `lockable`, or assigned to seed terms at `init` time.
- **Mode change** (§8.4) — a permissive↔strict transition; recorded as a `mode_changed` event against a synthetic project entity.
- **Permissive / strict mode** (§8) — project-level setting controlling *when* the grounding check fires (`conclude`-time vs `dismiss`-time). Data shape is identical.
- **Primitive** (§6) — one of the five schema-level concepts the tool understands: `attribute`, `derived_class`, `enum`, `prefix`, `rule`. Distinct from a "core verb."
- **Provenance** (§10.4) — the author/origin/session block attached to every claim and term for audit.
- **Reason-required / reason-not-hedge** (§9.3) — the two verb-level validators that `trust` applies to `--reason` in v0.3; replaces the v0.2 `vague-reason` rule.
- **Remediation** (§10.5) — the non-empty `{command, description}` array in an error envelope; the mechanism by which refusals are actionable. Required by invariant 3.2.5.
- **Reopen** (§9A.2) — the lifecycle verb escaping a stale cascade: `stale → unverified` on manual request.
- **Rule** (§6, §13) — a Datalog predicate (`.dl` file) that fires on events or queries; rules produce warnings or refusals. Distinct from a verb-level validator.
- **Rule name** (§10.5) — the name of the §13 rule that refused, or `null` for verb-level validators. Separate from `spec_ref`.
- **Seed vocabulary** (§7) — the ten locked `dont:`-prefixed terms installed at `init`; the bootstrap ontology.
- **Signal handling** (§4.2) — `SIGINT` / `SIGTERM` during a transaction cause clean rollback; append-only substrate guarantees no half-applied state. Exit codes `130` / `143`. *[v0.3.2]*
- **Spawn request** (§10.4, §12) — a structured `envelope_kind: "spawn_request"` envelope asking the harness to start a clean-context subagent.
- **Spawn timeout** (§12.2) — the point at which a pending spawn's `expires_at` is reached; emits a `spawn_timeout` event and restores the entity to its pre-spawn status.
- **Stale** (§5.1) — transient status indicating a dependency was doubted; auto-restores when the dependency resolves, or is manually escaped via `reopen`.
- **Subagent** (§12) — the clean-context LLM instance a harness starts in response to a spawn request; has a restricted tool set and terminal-only `dismiss`/`trust` actions.
- **Term** (§5, §9.2) — an LLM-coined vocabulary entry; a first-class doubtable entity with its own lifecycle.
- **Transaction (`tx`)** (§4.2, §10.3) — a monotonically increasing integer attached to every event; the primary ordering key.
- **Unverified** (§5.1) — the entry status for every claim and term; the state that makes `verified` something to be earned.
- **Usage error** (§10.7.1) — a malformed CLI invocation (unknown flag, bad argument). Exit code `2`, `code: "usage"`, `remediation[]` pointing at `dont help <cmd>`. Distinct from a refusal (exit `1`) because usage errors are not domain events. *[v0.3.2]*
- **Verify-evidence** (§9A.4) — the lifecycle verb checking liveness of evidence URIs; the only verb that hits the network.
- **Warning** (§10.2) — non-refusing rule flag attached to the envelope; the operation completed but something is worth noting.
- **Wrong-entity-kind** (§9A.1) — error code returned when a verb is applied to an entity of the wrong kind (e.g. `dont lock <term-id>` in v0.3.1, where `lock` is restricted to claims).

---

## 21. Changelog from v0.1 *[new]*

**Scope narrowed.** The tool is now a forcing function, not a knowledge substrate. Bug prevention, schema-system-ness, and claim-store-as-product are demoted to consequences of the forcing mechanic rather than pillars.

**User identified.** The LLM is the user. Human-readable output is secondary.

**MCP demoted.** MCP is one optional transport. The harness's tool-use channel is the primary surface, transport-agnostic.

**Fourth verb.** `define` joins `conclude`/`trust`/`dismiss`. Terms are first-class doubtable entities.

**Seed vocabulary.** Ten locked terms shipped on `init`.

**Modes.** Permissive (default) and strict, differing only in when the grounding check runs.

**Schema redesign.** Five primitives (attribute/derived_class/enum/prefix/rule), no inheritance, recognition by predicate. LinkML is an import adapter, not a dependency.

**Substrate deferred.** SQLite is the default working assumption; CozoDB/datom remains a candidate for later if Datalog expressiveness is needed.

**`suggest-term`, `terms`, `dangling-definition` rule, `vague-reason` rule, `unresolved-terms` rule.** New surfaces supporting the four-verb core.

**Tool independence.** `dont`, `beads`, `wai` are explicitly peer-and-independent. Shared conventions, no shared code or config.

---

### v0.2 patch pass (Rule-of-5 review)

Targeted edits that closed drift from the v0.1→v0.2 reconciliation. No new features; no change to the four-verb core; no change to the data shape.

- **Numbering.** §20 subsections (mislabeled 19.1–19.8) renumbered to 20.1–20.8 (absorbing the private-upload §20.7 into the reading order, now §20.7). §9.1 "Envelope" (a v0.1 heading that survived) renumbered under §10 along with the adjacent subsections; new layout: §10.1 suggest-term, §10.2 Envelope, §10.3 Identity and format conventions, §10.4 Core payload types, §10.5 Error envelope, §10.6 Input schemas.
- **Dead references.** `no-doubt --evidence` (v0.1 verb) → `dismiss --evidence` (§20.2 Shinn entry). Error-envelope example `"rule": "§6.3"` → `"rule": "§9.4"` (§10.5). `SpawnRequest` pointer "§8.11" → "§12" (§10.4). Stale `ignore` remediation in the error example replaced with `assume`-then-`dismiss`.
- **Verb naming rationale.** New §9.0 explains the `trust` / `dismiss` inversion that was previously present but unexplained.
- **`reopen` and `ignore` specified.** §9.5 defines `dont reopen --lock` (the only path to `:locked` for non-seed entities) and `dont reopen` (manual `:stale` → `:unverified` escape). §9.6 defines `dont ignore` (the escape valve for permanently unresolvable CURIEs).
- **Status lattice.** Rewritten with explicit transition table and with `:verified → :locked`, `:stale → :doubted`, and `:stale → :unverified` arrows that were previously missing (§5.1).
- **`stale-cascade` extended.** Dependency edges now cover term→term via `kind_of` / `related_to`, not just claim→claim and claim→term (§5.1, §13).
- **Atoms defined.** §5.2 now states what an atom is, when it becomes `:verified`, and how `trust` / `dismiss` target individual atoms.
- **CURIE collision.** §5.3 states that coined `term` shadows `imported_term` and specifies re-import behavior.
- **Author identity.** §4.4 states the `<actor-kind>:<id>` convention explicitly instead of leaving it implicit in examples.
- **Seed versioning.** §4.4 commits to snapshot-at-init; binary-embedded seed is template only.
- **Transaction semantics.** §4.2 specifies single-invocation transaction boundaries and parallel-call serialisation.
- **Cyclic definitions.** §8.3 documents the trust-redefine pattern for mutually-referential terms.
- **Rust rationale.** §4.1 explains why Rust specifically, not just "a compiled systems language."
- **Command one-liners.** §10 now carries a one-line summary per derived command rather than deferring everything to v0.1 docs.
- **Worked example.** §11.2 shows a seven-call productive session end-to-end.
- **Rule severity table.** §13 replaces prose bullets with a four-column table.
- **Glossary.** New Appendix A with one-line definitions for every term the spec introduces or specialises.

---

### v0.3 pass (post-evaluation revision)

Targeted changes closing the findings from a Specification Evaluation Diagnostician pass against v0.2-patched. Substantive architectural commitments in three places (substrate, verb split, `dismiss` determinism) and editorial/coherence fixes throughout. No change to the forcing-function thesis or to the four-verb core's meaning.

**Architectural commitments.**

- **Substrate: CozoDB.** The v0.2 hedge ("SQLite or Cozo, decided late") is resolved. v0.3 commits to Cozo, embedded via the `cozo` crate on a RocksDB backend. §4.2 restores the v0.1 argument for datoms over rows. Transaction-time-only at the CLI layer; full bitemporality in the substrate for future use. §4.1 Rust rationale updated (no more "substrate-hedge trait" clause). §13 rules are `.dl` only — the dual-format fallback removed. §14 layout uses `db.cozo`.
- **Verb split: `reopen` → `lock` + `reopen`.** The v0.2 compound `reopen --lock` / `reopen` is split on ISO 29148 singularity grounds into two verbs. New §9A "Lifecycle verbs" separates `lock`, `reopen`, `ignore`, and the new `verify-evidence` from the four-verb core. §9 is now actually four verbs.
- **`dismiss` deterministic.** Network-free: only schema-shape checks (`unresolvable-uri` narrowed to malformed URI / unknown prefix), `unresolved-terms`, and the atom-completion gate. Evidence liveness moves to the new `dont verify-evidence` command (§9A.4). Closes the non-determinism flagged in the evaluation and makes refusals reproducible offline.

**Load-bearing open questions closed.**

- **Confidence calibration (§18.3, v0.2).** Stored uncalibrated in v0.3 (§5.2); calibration deferred to §17.
- **`DONT_HARNESS` detection (§18.6, v0.2).** Explicit five-step detection order in §12.1.
- **Spawn-request failure modes (§18.7, v0.2).** `spawn_timeout` event, auto-revert to pre-spawn status, `dont spawns --timed-out` (§12.2).
- **Envelope versioning (§18.8, v0.2).** Field renamed `dont_version` → `envelope_version` (with a sibling `cli_version`); current value set to `"0.2"` to match the schema's actual maturity; versioning policy spelled out in §10.2. Closes the stale `"0.1"` in v0.2 examples.
- **Mode change as event (§18.13, v0.2).** `mode_changed` event attached to a synthetic project entity (§8.4).
- **LinkML import scope (§18.14, v0.2).** Three-tier behaviour (flattened-silent / imported-with-warning / refused) in §15, with `linkml-unsupported-feature` error code for the refuse case.

**Coherence and editorial fixes.**

- **Atom-completion gate** is now authoritatively stated once (§5.2) and referenced from §9.4; the ambiguity of "does whole-claim dismiss bypass unverified atoms?" is resolved as "no, `atoms-incomplete` refusal."
- **`rule` field split** in the ErrorResult into `rule_name` (nullable; names a §13 rule) and `spec_ref` (a section pointer for debugging). Closes the v0.2 overloading where the field held spec refs but claimed to hold rule names.
- **`vague-reason` → verb-level.** The hedge check promoted from a warn-rule to verb-level validators `reason-required` and `reason-not-hedge` on `trust` (§9.3). Config moved under `[trust.hedges]` in `config.toml`. Rule file no longer ships.
- **`remediation[]` required.** Invariant 3.2.5 explicitly requires non-empty `remediation[]` on every error envelope; schema enforces; `dont doctor --strict` validates.
- **Event-kind enumeration.** Canonical list of all `event_kind` values in §5.2, no ellipsis.
- **"Kind" disambiguation.** `entity_kind`, `envelope_kind`, `event_kind` all separately named in serialisation (§5.2). The bare word "kind" is no longer load-bearing in JSON.
- **Status serialisation.** Colons dropped from wire-format status values (`"status": "verified"` not `":verified"`); colons retained in prose as a visual cue; §5.1 states the convention. Glossary updated to match.
- **§3 split.** "Core principles" separated into §3.1 Design principles (directional) and §3.2 Invariants (testable), closing the mix of aspirations and commitments flagged by the review.
- **§19 split.** Evaluation questions split into §19.1 Acceptance criteria (five measurable conditions with test harnesses) and §19.2 Design-review prompts (the rest, relabelled honestly as open-ended prompts rather than tests).

**New payload types and surfaces.**

- `DoctorReport`, `ExamplesList`, `SchemaDoc` fully specified in §10.4 (v0.2 mentioned the commands but left the payloads unspecified).
- `TermView` moved from an orphaned post-§10 block into §10.4 proper.
- `EventView` now uses `entity_id` (not `claim_id`) and `event_kind` (not `kind`); reflects the namespacing cleanup.
- `SpawnRequest` carries `expires_at` directly; `forbidden_tools` includes `dont.lock` and `dont.reopen` alongside `dont.conclude`.

**Consequence-only changes.**

- §10.6 input schemas use `entity_id` universally (not `claim_id`), since every verb operates on both claims and terms.
- §11.1 orientation block updated for the four + four shape (core four, lifecycle four).
- §11.2 Call 7 uses `dont lock` (not `dont reopen --lock`).
- §17 retitled "Out of scope for v0.3"; confidence calibration and seed-migration added as explicit deferrals; bitemporality wording refined (substrate-capable, CLI-deferred).
- §18 shrunk from 14 items to 7. All remaining items are genuinely deferred, not load-bearing.
- Appendix A glossary expanded with `Atom-completion gate`, `CLI version`, `Core four verbs`, `Entity kind`, `Envelope kind`, `Envelope version`, `Event kind`, `Evidence liveness`, `Hedge pattern`, `Invariant`, `Lifecycle verb`, `Lock`, `Mode change`, `Reason-required / reason-not-hedge`, `Reopen`, `Rule name`, `Spawn timeout`, `Verify-evidence`. Cross-references updated for the verb split (`Locked` → §9A.1, `Ignored` → §9A.3).

---

### v0.3.1 patches (post-review-pass)

Targeted fixes closing the findings from a Rule-of-5 specification review against v0.3. Two CRITICAL internal contradictions resolved; one unreachable capability removed; the remaining items are coverage, precision, and input-schema cleanups. No change to the forcing-function thesis, the four-verb core, the lifecycle verbs, or the data lattice.

**Critical contradictions closed.**

- **Worked-example atom-completion conflict (§11.2 Call 6).** v0.3 Call 6 whole-claim-dismissed a three-atom claim with all atoms `:unverified`, which §5.2's atom-completion gate explicitly refuses with `atoms-incomplete`. Fixed by making `--atom` repeatable on `dismiss` (§9.4) and rewriting Call 6 to use `--atom 0 --atom 1 --atom 2` in a single call. Shared evidence applies to each named atom; whole-claim auto-promotion still happens on the last atom's transition.
- **`evidence` / `depends_on` relation keying (§5.2).** v0.3 kept `claim_id` as the foreign key on both relations even though §9.4 and the §21 changelog committed to "every verb operates on both claims and terms." Fixed by re-keying both relations to `entity_id`, with a paragraph clarifying that term-level `depends_on` rows are synthesised from `kind_of`/`related_to` at rule-evaluation time rather than written directly.

**Unreachable capability removed.**

- **`lock` restricted to claims (§9A.1).** `lockable` requires ≥3 assessed hypotheses; the TermView schema has no `hypotheses[]` slot; non-seed terms could never satisfy the rule. Rather than add hypotheses to the term schema (a larger change with cascading effects on `define` and on the term worked-example surface), v0.3.1 scopes `lock` to claims and introduces the `wrong-entity-kind` error code for a term-target refusal. Seed terms continue to enter `:locked` via `dont init`. §10.4 TermView's `applicable_rules` no longer lists `lockable`.

**Coverage gaps closed.**

- **Duplicate `conclude` and duplicate `define` (§9.1, §9.2).** v0.3 did not specify either; v0.3.1 states that `conclude` does not deduplicate (two identical statements → two entities with separate provenance; optional opt-in `duplicate-statement` rule for projects that want a warn-level flag) and that `define` on an existing CURIE is redefinition per §8.3, not a new entity.
- **`dont init` re-invocation (§4.4).** Refuses with `already-initialised`. No `--force` in v0.3.x. Harnesses calling `init` idempotently tolerate the refusal.
- **Author-string colon parsing (§4.4).** Split on the first `:` only; `id` is opaque and may contain colons.
- **`verify-evidence` timeout shape (§9A.4).** Per-URI `--timeout` (default 10 s); on timeout, `evidence_checked` event records `status_code: null`, `timed_out: true`, and an `evidence-stale` warning is attached to the envelope. The command always returns partial results.
- **Post-retry `db-locked` failure shape (§4.2).** After 3 retries, the call emits a non-zero-exit `error` envelope with `code: "db-locked"`, `unmet_clauses[]` naming the holding transaction, and `remediation[]` suggesting `dont doctor` plus a retry.
- **Cyclic-definition expected state (§8.3).** A four-row table follows the four-step pattern, showing status and event history for both terms after step 4.
- **Seed YAML shape (§7).** `.dont/seed/dont-seed.yaml` example block added; each entry is one synthetic `defined` + `locked` event pair at init time.

**Precision and internal-consistency fixes.**

- **`applicable_rules` discriminated (§10.4).** v0.3 showed two different shapes (`{met, unmet}` for gates, `{flagged}` for flags) without a discriminator. v0.3.1 adds a `kind: "gate" | "flag"` field to every entry. New kinds require a minor envelope-version bump.
- **`rule-not-met` scope clarified (§10.5).** Generic for any §13 rule-system failure; `rule_name` identifies the specific rule. Verb-level validators have their own dedicated codes and set `rule_name: null`.
- **§4.3 event-kind ellipsis → pointer (§4.3).** v0.3 had `(created, concluded, ..., ignored, ...)` while §5.2 declared a canonical complete list. §4.3 now points at §5.2 instead of competing with it.
- **Appendix A event-kind entry fixed.** v0.3's glossary entry used `etc.` and was missing `defined` and `stale_restored`. v0.3.1 lists all fourteen values and names §5.2 as the single source.
- **`WhyView` JSON example added (§10.4).** v0.3 described the shape in prose only; v0.3.1 adds an example in the same style as the other payload types, including the `remediation[]` per-unmet-rule structure.
- **Input-schema array notation (§10.6).** Fields are now explicitly typed (`atoms?: string[]`, `depends_on?: ClaimId[]`, `evidence: EvidenceSpec[]`, etc.). `DismissInput.atom_idx` is now `int[]` (repeatable) to match the `--atom` repeatability in §9.4.
- **Acceptance-criterion bar frozen (§19.1).** Refusal-recovery bar ≥ 80% is frozen for the v0.3.x series; future tightening requires a numbered changelog entry and a dedicated test-corpus version.
- **§3.1.2 de-duplicated from invariant 3.2.5.** Reworded as a directional design attitude ("Refusals teach, not punish") rather than a paraphrase of the testable invariant.
- **Cozo crate minimum version pinned (§4.2).** `cozo` ≥ 0.7 — Validity-type and transaction-API breakage below that version is not supported.
- **Quantified "bounded work" (§12.2).** Replaced with `O(pending_spawn_count)` and indexing detail.
- **§6 primitive-addition bar procedural, not subjective.** "Requires written justification in the PR that introduces them" replaces "nothing more should be added without evidence."
- **§20.1 Stechly citation specified.** Single 2024 paper named instead of "multiple 2023–24 papers."

**Glossary additions.** `Already-initialised`, `Wrong-entity-kind`. `Atom-completion gate` and `Lock` / `Locked` entries updated for the multi-atom-dismiss and claims-only semantics respectively.

**Consequence-only changes.**

- §9.4 signature: `[--atom <idx>]*` (repeatable) replacing `[--atom <idx>]` (single).
- §10.6 `DismissInput.atom_idx`: `int[]` replacing `int?`.
- §10.4 ClaimView and TermView `applicable_rules` shapes carry `kind` discriminators.
- §10.4 TermView `applicable_rules` no longer lists `lockable`.
- §10.5 known-codes list gains `wrong-entity-kind`, `already-initialised`, `linkml-unsupported-feature`; warning codes `evidence-malformed`, `evidence-stale` explicitly noted.
- §11.2 Call 6 rewritten with repeatable `--atom`.

---

### v0.3.2 patches (post-UX/DX-review)

Targeted fixes closing the findings from a UX/DX evaluation against v0.3.1, run across Diataxis Layer 5 (documentation modality coverage) and the CLI/API heuristics of Layer 3 (interface discoverability, conventions, composability). No CRITICAL findings; both applicable layers rolled up to DEGRADED on High-severity gaps. This pass closes them. No change to the forcing-function thesis, the four-verb core, the lifecycle verbs, the data lattice, or the envelope schema (no `envelope_version` bump — the patch is additive on the shell-facing surface and on the documentation sections; no existing field is renamed or removed).

**Layer-3 (interface) fixes.**

- **Exit-code taxonomy specified (§10.7.1).** v0.3.1 had two ad-hoc mentions of "non-zero exit" (§4.2 `db-locked`, §10.4 `dont doctor --strict`) without a complete scheme. v0.3.2 adds a six-row table stratifying `0` success, `1` refusal, `2` usage error, `3` substrate/config error, `4` internal error, `130` / `143` signals. The refusal-vs-substrate boundary is now the first branch a harness inspects, before the envelope — exit `1` means the LLM can make progress via `remediation[]`, exit `3` means stop asking the LLM.
- **Universal `--help` flag (§10.7.2).** v0.3.1 shipped `dont help` and `dont help <cmd>` but never committed to `--help` / `-h` on every subcommand. v0.3.2 states that every subcommand accepts `--help` / `-h` and routes to the same content as `dont help <cmd>`. A new universal-flag table also specifies `-j` for `--json`, `-a` for `--author`, `--plain`, `--direct`, `--version`.
- **Signal handling (§4.2).** v0.3.1 left behaviour under `SIGINT` / `SIGTERM` implicit. v0.3.2 specifies clean rollback on either: no half-written state, exit codes `130` / `143`. Relies on the append-only substrate guarantee — either a transaction commits in full before the signal or it did not happen. `SIGPIPE` is treated as normal termination (exit `0`) so NDJSON pipelines `head -n 1` cleanly.
- **Stdin input for bulk operations (§10.7.4).** v0.3.1 specified JSON on stdout (the downstream half of a pipeline) but no upstream half. v0.3.2 specifies that commands accepting `<entity-id>` accept `-` as a stdin sentinel, reading one ID per line and emitting one envelope per line (NDJSON). Closes the missing composability half of the Unix-pipeline story. Listed verbs: `show`, `why`, `trust`, `dismiss`, `ignore`, `lock`, `reopen`, `verify-evidence`. `conclude` and `define` explicitly excluded — they take domain content, not IDs.
- **Colour and terminal awareness (§10.7.3).** Honours `NO_COLOR` convention, honours `CLICOLOR_FORCE=1`, adds `--plain` for unformatted human output. Independent of `--json` (which is uncoloured by definition).
- **Shell completion (§10.7.5, §10 command listing).** New `dont completions <shell>` subcommand for `bash`, `zsh`, `fish`, `powershell`, `elvish`. Covers subcommands, flags, enum-valued flags, and fast-path dynamic completion of entity IDs (≤ 10 ms budget; degrades to static rather than blocking the prompt).

**Layer-5 (documentation) fixes.**

- **Tutorial modality added (§11.3).** v0.3.1 had reference (§5, §9, §9A, §10, §13, §14) and explanation (§3, §4, §9.0, §20) but no Diataxis tutorial — no sequenced, learning-oriented walkthrough aimed at first contact. v0.3.2 adds §11.3 "First session: a teaching walkthrough" — seven steps, one concept per step, surfaces at least one instructive refusal, culminates in the productive-loop shape (`prime` → `suggest-term` → `define` → `conclude` → `assume` → remediation → optional `lock`). Explicitly aimed at the LLM-user persona; humans reading along are welcome but are not the primary audience.
- **How-to modality added (§11.4).** v0.3.1 was strong on reference and explanation but thin on goal-oriented recipes. v0.3.2 adds §11.4 with three how-to guides covering the three most common integrator and operator goals: authoring a project-specific rule (§11.4.1), integrating `dont` into a new harness (§11.4.2), and recovering a corrupted `.dont/` store (§11.4.3). Each guide is terse by design — Diataxis how-tos assume familiarity; for first-contact sequencing see §11.3, for conceptual background see the referenced reference sections.
- **Help surface cross-referencing (§10.7.6, §11.1 orientation block).** `dont help --tutorial` prints §11.3; `dont help --howto <topic>` prints a §11.4 guide; `dont help --topics` lists both. The orientation block in §11.1 now points the LLM at these entry points alongside `dont help <cmd>`.

**Coverage fixes (medium).**

- **Rate limiting and backoff (§9A.4, §15).** v0.3.1 had a per-URI `--timeout` on `verify-evidence` but no rate-limit policy on outbound network traffic. v0.3.2 specifies: 4 concurrent requests per invocation, 2 requests/second/host sustained with burst of 4, `Retry-After`-honouring exponential backoff on `429` / `503` up to 3 retries per URI, configurable via `[verify_evidence]` in `config.toml`. Same policy applies to HTTP-fetching import adapters (`ols`, `wikidata`, `openalex`, URL-based `obo`) per-importer in `[import.<n>]`. `User-Agent` header emits `dont/<cli_version>` for attribution.
- **`config.toml` extended with `[verify_evidence]` block (§14).** Mirrors the network-politeness fields above; defaults match the §9A.4 text. Callers running against their own ontology servers can tighten.
- **Error codes `usage` and `internal` (§10.5).** Two exit codes introduced in §10.7.1 imply two corresponding error-envelope codes. v0.3.2 adds both to the known-codes list. `usage` remediations point at `dont help <cmd>`; `internal` remediations point at `dont doctor` and at issue-reporting.

**New acceptance criteria (§19.1).**

- **Exit-code contract.** Every test-corpus invocation exits with a code from the §10.7.1 table matching the envelope's `ok` and `envelope_kind`. Verified by an exit-code-only harness that does not parse JSON.
- **Universal `--help` coverage.** Every subcommand responds to `--help` and `-h` with the same content as `dont help <cmd>` and exits `0`. Smoke-test enumerates subcommands and asserts.
- **Error-message prose linting.** Every shipped error message and every `remediation[].description` passes a bundled Vale configuration. Checks for imperative voice in remediations, absence of hedge patterns in messages, and absence of internal jargon undefined in Appendix A. CI-enforced. Goal: consistent and actionable language for the LLM-user, for whom these strings are the primary teaching surface.

**Glossary additions (Appendix A).** `Exit code`, `Signal handling`, `Usage error`. No existing entries revised.

**Consequence-only changes.**

- §10 derived-commands listing gains `dont completions <shell>` alongside the existing commands; §10 one-liner summaries gain a `completions` line.
- §11.1 orientation block gains two lines pointing the LLM at `dont help --tutorial` and `dont help --howto <topic>`.
- `config.toml` example (§14) gains the `[verify_evidence]` block.
- §10.5 known-codes list gains `usage` and `internal`.
- Appendix A gains three entries as listed above.

**Deliberately unchanged.** The `envelope_version` remains `"0.2"`. No payload-type schemas are modified. No verb signatures change. The four-verb core is untouched, as are the lifecycle verbs and their refusal taxonomy. §21's v0.3.1 patch block remains authoritative for the v0.3→v0.3.1 transition; the v0.3.2 block above documents only the v0.3.1→v0.3.2 delta.
