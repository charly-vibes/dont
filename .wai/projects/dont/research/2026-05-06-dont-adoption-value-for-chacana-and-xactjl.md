# Research: `dont` adoption value for Chacana and XAct.jl

## Evaluation summary

`dont` already shows real value as an **epistemic sidecar** for AI-heavy scientific repositories, but it is not yet strong enough to serve as the **primary workflow spine** for Chacana or XAct.jl.

The strongest current value is:

- forcing explicit claims instead of chat-only assertions
- attaching evidence to those claims
- preserving an auditable claim/evidence history
- making agents distinguish verified facts from loose inferences

The main current limitation is that the tool is much better at **standalone verified claims** than at **dense term/dependency-centered knowledge building**, which is the mode these two repositories most want.

## Context and method

This evaluation used a kicking-tires round against two external repositories:

- `../../sk/chacana/`
- `../../sk/XAct.jl/`

To avoid mutating either repository, `dont` was run with per-repo temporary stores:

- `/tmp/dont-kicking-tires/chacana.dont`
- `/tmp/dont-kicking-tires/xactjl.dont`

Observed commands:

- `dont init --json`
- `dont prime --json`
- `dont define ...`
- `dont conclude ...`
- `dont dismiss ...`
- `dont list --json`
- `dont show ... --json`

## What worked well

### 1. Temporary sidecar operation worked cleanly

Using `DONT_DIR=/tmp/...` made it possible to evaluate `dont` against a real repository without creating `.dont/` state inside the target repo. That lowers adoption risk substantially for first trials.

### 2. Session-orientation value is already visible

`dont prime --json` gives a useful session-start summary. Even in its current minimal state, it already nudges the operator toward an explicit epistemic workflow instead of ad-hoc assertions.

### 3. Standalone documented facts work well

The strongest successful pattern was:

1. conclude a claim
2. dismiss it with repository evidence
3. inspect the verified result

Examples that worked cleanly:

- Chacana: `"Chacana parses tensor expressions into a MathJSON-style AST."`
- XAct.jl: `"XAct.jl provides a Julia tensor algebra engine and a Python wrapper."`

This is already useful for:

- onboarding facts
- architecture facts
- README-backed truths
- project invariants an agent should stop bluffing about

### 4. Strict/permissive mode is legible

Running XAct.jl with strict init produced an understandable orientation result. That suggests the tool can eventually support different epistemic operating postures across projects.

## Value added by `dont`

## 1. It turns agent assertions into first-class objects

Without `dont`, claims usually live in chat, commit messages, issue comments, or transient notes. With `dont`, a claim can become an explicit entity with:

- statement
- status
- evidence
- history
- dependencies

For scientific and specification-heavy projects, this is a meaningful improvement because it separates:

- documented fact
- design intent
- implementation inference
- unresolved assumption

## 2. It reduces AI overclaiming

Both Chacana and XAct.jl are domains where plausible-sounding nonsense is easy to generate. `dont` adds value by requiring an agent to say:

- what exactly is being claimed
- what evidence supports it
- whether it is verified
- what other concepts it depends on

That is especially valuable for mathematical, parser, and architecture claims.

## 3. It creates a reusable epistemic memory

A verified claim ledger is more precise than an issue tracker, design note, or handoff because it records not just that something was discussed, but whether it was actually grounded.

This is useful for:

- future agent sessions
- code review
- architecture review
- onboarding
- recovery after context loss

## 4. It is a good complement, not a replacement

`dont` does not replace:

- `bd` for task tracking
- `wai` for design/research memory
- `openspec` for normative change proposals
- tests for executable behavioural proof

Its distinctive value is epistemic hygiene: **what is claimed, what is evidenced, and what remains unsafe to trust**.

## Why these repositories are good candidates

## Chacana

Good fit because the repository has:

- explicit formal concepts
- a parser/checker architecture
- documentation-backed invariants
- a high risk of agent bluffing about grammar or type-checking behaviour

High-value claim examples:

- parser output shape
- checker invariants
- context model boundaries
- editor tooling boundaries

## XAct.jl

Good fit because the repository has:

- a mathematically sensitive domain
- a complex architecture boundary between Julia engine, Python wrapper, and verification tooling
- a README full of nuanced truths worth grounding
- a high cost for false statements about parity, typing, or canonicalization

High-value claim examples:

- typed API guarantees
- engine limitations
- wrapper/runtime boundary
- verification-scope claims

## Current limitations

## 1. The best current path is narrow

The path that works reliably is:

1. conclude a standalone claim
2. attach evidence
3. verify it

That is real value, but it is narrower than the long-term promise.

## 2. Term/dependency-centered knowledge work is still weak

The natural workflow for these repositories is often:

1. define a term
2. conclude claims depending on the term
3. verify the term
4. verify the dependent claims
5. inspect the dependency graph

In the kicking-tires round, this path was not smooth. A claim depending on a coined term became blocked in a way that makes current adoption feel brittle.

Important note: this looks primarily like an **implementation gap against the existing specs**, not necessarily a missing specification. The current specs already distinguish unresolved references from merely unverified ones, and they already allow `dismiss` on terms.

## 3. Evidence is too weakly shaped for repository-grounding workflows

Using bare URIs such as `file:///repo/README.md` is enough for a tracer bullet, but not enough for a high-trust workflow. For repository-grounded use, the tool needs stronger evidence structure, such as:

- file path relative to repo
- line spans or anchors
- captured excerpt/quote
- optional fingerprint to detect drift

Without that, the tool records that evidence exists, but not precisely *what* in the repository justified the claim.

## 4. The current flow has too much ceremony for sidecar use

For a simple documented fact, the operator currently needs at least:

1. conclude
2. dismiss
3. possibly inspect

That is acceptable for deliberate work, but too heavy for the common sidecar use case:

> “I found a documented fact in this repo; record it precisely and move on.”

## 5. Dependency diagnostics are not yet actionable enough

When a claim is blocked by dependency fallout, the operator needs a short causal explanation, not just a label such as `stale`. The value of graph-aware epistemic tooling depends on making blockers inspectable and fixable.

## Is `dont` worth using now?

### Yes, for limited adoption

Recommended current use:

- architecture facts
- onboarding facts
- README/spec-backed claims
- scientifically important assertions that should not be left in chat only

### No, not yet as the main workflow spine

Not recommended yet:

- forcing every task through `dont`
- heavy ontology-building
- dense dependency-graph workflows
- using it as a substitute for tests, issues, or specs

## Recommendation

Adopt `dont` first as a **high-value fact ledger** for AI-assisted sessions in Chacana and XAct.jl. Strengthen the product in three directions before broader adoption:

1. **stronger evidence structure for repository grounding**
2. **lower-friction one-shot grounding flow for documented facts**
3. **actionable dependency/blocker tracing**

## Proposed specification work

This research recommends three OpenSpec changes:

1. `add-evidence-locators`
   - add structured repository evidence locators and captured excerpts
2. `add-ground-command`
   - add a one-shot command for conclude+verify sidecar workflows
3. `add-trace-query`
   - add an explicit dependency/blocker tracing query for actionable diagnosis

## Non-proposal implementation note

The current implementation should also be checked against existing specs for coined-term dependency handling. Based on the current specifications, a claim that depends on a coined term should not become blocked merely because that term is still `unverified`; that is separate from an unresolved-term condition.

## Bottom line

`dont` already adds real value where the cost of agent bluffing is high. Chacana and XAct.jl are both in that category. The tool is worth using experimentally today as a sidecar for high-value grounded facts. It is not yet worth making central to the full workflow until repository evidence, one-shot grounding, and blocker tracing are stronger.
