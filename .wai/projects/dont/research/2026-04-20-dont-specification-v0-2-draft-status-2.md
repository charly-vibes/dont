# `dont` — specification (v0.2 draft)

**Status:** design draft, not implemented. This revision replaces the v0.1 draft after a round of value-proposition work that substantially narrowed the tool's scope and reshaped several core decisions. Sections marked *[revised]* differ materially from v0.1; sections marked *[new]* are additions; sections marked *[unchanged]* carry over intact.

---

## 1. Purpose *[revised]*

`dont` is a **forcing function** for autonomous LLM harnesses. It is a per-project command-line tool that an LLM calls directly, as a peer tool alongside harness-provided memory (`beads`) and workflow (`wai`) tools. Its single job is to interrupt the LLM's default behaviour of confidently asserting unchecked claims, and to give the LLM a structured, mechanical path to earn the right to assert.

The problem it addresses, distilled:

- LLMs do not reliably self-correct. Lift comes from external signal.
- Once a claim enters the context window, the autoregressive substrate makes it cheaper to defend than to retract.
- Durable knowledge-producing institutions (peer review, CI gates, the nuclear two-person rule) solve this by separating generator from evaluator and by making retraction a first-class event.

`dont` implements that separation as a minimal CLI with four verbs, an append-only event log, and a refusal protocol whose error messages *are the user interface*. When the LLM tries to assert something it has not adequately grounded, `dont` refuses with a structured remediation the LLM can act on without human involvement. When the LLM asks for verification, `dont` emits a structured instruction for the harness to spawn a clean-context subagent whose only terminal actions are `dismiss` (with evidence) or `trust` (with reason).

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
4. **`conclude` and `define` are always unverified.** No flag, no import path, no privileged source bypasses this. `:verified` is reached only through `dismiss` with evidence.
5. **Terms are first-class entities.** A CURIE coined through `define` enters the store as a doubtable entity, subject to the same status lattice as a claim. The vocabulary the LLM builds is itself data in the store.
6. **Imports are not claims.** Data brought in from external ontologies, registries, or papers populates separate relations and is not doubtable through the normal verbs. If an import is wrong, re-import; do not doubt.
7. **Structured output is the contract.** Every command supports `--json` with a versioned, stable envelope. Human output is a rendering of the same data; the two must not drift.
8. **`dont` delegates, does not embed.** Commands that need fresh reasoning (`assume`, `overlook`, `guess`) emit spawn requests for the harness to fulfill in a clean-context subagent. Direct LLM calls are a `--direct` fallback for CI and shells without spawn support.
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
:unverified ──trust──▶ :doubted ──dismiss(+evidence)──▶ :verified
      │                    ▲                                  │
      └──dismiss(+evidence)┘                                  │
                                                               ▼
                                                          :locked (terminal)

(any) ── a dependency becomes :doubted ──▶ :stale  (auto-cascade)
:stale ── dependency resolves to :verified ──▶ previous status
```

Invariants:

- Claims and terms enter only via `conclude`/`define` and only `:unverified`.
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

### 8.4 Mode migration

A project in permissive mode can switch to strict at any time by editing `config.toml`. Existing `:verified` claims already satisfy strict's precondition by the invariant above. Existing `:unverified` claims with unresolved CURIEs become visible as work-to-do but do not retroactively fail. The data shape is unchanged; only the gating of *new* `conclude` calls changes.

---

## 9. Primary CLI: four verbs *[revised]*

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

### 9.3 `dont trust`

```
dont trust <entity-id>
        --reason "<text>"                    # required
        [--atom <idx>]
        [--author <id>]
        [--json]
```

Retracts the current `:status` of a claim or term, asserts `:doubted`, records reason. Triggers `stale-cascade` on dependents. Refuses if `:locked`. Refuses if reason is empty or consists only of hedges (a `vague-reason` rule, §13).

### 9.4 `dont dismiss`

```
dont dismiss <entity-id>
        --evidence <uri|curie>               # required, repeatable
        [--quote "<excerpt>"]
        [--note "<text>"]
        [--author <id>]
        [--json]
```

Transitions to `:verified`, records evidence rows, emits `:dismissed` event. Refuses without `--evidence`, if `:locked`, if evidence URIs are unresolvable, or if `unresolved-terms` rule still flags the entity.

Applies equally to claims and terms. The semantics of what counts as evidence differ (a claim's evidence is usually a paper or experiment; a term's evidence is usually a definitional authority), but the verb is the same.

---

## 10. Derived commands and envelope *[revised]*

Orchestrations over the four primitives. In harness mode (default when `DONT_HARNESS` is set or when invoked through an LLM tool-use channel), commands that need new reasoning emit **spawn requests**; they do not call an LLM. In `--direct` mode they call the configured provider directly.

```
dont guess "<question>" [--n 3] [--temperature 0.7] [--json]
dont assume <entity-id> [--json]
dont overlook <entity-id> [--json]
dont reopen <entity-id> [--json]

dont list [--status <s>] [--as-of <ts>] [--json]
dont vocab  [--status <s>] [--as-of <ts>] [--json]
dont show <entity-id> [--history] [--json]
dont why  <entity-id> [--json]

dont ignore <claim-id> [--json]
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

### 10.2 Communication schema (full detail)

All machine-parseable output across the CLI and MCP surfaces follows the same envelope. This section is the contract.

### 9.1 Envelope

```json
{
  "dont_version": "0.1",
  "ok": true,
  "kind": "claim",
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

- `dont_version` — CLI version. Envelopes are stable within a major version; minor versions add fields but do not remove or rename.
- `ok` — `true` for success, `false` for refusal or error. On `false`, `data` is the `ErrorResult` shape (§10.4) and `kind` is `"error"`.
- `kind` — discriminator for `data`: `claim`, `claims`, `event`, `events`, `spawn-request`, `spawn-requests`, `rule`, `rule-result`, `prime`, `why`, `empty`, `error`.
- `data` — the typed payload.
- `hints` — ordered array of `{command, description}`. Safe to ignore; useful for agents.
- `warnings` — rule flags triggered during the operation, `{rule, claim_id?, message, suggested_remediation?}`.
- `meta` — execution metadata. `tx` is the transaction id that applied any mutations (`null` for read-only commands). `request_id` is populated when the command resolves a pending spawn.

In `--json` mode the envelope is the only thing on stdout. Human logging goes to stderr.

### 10.2 Identity and format conventions

- **Claim IDs** are ULIDs prefixed `claim:` — `claim:01HX05A9…`. Lexicographically sortable, timestamp-embedded.
- **Spawn request IDs** prefixed `spawn:`.
- **Rule names**, **event kinds**, **status values**: lower-kebab-case strings.
- **Timestamps**: RFC 3339 in UTC — `2026-04-18T14:20:00Z`.
- **Validity**: `[timestamp, assertion_bool]`.

### 10.3 Core payload types

**`ClaimView`** (`kind: "claim"`):

```json
{
  "id": "claim:01HX05A9K8VP",
  "statement": "CRISPR-Cas9 causes off-target edits in human cells",
  "status": "verified",
  "confidence": 0.78,
  "atoms": [
    {"idx": 0, "text": "CRISPR edits DNA", "verified": true},
    {"idx": 1, "text": "off-target edits have been observed", "verified": true}
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
    "lockable":         {"met": false, "unmet": ["needs >=3 hypotheses; has 1"]},
    "correlated-error": {"flagged": false},
    "ungrounded":       {"flagged": false}
  }
}
```

**`EventView`** (`kind: "event"`):

```json
{
  "claim_id": "claim:01HX05…",
  "tx": 42,
  "kind": "trusted",
  "at": "2026-04-15T09:34:00Z",
  "author": "human:sasha",
  "reason": "atom 3 conflates correlation with causation",
  "evidence_uri": null,
  "spawn_request_id": null
}
```

**`SpawnRequest`** (`kind: "spawn-request"`): the structured subagent instruction (§8.11). In `--json` mode, the same YAML content lives as JSON fields; no HTML comment wrapping.

```json
{
  "request_id": "spawn:01HX0A3K8P",
  "kind": "assume",
  "claim_id": "claim:01HX05…",
  "context": {
    "clean": true,
    "prompt": "You are an independent verifier…",
    "allowed_tools": ["dont.dismiss", "dont.trust", "dont.show", "web_search"],
    "forbidden_tools": ["dont.conclude", "dont.reopen"],
    "model_hint": "verifier",
    "max_tool_calls": 20
  },
  "return_to": "claude-code",
  "issued_at": "2026-04-18T14:20:00Z"
}
```

**`PrimeView`** (`kind: "prime"`): structured project orientation.

```json
{
  "project": "research-demo",
  "state": {"unverified": 23, "doubted": 11, "verified": 10, "locked": 3, "stale": 0},
  "rules": {"strict": ["lockable", "ungrounded"], "warn": ["correlated-error"]},
  "ontologies": [
    {"prefix": "GO", "refreshed": "2026-04-15T00:00:00Z"},
    {"prefix": "HPO", "refreshed": "2026-04-15T00:00:00Z"}
  ],
  "blocking": [
    {"id": "claim:01HWY9…", "statement": "…", "status": "doubted"}
  ],
  "harness_mode": true,
  "invariants": [
    "conclude always creates :unverified",
    "dismiss requires --evidence",
    ":locked is terminal"
  ]
}
```

**`WhyView`** (`kind: "why"`): claim + full event timeline + applicable rules + remediation.

**`ClaimsList`** (`kind: "claims"`):

```json
{
  "as_of": "2026-04-18T14:20:00Z",
  "count": 47,
  "claims": [ /* ClaimView */ ]
}
```

Full JSON Schemas for each type are shipped with the binary; `dont schema <name>` prints them.

### 10.4 Error envelope

On refusal or error: `ok: false`, `kind: "error"`, `data` takes the `ErrorResult` shape:

```json
{
  "code": "no-evidence",
  "message": "dismiss requires at least one --evidence URI",
  "rule": "§6.3",
  "claim_id": "claim:01HX05…",
  "unmet_clauses": [
    {"clause": "at least one --evidence flag", "fix": "--evidence <uri>"}
  ],
  "remediation": [
    {"command": "dont ignore claim:01HX05",
     "description": "find candidate evidence"},
    {"command": "dont dismiss claim:01HX05 --evidence <uri>",
     "description": "supply evidence"}
  ]
}
```

Error codes are stable lowercase-kebab strings. A partial list: `no-evidence`, `claim-not-found`, `claim-locked`, `rule-not-met`, `unresolvable-uri`, `reason-required`, `schema-mismatch`, `db-locked`, `config-missing`.

### 10.5 Input schemas

CLI args map 1:1 to structured payloads. In MCP mode, tools accept the payload directly. Both are validated against the same JSON Schema.

```
ConcludeInput = {statement, atoms?, refs?, confidence?, depends_on?,
                 session_id?, author?, origin?}
TrustInput    = {claim_id, reason, atom_idx?, author?}
DismissInput  = {claim_id, evidence: [{uri, kind?, quote?, supports?}],
                 note?, author?}
AssumeInput   = {claim_id, model_hint?, max_tool_calls?}
GuessInput      = {question, n?, temperature?}
ImportInput   = {source, …}  # varies by importer
```

---

### TermView (v0.2 addition)

`TermView` is a peer to `ClaimView` (`kind: "term"`). Identical envelope shape, different data fields:

```json
{
  "id": "term:01HX07…",
  "curie": "proj:RicciTensor",
  "definition": "A symmetric (0,2) tensor encoding intrinsic curvature of a Riemannian manifold.",
  "kind_of": ["dont:Term"],
  "related_to": ["proj:MetricTensor"],
  "status": "unverified",
  "confidence": null,
  "provenance": { "author": "llm:claude-opus-4.7", "origin": "llm" },
  "created_at": "2026-04-18T09:00:00Z",
  "applicable_rules": {
    "lockable": {"met": false, "unmet": ["needs >=3 hypotheses"]},
    "unresolved-terms": {"flagged": false}
  }
}
```

---

## 11. Self-teaching and harness-integration surface *[mostly unchanged]*

The self-teaching surface from v0.1 (three-part errors, `dont prime`, `dont why`, `dont explain`, `dont examples`, `dont ignore`, contextual hints, self-correcting refusals, agent-addressed docs, `dont help`/`dont doctor`/`dont schema`) carries over with two changes:

- Every reference to "a claim" or "the claim" applies equally to terms.
- `.dont/AGENTS.md` is now primary; root-level managed blocks remain but are shorter, because the LLM reads `dont prime --json` at session start.

The managed block template (updated for four verbs):

```markdown
<!-- dont-managed:start -->
## Claim and term management — `dont`

This project uses `dont` to track knowledge claims and coined terms with enforced doubt.

- Run `dont prime --json` at session start.
- Full usage: [`.dont/AGENTS.md`](./.dont/AGENTS.md) or `dont help`.
- Core verbs: `dont conclude`, `dont define`, `dont trust`, `dont dismiss`.
- `assume` / `overlook` / `guess` emit spawn requests.
- Prefer `--json` for parsing.

This block is managed by `dont`. Edits inside the markers will be overwritten on `dont sync-docs`.
<!-- dont-managed:end -->
```

The spawn-request protocol is reframed in §12.

### 11.1 Orientation block for the LLM

The v0.1 §11.5 minimum-viable orientation prompt is updated to four verbs and to mention modes:

```
You have access to `dont`, a tool that forces grounding before you
can assert claims. You are the user; always pass --json.

Four verbs:
  dont.conclude  — introduce a claim (one declarative sentence,
                  decomposed into atoms). Enters :unverified.
  dont.define   — coin a vocabulary term. Requires a prose definition.
                  Parent/related references must already resolve.
  dont.trust    — challenge a claim or term with a specific reason.
  dont.dismiss — affirm a claim or term with at least one
                  resolvable evidence URI.

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
```

---

## 12. Spawn-request protocol *[revised]*

Unchanged in shape from v0.1 §8.11, but reframed: MCP is not privileged. The protocol is *harness-agnostic*: any harness that can read structured output from a tool call and start a subagent can implement it. Concrete transports:

- **Direct CLI, harness reads stdout/stderr.** The structured envelope arrives on stdout as JSON; the harness parses it, spots `kind: "spawn-request"`, and invokes its own subagent mechanism.
- **MCP (optional).** A `dont mcp` server mode exposes the same commands as MCP tools; the envelope is the tool's return value. This is one transport among several, not the privileged surface.
- **Shell pipelines.** In `--direct` mode the tool calls a configured LLM provider itself; useful for CI and for harnesses without subagent support.

The harness contract (clean-context subagent, restricted tool set, terminal-only `dismiss`/`trust` callbacks, `:spawn-requested`/`:spawn-resolved` event pairs) is unchanged.

---

## 13. Methodology as rules *[revised]*

Rules live in `.dont/rules/*.dl` (Cozo Datalog if that substrate is chosen) or `.dont/rules/*.sql` (if SQLite is the substrate). Each has a sibling `.md` with its English translation. Rules shipped by default:

**`ungrounded`** — flags or refuses claims referencing unresolved CURIEs.

**`unresolved-terms`** — blocks `dismiss` on a claim whose CURIEs have not all resolved. Always strict. The invariant that permits permissive mode to be permissive at `conclude` time.

**`stale-cascade`** — on any `trust` transition, cascade dependents (claims depending on a doubted claim, claims using a doubted term) to `:stale`.

**`lockable`** — `:verified`, ≥3 assessed hypotheses, ≥2 independent supporting evidence items. Precondition for `dont reopen`.

**`correlated-error`** — flags claims whose only evidence shares a source with the author.

**`vague-reason`** — flags `trust` calls whose `--reason` matches hedge patterns (`"might be wrong"`, `"not sure"`, `"probably incorrect"`) without a specific defect. Warn by default; users can promote to strict.

**`dangling-definition`** — refuses `define` calls whose `--kind-of` or `--related-to` references do not resolve. Always strict, in both modes. Enforces §8.3.

Rule severity defaults:

- Strict in both modes: `unresolved-terms`, `dangling-definition`, `stale-cascade`.
- Strict in strict mode, warn in permissive mode: `ungrounded`.
- Warn in both modes by default: `correlated-error`, `vague-reason`.
- Manual-trigger only: `lockable` (runs on `dont reopen`).

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
3. **Confidence calibration.** LLM-reported confidences are miscalibrated. Platt-scale against historical `assume` outcomes, or leave raw?
4. **Cascades on `dismiss`.** `:stale` dependents auto-revive or stay stale? Leaning stay-stale.
5. **Evidence liveness checks.** On-command for `dismiss`, nightly sweep for the rest.
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

- Does the four-verb API (`conclude` / `define` / `trust` / `dismiss`) cover every state transition the workflow actually needs?
- Is the permissive/strict mode distinction the right axis, or are there orthogonal axes (e.g. "evidence liveness required vs not") that deserve their own dimension?
- Are the ten seed-vocabulary terms enough, too many, or is something missing?
- Is the refusal-with-remediation surface expressive enough that an LLM with no prior context can recover from every refusal without human intervention?
- Is §12's spawn protocol implementable by `wai`, `beads`-adjacent harnesses, Claude Code, Cursor, and generic shell-based loops without meaningful variation?
- Does the §9/10 envelope contract remain stable enough across the v0.1→v0.2 changes that downstream adapters don't break?
- Does the LinkML import adapter's lossiness cause surprises in practice, or is the documented lossy behaviour acceptable to the target users?
- Can a researcher using a `wai`- or `beads`-backed harness complete a realistic session (coin terms, assert claims, verify through spawns, lock what deserves locking) without ever reading the spec?

---

## 20. References and learning material *[unchanged from v0.1]*

This section gathers the prior work that underpins `dont`. Entries are grouped by what they illuminate and carry a one-line note on relevance to the tool. The goal is to let a reader (or contributor) understand the *why* of the design without reconstructing the argument from scratch. Where a paper has an arXiv preprint and a venue publication, the arXiv ID is listed for accessibility.

### 19.1 Why LLMs defend what they have said

The core behavioral problem `dont` exists to work around.

- **Perez et al.**, *Discovering Language Model Behaviors with Model-Written Evaluations*, arXiv:2212.09251, 2022. Establishes sycophancy and inverse scaling of honesty under RLHF — the statistical substrate of "defend earlier answers."
- **Sharma et al.**, *Towards Understanding Sycophancy in Language Models*, arXiv:2310.13548, ICLR 2024. Frontier models (Claude, GPT-4, LLaMA-2) flip correct answers under mild user pushback; preference models reward flattering-but-wrong replies.
- **Turpin et al.**, *Language Models Don't Always Say What They Think*, arXiv:2305.04388, NeurIPS 2023. Chain-of-thought is rationalization, not introspection. Directly motivates why `dont assume` spawns a clean-context subagent instead of asking the original session to reconsider.
- **Anthropic**, *Reasoning Models Don't Always Say What They Think*, March 2025. Same unfaithfulness persists in extended-CoT reasoning models; the problem is not solved by more thinking tokens.
- **Ranzato et al.**, *Sequence Level Training with Recurrent Neural Networks*, arXiv:1511.06732, 2015. The foundational account of exposure bias — models teacher-forced on ground truth at training but conditioned on their own samples at inference compound early errors.
- **Huang et al.**, *Large Language Models Cannot Self-Correct Reasoning Yet*, arXiv:2310.01798, 2023. Intrinsic self-correction of reasoning either fails or degrades performance without an external signal.
- **Stechly et al.** (multiple 2023–24 papers on planning and self-critique) and **Kamoi et al.**, *Evaluating LLMs at Detecting Errors in LLM Responses*, 2024. Empirical corroboration that self-verification on reasoning is unreliable; strong external verifiers produce almost all the real gains.

### 19.2 Methods that produce real verification

External-signal methods whose patterns `dont`'s derived commands and rules encode procedurally.

- **Wang et al.**, *Self-Consistency Improves Chain of Thought Reasoning in Language Models*, arXiv:2203.11171, 2022. Majority-voting across diverse samples. The pattern behind `dont guess --n`.
- **Dhuliawala et al.**, *Chain-of-Verification Reduces Hallucination in Large Language Models*, arXiv:2309.11495, 2023. Decompose → independently verify → synthesize. The canonical template behind `dont assume` and the factored-evidence part of the claim schema.
- **Lightman et al.**, *Let's Verify Step by Step*, arXiv:2305.20050, 2023. Process reward models beat outcome reward models; released PRM800K. The argument for rule-level (not just final-answer-level) gating.
- **Yao et al.**, *Tree of Thoughts: Deliberate Problem Solving with Large Language Models*, arXiv:2305.10601, 2023. Backtracking search over intermediate states. Relevant to branch-on-doubt workflows.
- **Shinn et al.**, *Reflexion: Language Agents with Verbal Reinforcement Learning*, arXiv:2303.11366, 2023. Works precisely because a real environment test exists — the same principle enforced by `no-doubt --evidence`.
- **Du et al.**, *Improving Factuality and Reasoning in Language Models through Multiagent Debate*, arXiv:2305.14325, 2023. Independent agents critique each other; uncertain facts get filtered. Inspiration for the spawn-request contract.
- **Gou et al.**, *CRITIC: LLMs Can Self-Correct with Tool-Interactive Critiquing*, arXiv:2305.11738, 2023. Critique with external tools, not prose-only reflection.
- **Snell et al.**, *Scaling LLM Test-Time Compute Optimally*, arXiv:2408.03314, 2024. Best-of-N with a PRM beats longer CoT at equal compute. Guides when to spawn additional verifiers.
- **Madaan et al.**, *Self-Refine: Iterative Refinement with Self-Feedback*, arXiv:2303.17651, 2023. Useful for subjective tasks; less so for factual claims — a helpful boundary case.

### 19.3 Institutional analogs

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

### 19.4 Data-modeling foundations

Why datoms and not triples.

- **Hickey**, *The Datomic Information Model* (datomic.com documentation and accompanying talks). The datom — an immutable atomic fact tagged with a transaction and an assertion/retraction bit. The shape adopted at §4.3.
- **Snodgrass & Jensen**, *A Consensus Glossary of Temporal Database Concepts*, 1994; later codified in SQL:2011 temporal features. Formalises transaction-time vs. valid-time — the model Cozo's `Validity` type implements in a minimal form.
- **CozoDB documentation**, cozodb.org. Embedded Datalog with Validity-typed time travel; the store `dont` is built on.
- **XTDB documentation**, docs.xtdb.com. Bitemporal EDN-document DB; proves the shape scales. Considered and deferred as a deployment target because it is a JVM service rather than an embeddable binary.
- **Codd**, *A Relational Model of Data for Large Shared Data Banks*, *CACM* 13:6, 1970. The relational model that EAV-plus-time generalises; included for grounding, not for day-to-day reference.

### 19.5 Ontology and grounding infrastructure

The semantic-web plumbing `dont` imports from but does not reimplement.

- **Moxon et al.**, *LinkML: an open data modeling framework*, *GigaScience* 2025. One YAML compiles to JSON Schema, SHACL, OWL, Pydantic, ShEx. "Swagger for ontologies" — the candidate schema layer if rule authoring ever gains one.
- **Owen et al.**, *OLS4: a new Ontology Lookup Service for a growing interdisciplinary knowledge ecosystem*, *Nucleic Acids Research* 2024. The primary biomedical ontology registry; `dont import ols` is a thin REST client.
- **Hoyt et al.**, *Unifying the identification of biomedical entities with the Bioregistry*, *Scientific Data* 9, 2022. CURIE and prefix normalization — prevents the most common class of LLM hallucination in ontology workflows.
- **Monarch Initiative**, *Ontology Access Kit (OAK)*, github.com/INCATools/ontology-access-kit. Python library for local OBO traversal; the offline-mode reference.
- **Caufield et al.**, *Structured Prompt Interrogation and Recursive Extraction of Semantics (SPIRES)*, *Bioinformatics* 2024 (preprint at PMC10924283). Zero-shot LLM extraction guided by a LinkML schema plus ontology grounding. A future `dont assume` extension point.
- **Mungall et al.**, *CurateGPT: a flexible language-model assisted biocuration tool*, arXiv:2411.00046, 2024. Multi-agent ontology maintenance patterns.
- **W3C recommendations**: RDF 1.1, SPARQL 1.1, OWL 2, SHACL. The standards stack that `dont` can project into but deliberately does not treat as primary.

### 19.6 Adjacent tools and integration layers

What `dont` borrows from in shape.

- **Steve Yegge**, *Introducing Beads: A coding agent memory system*, steve-yegge.medium.com, October 2025; github.com/steveyegge/beads. The install-once-use-everywhere CLI pattern, the `.beads/` per-project convention, and the agent-memory framing.
- **Model Context Protocol specification**, modelcontextprotocol.io. The north-side surface `dont mcp` implements.
- **WAI**, github.com/charly-vibes/wai. Independent sibling tool; `dont` shares conventions (system-wide install, per-project directory, self-documenting surface, managed blocks) but not code.
- **Claude Code, Cursor, OpenCode** — harness implementations whose subagent-spawn mechanisms `dont` delegates to (§8.11).
- **Dependabot**, **pre-commit**, **Terraform-managed files** — the broader tradition of tools that own a delimited block inside human-edited config files, which `dont sync-docs` follows.

### 19.7 The source synthesis documents

The six documents uploaded at the start of the design conversation — two analyses of why LLMs defend their answers (Claude, ChatGPT), one on the same topic in Spanish (Gemini), and three on ontology tooling for scientific research pipelines (Claude, ChatGPT, Gemini) — contain the broader bibliographies this section distils. They remain the first thing to read for any contributor who wants the fuller picture before diving into individual papers.

### 19.8 Suggested reading order for new contributors

For someone coming into the project cold, a minimum-viable curriculum:

1. One of the synthesis documents in §20.7 — whichever style suits best.
2. Heuer 1999, chapter 8 on ACH — about twenty pages, and the most direct human precedent for `dont`'s methodology rules.
3. Turpin et al. 2023 and Sharma et al. 2024 — thirty minutes to understand why in-context reconsideration fails.
4. Dhuliawala et al. 2023 (CoVe) and Lightman et al. 2023 (PRM) — thirty minutes to understand what replaces it.
5. The CozoDB documentation, specifically the Validity / time-travel sections — the data model in under an hour.
6. Klein 2007 (premortem) and Mellers/Hertwig/Kahneman 2001 (adversarial collaboration) — the social dynamics `dont` tries to automate.

That is roughly half a day of reading. Enough to defend every non-obvious choice in this spec.

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
