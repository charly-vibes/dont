# `dont` — specification (v0.2 draft)

**Status:** design draft, not implemented. This revision replaces the v0.1 draft after a round of value-proposition work that substantially narrowed the tool's scope and reshaped several core decisions. Sections marked *[revised]* differ materially from v0.1; sections marked *[new]* are additions; sections marked *[unchanged]* carry over intact.

---

## 1. Purpose *[revised]*

`dont` is a **forcing function** for autonomous LLM harnesses. It is a per-project command-line tool that an LLM calls directly, as a peer tool alongside harness-provided memory (`beads`) and workflow (`wai`) tools. Its single job is to interrupt the LLM's default behaviour of confidently asserting unchecked claims, and to give the LLM a structured, mechanical path to earn the right to assert.

The problem it addresses, distilled:

- LLMs do not reliably self-correct. Lift comes from external signal.
- Once a claim enters the context window, the autoregressive substrate makes it cheaper to defend than to retract.
- Durable knowledge-producing institutions (peer review, CI gates, the nuclear two-person rule) solve this by separating generator from evaluator and by making retraction a first-class event.

`dont` implements that separation as a minimal CLI with four verbs, an append-only event log, and a refusal protocol whose error messages *are the user interface*. When the LLM tries to assert something it has not adequately grounded, `dont` refuses with a structured remediation the LLM can act on without human involvement. When the LLM asks for verification, `dont` emits a structured instruction for the harness to spawn a clean-context subagent whose only terminal actions are `no-doubt` (with evidence) or `doubt` (with reason).

**The user is the LLM.** Humans may inspect `dont`'s store after the fact for audit, but they are not the target of the CLI. Human-mode output exists as a courtesy; `--json` is the product.

**The tool grows its vocabulary with use.** `dont` does not ship a domain ontology. It ships a tiny seed vocabulary (§6) and forces the LLM to coin or import every domain-specific term it uses. A term coined by the LLM is itself a claim and travels through the same doubt machinery as any other claim.

---

## 2. What `dont` is not *[revised]*

- Not a task tracker. That is `beads`.
- Not a workflow spec engine. That is `wai`.
- Not a general-purpose ontology editor. Protégé, LinkML, and OWL tooling exist for that; `dont` grows a project-local working vocabulary through use and supports one-way import from LinkML/OBO/etc. as a convenience.
- Not a knowledge base to be queried as a source of truth. The store exists to enforce doubt; reading it back is secondary.
- Not a bug catcher in its own right. It is a *checkpoint* at which external verifiers (SymPy, pytest, Lean, CAS sidecars) may be consulted; it owns none of them.
- Not an LLM wrapper. The tool does not call LLMs in its primary mode; it emits spawn-request payloads for the harness to execute.

---

## 3. Core principles *[revised]*

1. **The forcing is the product.** Every other feature — the store, the schema, the spawn protocol — is machinery in service of making refusal credible and remediation concrete.
2. **Refusals are affordances.** Every refusal carries a structured `remediation[]` the LLM can execute without re-reading documentation. An error without a remediation is a bug.
3. **Nothing is deleted.** All state transitions append to an immutable event log. Doubt is an event, not a deletion.
4. **`speak` and `define` are always unverified.** No flag, no import path, no privileged source bypasses this. `:verified` is reached only through `no-doubt` with evidence.
5. **Terms are first-class entities.** A CURIE coined through `define` enters the store as a doubtable entity, subject to the same status lattice as a claim. The vocabulary the LLM builds is itself data in the store.
6. **Imports are not claims.** Data brought in from external ontologies, registries, or papers populates separate relations and is not doubtable through the normal verbs. If an import is wrong, re-import; do not doubt.
7. **Structured output is the contract.** Every command supports `--json` with a versioned, stable envelope. Human output is a rendering of the same data; the two must not drift.
8. **`dont` delegates, does not embed.** Commands that need fresh reasoning (`verify`, `premortem`, `ask`) emit spawn requests for the harness to fulfill in a clean-context subagent. Direct LLM calls are a `--direct` fallback for CI and shells without spawn support.
9. **The LLM is the user.** The CLI surface, the error messages, the orientation block, the hint system — all are written for an LLM reader first. Human-readable rendering is secondary and will not be invested in beyond correctness.
10. **Tools are independent.** `dont`, `beads`, and `wai` share conventions (install-once binary, per-project directory, agent-addressed documentation, author identity strings) but no code and no configuration. A harness invokes each independently.

---

## 4. Architectural decisions *[revised]*

### 4.1 Language: Rust *[unchanged]*

Single-binary distribution, sub-50 ms cold start, embeddable store. Ontology-heavy work (SHACL, OWL, SPIRES-style extraction) stays out-of-process behind subprocess sidecars.

### 4.2 Storage: append-only event log with derived current-state view *[revised]*

The v0.1 draft specified CozoDB with datoms. That choice is **deferred**. For the priorities of v0.2 (grounding, refusal-with-remediation, spawn protocol), full bitemporality and Datalog at query time are not load-bearing. A simpler substrate — SQLite with an `events` table and materialised views for current state — covers the required operations:

- Append an event.
- Compute current status of any claim/term from its event history.
- Query claims/terms by current status.
- Reconstruct state at a past time for audit.

CozoDB remains a candidate if rule-level computation proves expensive to express in SQL; the interface (§5) is deliberately substrate-agnostic so the decision can be made late. The v0.1 arguments for datoms over triples remain valid and will be revisited if/when the substrate is re-chosen.

### 4.3 Data model: entities with kind, attributes, and history *[revised]*

All stored objects are **entities**. Each entity has:

- an `id` (ULID, prefixed per kind: `claim:`, `term:`, `event:`, `evidence:`),
- a `kind` attribute (`claim`, `term`, `hypothesis`, `evidence`, ...),
- a set of attribute assertions,
- a history of events (`created`, `asserted`, `doubted`, `no-doubted`, `locked`, ...).

Classes are not declared; membership is *recognised* by predicate match. "Is this entity a `gr:RicciTensor`?" is a rule evaluation, not a stored fact. This follows the Clojure/Datomic-style modelling discussion from the design pass: attributes are first-class and classes are queries.

The minimum-viable primitives the tool understands are described in §6; the seed vocabulary shipped with the binary is described in §7.

### 4.4 Distribution and initialisation *[mostly unchanged]*

Single static binary. `curl -fsSL https://.../install.sh | bash`. Per-project: `dont init [--strict]` creates `.dont/`, writes `.dont/AGENTS.md`, installs the seed vocabulary (§7), and runs `dont sync-docs` to inject the managed block into root-level agent docs.

`dont init` defaults to **permissive mode** (§8). `dont init --strict` starts the project in strict mode. Mode can be changed later via `config.toml`; the data shape is identical between modes.

---

## 5. Data model *[revised]*

### 5.1 Status lattice *[unchanged]*

```
:unverified ──doubt──▶ :doubted ──no-doubt(+evidence)──▶ :verified
      │                    ▲                                  │
      └──no-doubt(+evidence)┘                                  │
                                                               ▼
                                                          :locked (terminal)

(any) ── a dependency becomes :doubted ──▶ :stale  (auto-cascade)
:stale ── dependency resolves to :verified ──▶ previous status
```

Invariants:

- Claims and terms enter only via `speak`/`define` and only `:unverified`.
- `:locked` is terminal.
- `:stale` is auto-cascade.
- Seed vocabulary terms (§7) enter `:locked` on `dont init` and require an explicit unlock operation to doubt (reserved for future use).
- Every transition records author, timestamp, and reason/evidence.

### 5.2 Core relations (substrate-agnostic schematic)

```
entity       { id, kind, created_at, created_by }
attribute    { entity_id, name, value, tx }
event        { id, entity_id, kind, at, author,
               reason?, evidence_uri?, spawn_request_id? }
evidence     { id, claim_id, source_uri, kind, supports, quote? }
depends_on   { claim_id, dep_id }
```

Claim-specific attributes: `statement`, `status`, `confidence`, `provenance`, `atoms[]`, `refs[]`.
Term-specific attributes: `curie`, `definition`, `kind_of[]`, `related_to[]`, `provenance`.

`atoms[]` and `refs[]` are stored as repeated attribute rows, not JSON blobs, so that rules can reason over individual atoms.

### 5.3 Import relations

```
imported_term   { curie, label, definition, xrefs, source, imported_at }
reference       { uri, title, authors, year, source, imported_at }
prefix          { prefix, uri_base, canonical, imported_at }
```

Imports populate reference material; they are not entities in the core store and cannot be doubted through the normal verbs. `imported_term` and `term` (LLM-coined) are queried together when checking CURIE resolution, but they have different lifecycles.

---

## 6. Minimum-viable primitives *[new]*

The tool recognises exactly these concepts at the schema level. They are the vocabulary an LLM needs to participate in the forcing function; nothing more should be added without evidence that existing primitives cannot express the need.

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
- `dont:Evidence` — material cited in `no-doubt`.

*Relations:*

- `dont:kind_of` — "X is a kind of Y" for the purposes of rule matching. Deliberately looser than `rdfs:subClassOf`; carries no OWL semantics.
- `dont:related_to` — "X is related to Y without a stronger claim."
- `dont:defined_as` — attaches a prose definition to a term.

*Epistemic:*

- `dont:Hypothesis` — a claim offered for consideration. Weaker commitment than a `:unverified` claim.
- `dont:Retraction` — the event kind recording a `doubt` transition.

*External anchoring:*

- `dont:external_ref` — attaches a URI or CURIE pointing at something outside the store (paper, file, commit).

Terms explicitly **not** in the seed: `owl:Thing`, `rdfs:subClassOf`, `skos:definition` (pull in unwanted semantics); provenance/author terms (these are event attributes, not vocabulary); confidence or probability terms (too theory-laden); domain-specific terms (that is what `define` and `import` are for).

Import adapters may provide one-way mappings from `owl:Thing`/`rdfs:subClassOf`/etc. into the seed terms for projects bringing in external vocabularies.

---

## 8. Modes *[new]*

`dont` operates in one of two modes, set at `init` and changeable in `config.toml`. The **invariant** is the same in both modes: no `:verified` entity may reference an unresolved CURIE. The modes differ only in *when* the check runs.

### 8.1 Permissive mode (default)

- `speak` accepts claims whose CURIEs do not resolve. The claim enters `:unverified` with an `:ungrounded-term` marker per offending CURIE listed in `warnings[]`.
- A rule (`unresolved-terms`) blocks `no-doubt` until every CURIE in the claim resolves.
- `dont prime` and `dont claims --status unverified` surface pending-grounding work prominently so the LLM is prompted to return and complete it.
- The LLM can make progress on a first call; it cannot shortcut the grounding requirement when it matters (at `:verified`).

### 8.2 Strict mode

- The `ungrounded` rule fires at `speak` time, refusing the claim.
- Nothing enters the store until every CURIE resolves.
- Use case: mature projects with a settled vocabulary, or domains where even `:unverified` storage of ungrounded claims is undesirable.

### 8.3 Constraint that binds both modes

`define` **always** forbids references to undefined terms, regardless of mode. A definition with an unresolved parent or related-to reference is not a term; it is an intention to define one. This is the one place where permissive mode is not permissive: the vocabulary skeleton must be built coherently or not at all.

Rationale: if definitions could dangle, the bootstrap problem becomes unbounded — the LLM could define `A` in terms of `B`, `B` in terms of `C`, and never resolve anything. Forbidding dangling references forces each `define` call to either reference existing terms or introduce a self-contained term (no `--kind-of`, no `--related-to`).

### 8.4 Mode migration

A project in permissive mode can switch to strict at any time by editing `config.toml`. Existing `:verified` claims already satisfy strict's precondition by the invariant above. Existing `:unverified` claims with unresolved CURIEs become visible as work-to-do but do not retroactively fail. The data shape is unchanged; only the gating of *new* `speak` calls changes.

---

## 9. Primary CLI: four verbs *[revised]*

### 9.1 `dont speak`

```
dont speak "<statement>"
        [--atom "<sub-statement>"]*
        [--ref <uri|curie>]*
        [--confidence <0..1>]
        [--depends-on <claim-id>]*
        [--session <id>]
        [--author <id>]
        [--json]
```

Creates a `claim` entity with `status = :unverified`. Records atoms, refs, dependencies. Emits a `:spoken` event. Returns a `ClaimView` (§10.3).

In strict mode: refuses if any CURIE in statement or atoms does not resolve.
In permissive mode: accepts with `warnings[]` listing unresolved CURIEs; `unresolved-terms` rule will later block `no-doubt`.

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

Terms promote to `:verified` the same way claims do — `no-doubt` with evidence. Evidence for a term might be a link to an authoritative source (a paper establishing the concept, a section of a specification, a reviewer's approval). Terms can be `:doubted` and `:stale`-cascaded like claims; when a term is doubted, every claim referencing it cascades to `:stale`.

### 9.3 `dont doubt`

```
dont doubt <entity-id>
        --reason "<text>"                    # required
        [--atom <idx>]
        [--author <id>]
        [--json]
```

Retracts the current `:status` of a claim or term, asserts `:doubted`, records reason. Triggers `stale-cascade` on dependents. Refuses if `:locked`. Refuses if reason is empty or consists only of hedges (a `vague-reason` rule, §13).

### 9.4 `dont no-doubt`

```
dont no-doubt <entity-id>
        --evidence <uri|curie>               # required, repeatable
        [--quote "<excerpt>"]
        [--note "<text>"]
        [--author <id>]
        [--json]
```

Transitions to `:verified`, records evidence rows, emits `:no-doubted` event. Refuses without `--evidence`, if `:locked`, if evidence URIs are unresolvable, or if `unresolved-terms` rule still flags the entity.

Applies equally to claims and terms. The semantics of what counts as evidence differ (a claim's evidence is usually a paper or experiment; a term's evidence is usually a definitional authority), but the verb is the same.

---

## 10. Derived commands and envelope *[revised]*

Orchestrations over the four primitives. In harness mode (default when `DONT_HARNESS` is set or when invoked through an LLM tool-use channel), commands that need new reasoning emit **spawn requests**; they do not call an LLM. In `--direct` mode they call the configured provider directly.

```
dont ask "<question>" [--n 3] [--temperature 0.7] [--json]
dont verify <entity-id> [--json]
dont premortem <entity-id> [--json]
dont lock <entity-id> [--json]

dont claims [--status <s>] [--as-of <ts>] [--json]
dont terms  [--status <s>] [--as-of <ts>] [--json]
dont show <entity-id> [--history] [--json]
dont why  <entity-id> [--json]

dont suggest-evidence <claim-id> [--json]
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
dont import <source> ...
```

### 10.1 `dont suggest-term` *[new]*

Before coining a new term, the LLM should run `dont suggest-term "<rough concept>"`. The command searches:

1. The local `term` table (terms already coined in this project).
2. The local `imported_term` table (terms imported from OLS, LinkML schemas, etc.).
3. Optionally, enabled import sources via their HTTP APIs.

Returns a ranked list. The LLM is then expected to either pick an existing term or, if nothing matches, proceed with `define`. The forcing function here is soft — the tool does not require `suggest-term` to have been run before `define` — but the orientation block and the `prime` output recommend it, and a future rule (`unsearched-before-coining`, opt-in) could enforce it for projects that want stronger vocabulary hygiene.

### 10.2 Envelope *[unchanged from v0.1 §9]*

The `--json` envelope, `ClaimView`, `TermView` *[new shape, parallels ClaimView]*, `EventView`, `SpawnRequest`, `PrimeView`, `WhyView`, error envelope with `remediation[]`, identity conventions, exit codes — all carry over from v0.1 §9 without change. The only addition is `TermView` as a peer to `ClaimView`, identical in shape except for term-specific attributes (`curie`, `definition`, `kind_of`, `related_to` instead of `statement`, `atoms`, `refs`, `depends_on`).

---

## 11. Self-teaching and harness-integration surface *[mostly unchanged]*

Sections 8.1 through 8.11 of v0.1 carry over with two edits:

- Every reference to "a claim" or "the claim" is read as "an entity" where the verb applies equally to terms.
- §8.9 (agent-addressed docs) is reframed: `.dont/AGENTS.md` is primary and canonical; root-level managed blocks still exist but are shorter, because the LLM now reads `dont prime --json` at session start and rarely reads the markdown files directly.

The spawn-request protocol (§8.11 in v0.1) is reframed in §12 here.

### 11.1 Orientation block for the LLM

The v0.1 §11.5 minimum-viable orientation prompt is updated to four verbs and to mention modes:

```
You have access to `dont`, a tool that forces grounding before you
can assert claims. You are the user; always pass --json.

Four verbs:
  dont.speak    — introduce a claim (one declarative sentence,
                  decomposed into atoms). Enters :unverified.
  dont.define   — coin a vocabulary term. Requires a prose definition.
                  Parent/related references must already resolve.
  dont.doubt    — challenge a claim or term with a specific reason.
  dont.no-doubt — affirm a claim or term with at least one
                  resolvable evidence URI.

Mode: permissive (default) or strict.
  - Permissive: `speak` accepts unresolved CURIEs with warnings;
    `no-doubt` requires them resolved.
  - Strict: `speak` refuses unresolved CURIEs up front.
  - `define` forbids unresolved references in both modes.

On refusal:
  Read data.remediation[0].command and run it. Then retry.
  Do not guess reformulations. Do not apologise.

On a spawn-request envelope:
  Invoke your harness's subagent mechanism with context.prompt and
  allowed_tools. Do not perform the verification yourself.

Before coining a new term: run `dont suggest-term "<concept>"` first.

For full docs: `dont help <cmd>` or `.dont/AGENTS.md`.
```

---

## 12. Spawn-request protocol *[revised]*

Unchanged in shape from v0.1 §8.11, but reframed: MCP is not privileged. The protocol is *harness-agnostic*: any harness that can read structured output from a tool call and start a subagent can implement it. Concrete transports:

- **Direct CLI, harness reads stdout/stderr.** The structured envelope arrives on stdout as JSON; the harness parses it, spots `kind: "spawn-request"`, and invokes its own subagent mechanism.
- **MCP (optional).** A `dont mcp` server mode exposes the same commands as MCP tools; the envelope is the tool's return value. This is one transport among several, not the privileged surface.
- **Shell pipelines.** In `--direct` mode the tool calls a configured LLM provider itself; useful for CI and for harnesses without subagent support.

The harness contract (clean-context subagent, restricted tool set, terminal-only `no-doubt`/`doubt` callbacks, `:spawn-requested`/`:spawn-resolved` event pairs) is unchanged.

---

## 13. Methodology as rules *[revised]*

Rules live in `.dont/rules/*.dl` (Cozo Datalog if that substrate is chosen) or `.dont/rules/*.sql` (if SQLite is the substrate). Each has a sibling `.md` with its English translation. Rules shipped by default:

**`ungrounded`** — flags or refuses claims referencing unresolved CURIEs.

**`unresolved-terms`** — blocks `no-doubt` on a claim whose CURIEs have not all resolved. Always strict. The invariant that permits permissive mode to be permissive at `speak` time.

**`stale-cascade`** — on any `:doubted` transition, cascade dependents (claims depending on a doubted claim, claims using a doubted term) to `:stale`.

**`lockable`** — `:verified`, ≥3 assessed hypotheses, ≥2 independent supporting evidence items. Precondition for `dont lock`.

**`correlated-error`** — flags claims whose only evidence shares a source with the author.

**`vague-reason`** — flags `doubt` calls whose `--reason` matches hedge patterns (`"might be wrong"`, `"not sure"`, `"probably incorrect"`) without a specific defect. Warn by default; users can promote to strict.

**`dangling-definition`** — refuses `define` calls whose `--kind-of` or `--related-to` references do not resolve. Always strict, in both modes. Enforces §8.3.

Rule severity defaults:

- Strict in both modes: `unresolved-terms`, `dangling-definition`, `stale-cascade`.
- Strict in strict mode, warn in permissive mode: `ungrounded`.
- Warn in both modes by default: `correlated-error`, `vague-reason`.
- Manual-trigger only: `lockable` (runs on `dont lock`).

`config.toml` overrides these on a per-project basis.

---

## 14. Project layout *[revised]*

```
.dont/
  db.sqlite                 # (or db.cozo if substrate changes)
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
    vague-reason.dl         vague-reason.md
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
warn   = ["correlated-error", "vague-reason"]
# `ungrounded` is auto-set by mode: strict in strict mode, warn in permissive.

[import]
ols       = { enabled = true,  base = "https://www.ebi.ac.uk/ols4" }
wikidata  = { enabled = true,  endpoint = "https://query.wikidata.org/sparql" }
openalex  = { enabled = true,  base = "https://api.openalex.org" }
linkml    = { enabled = true }   # subprocess adapter, shells out to linkml CLI
```

---

## 15. Import *[revised]*

Small handlers that fetch over plain HTTP or read a file and project into local relations. No LLM involvement. No MCP.

```
dont import obo <path.owl|.obo|.ttl>
dont import ols <ontology-prefix>
dont import wikidata --entity <Qid> | --sparql <file.rq>
dont import openalex --work <doi> | --snapshot <path>
dont import bioregistry
dont import jsonld <file>
dont import ttl <file>
dont import linkml <schema.yaml>         # [revised] subprocess, one-way, lossy
```

Writes to `imported_term`, `reference`, or `prefix`. Idempotent per source URI.

**LinkML import** shells out to the Python `linkml` CLI (`gen-json-schema`, `gen-owl`) to produce intermediate forms, then lowers the intermediate into `imported_term` rows and (optionally) generates Datalog rules in `.dont/rules/imported-linkml-<name>.dl` from the SHACL output. The adapter is explicitly **lossy**: inheritance is flattened into `kind_of` chains; `slot_usage` is expanded into per-class attribute predicates; mixins are flattened. Projects that need LinkML's full semantics should use LinkML directly; `dont`'s adapter is for grounding, not for round-tripping.

---

## 16. MCP interface *[revised]*

*Optional.* `dont mcp` runs the tool as an MCP server over stdio, exposing the same commands as MCP tools. Every tool returns the `Envelope` JSON as its result. This exists for harnesses that prefer MCP over direct CLI calls; it is not the privileged surface. Most harnesses invoking `dont` through a generic shell tool will not use MCP at all.

---

## 17. Out of scope for v0 *[revised]*

- Web UI.
- Multi-user concurrent editing and merge.
- OWL reasoning, SHACL validation (beyond what the LinkML import adapter produces as Datalog).
- SPIRES-style LLM extraction.
- Full bitemporality. The store is transaction-time only; valid-time is deferred.
- Schema migration tooling.
- Authentication / authorization.
- Encryption at rest.
- A unified identity service across `dont`/`beads`/`wai`. A shared author-string convention is enough.

---

## 18. Open questions *[revised]*

Carried from v0.1 and updated:

1. **Substrate choice (SQLite vs CozoDB).** Deferred. Interface is substrate-agnostic.
2. **Rule language surface.** Raw Datalog/SQL vs a thin DSL. Start raw; add sugar when patterns repeat.
3. **Confidence calibration.** LLM-reported confidences are miscalibrated. Platt-scale against historical `verify` outcomes, or leave raw?
4. **Cascades on `no-doubt`.** `:stale` dependents auto-revive or stay stale? Leaning stay-stale.
5. **Evidence liveness checks.** On-command for `no-doubt`, nightly sweep for the rest.
6. **`DONT_HARNESS` detection.** Env-var set by harness, fallback autosniff, `--direct` to opt out.
7. **Spawn-request failure modes.** `dont spawns --pending` + auto-timeout.
8. **`--json` schema versioning.** Envelopes carry `dont_version`. Migration story TBD.
9. **Streaming vs. buffered JSON.** NDJSON behind `--json-stream`, or cursor pagination.
10. **Spawn-request prompt stability.** Version the prompt template and log the version alongside the request.

New in v0.2:

11. **Seed vocabulary churn.** If the seed changes between `dont` versions, what happens to projects that coined terms against the old seed? Versioning of the seed, or lock at `init` time?
12. **`suggest-term` enforcement.** Opt-in rule `unsearched-before-coining`: does the cost of tracking pre-coining searches justify catching vocabulary duplication?
13. **Mode change as an event.** When a project switches permissive→strict, should that be a recorded event with an author, so audit can see "the tool got stricter on 2026-05-01"?
14. **LinkML import adapter scope.** Which subset of LinkML is faithfully supported, and what is the failure mode on unsupported features — refuse to import, import with warnings, or partially import?

---

## 19. Evaluation questions *[revised]*

- Does the four-verb API (`speak` / `define` / `doubt` / `no-doubt`) cover every state transition the workflow actually needs?
- Is the permissive/strict mode distinction the right axis, or are there orthogonal axes (e.g. "evidence liveness required vs not") that deserve their own dimension?
- Are the ten seed-vocabulary terms enough, too many, or is something missing?
- Is the refusal-with-remediation surface expressive enough that an LLM with no prior context can recover from every refusal without human intervention?
- Is §12's spawn protocol implementable by `wai`, `beads`-adjacent harnesses, Claude Code, Cursor, and generic shell-based loops without meaningful variation?
- Does the §9/10 envelope contract remain stable enough across the v0.1→v0.2 changes that downstream adapters don't break?
- Does the LinkML import adapter's lossiness cause surprises in practice, or is the documented lossy behaviour acceptable to the target users?
- Can a researcher using a `wai`- or `beads`-backed harness complete a realistic session (coin terms, assert claims, verify through spawns, lock what deserves locking) without ever reading the spec?

---

## 20. References *[unchanged from v0.1 §19]*

The reference list from v0.1 §19 carries over in full. The behavioural literature on LLM sycophancy and in-context defence still motivates the forcing function; the verification-methods literature still informs the spawn protocol and the rule catalogue; the institutional analogs still ground the design; the data-modelling references still cover the datom/event-log lineage even under the substrate deferral in §4.2.

---

## 21. Changelog from v0.1 *[new]*

**Scope narrowed.** The tool is now a forcing function, not a knowledge substrate. Bug prevention, schema-system-ness, and claim-store-as-product are demoted to consequences of the forcing mechanic rather than pillars.

**User identified.** The LLM is the user. Human-readable output is secondary.

**MCP demoted.** MCP is one optional transport. The harness's tool-use channel is the primary surface, transport-agnostic.

**Fourth verb.** `define` joins `speak`/`doubt`/`no-doubt`. Terms are first-class doubtable entities.

**Seed vocabulary.** Ten locked terms shipped on `init`.

**Modes.** Permissive (default) and strict, differing only in when the grounding check runs.

**Schema redesign.** Five primitives (attribute/derived_class/enum/prefix/rule), no inheritance, recognition by predicate. LinkML is an import adapter, not a dependency.

**Substrate deferred.** SQLite is the default working assumption; CozoDB/datom remains a candidate for later if Datalog expressiveness is needed.

**`suggest-term`, `terms`, `dangling-definition` rule, `vague-reason` rule, `unresolved-terms` rule.** New surfaces supporting the four-verb core.

**Tool independence.** `dont`, `beads`, `wai` are explicitly peer-and-independent. Shared conventions, no shared code or config.
