<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

<!-- WAI:START -->
# Workflow Tools

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

Detected workflow tools:
- **wai** — research, reasoning, and design decisions
- **beads** — issue tracking (tasks, bugs, dependencies). CLI command: **`bd`** (not `beads`)
- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)

> **CRITICAL**: Apply TDD and Tidy First throughout — not just when writing code:
> - **Planning/task creation**: each ticket should map to a red→green→refactor cycle; refactoring tasks must be separate tickets from feature tasks.
> - **Design**: define the test shape (inputs/outputs) before designing the implementation.
> - **Implementation**: write the failing test first, then make it pass, then tidy in a separate commit.

> **When beginning research or creating a ticket**: run `wai search "<topic>"` to check for existing patterns before writing new content.
> **Ro5**: The Rule of 5 skill is installed. Run `/ro5` after key phase transitions — implement, research, design — for iterative quality review.

## Quick Start

1. `wai sync` — ensure agent tools are projected
2. `wai status` — see active projects, phase, and suggestions
3. `dont prime` — see current claim status (open questions, unverified claims)
4. `bd ready` — find available work items

When context reaches ~40%: stop and tell the user — responses degrade past
this point. Recommend `wai close` then `/clear` to resume cleanly.
Do NOT skip `wai close` — it enables resume detection.

## Available Pipelines

| Pipeline | When to Use | Start |
|----------|-------------|-------|
| tdd-ro5 | Feature development or bug fixes requiring test-driven discipline and quality review | `wai pipeline start tdd-ro5 --topic=<topic>` |

> Pipeline steps may have gates that enforce artifact creation, review coverage, and oracle checks before advancement. Run `wai pipeline gates <name>` for details.

## Detailed Instructions

Full workflow reference — session lifecycle, capturing work, command cheat
sheets, cross-tool sync, and PARA structure — lives in **`.wai/AGENTS.md`**.
Read it at the start of your first session or when you need detailed guidance.

Keep this managed block so `wai init` can refresh the instructions.

<!-- WAI:END -->

<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.
<!-- WAI:REFLECT:REF:END -->





## Claim Discipline

`dont` exists to prevent ungrounded assertions. When building `dont`, agents must use `dont` to track design claims before asserting them — not after. A claim made without grounding is the exact failure mode the tool is designed to catch.

**When making a design or correctness assertion during development:**
1. `dont conclude "<assertion>"` — register the claim
2. Define any CURIEs used in the claim with `dont define`
3. Add evidence: `dont flag <id> --file="<path>" --anchor="<section>"` or atomically: `dont ground "<assertion>" --file="..." --anchor="..."`
4. Claims in `doubted` status block CI — `just check-claims` fails

**Prefer `dont ground` as the fast path** when you already have the claim text and evidence in hand — it atomically composes `conclude` + `dismiss` in one command.

**Before concluding, invoke `dont-grill`** (the structured interview skill pack) to verify the claim is ready. The skill checks for near-duplicates, claim-worthiness, and proper evidence before committing.

In permissive mode, `unverified` claims are allowed in CI. Only `doubted` claims are blocking. In strict mode, the tool enforces grounding at `conclude` time.

**Mode × gate interaction:** `just check-claims` runs `dont prime`, which exits 1 on any doubted claim. The recipe is the pre-push check; the tool's own strict-mode enforcement is orthogonal.

**Claim retirement:** when a design decision is reversed, run `dont trust <id> --reason="superseded by X"` then `dont ignore <id> --reason="superseded"`. Stale verified claims erode trust in `dont prime` output.

**Success metric (30-day retrospective):** run `dont list --status=verified`. If fewer than 60% of architectural claims are verified after 30 days, simplify or drop the discipline. A claim with no evidence activity for 30+ days is a retirement candidate.

## Beads Issue Tracker

This project uses **bd** for issue tracking.

Start with:
- `bd prime` — workflow context
- `bd ready` — unblocked work
- `bd create "Title" --type task --priority 2` — create an issue
- `bd close <id>` — complete work

## Git & Workflow Discipline

- **Never use `git add -A`** — always stage specific files with explicit paths
- **Per-ticket pipeline**: always follow `TDD → ro5u → fix → commit → next ticket`
