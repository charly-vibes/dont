<!-- BEADS:START -->
<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY.
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session
<!-- END BEADS INTEGRATION -->
<!-- BEADS:END -->

<!-- DONT:START -->
<!-- dont: managed block -->
See .dont/ for grounded-claim workflow.
<!-- DONT:END -->

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

<!-- WAI:START --># Workflow Tools

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
3. `bd ready` — find available work items

When context reaches ~40%: stop and tell the user — responses degrade past
this point. Recommend `wai close` then `/clear` to resume cleanly.
Do NOT skip `wai close` — it enables resume detection.

## Available Pipelines

| Pipeline | When to Use | Start |
|----------|-------------|-------|
| epic-tdd-ro5 | Working through an epic where each ticket requires full TDD discipline and quality review | `wai pipeline start epic-tdd-ro5 --topic=<topic>` |
| tdd-ro5 | Feature development or bug fixes requiring test-driven discipline and quality review | `wai pipeline start tdd-ro5 --topic=<topic>` |

> Pipeline steps may have gates that enforce artifact creation, review coverage, and oracle checks before advancement. Run `wai pipeline gates <name>` for details.
> **Before any edit**: run `wai pipeline current --json`; if a run is active for this project, resume it (`wai pipeline next`) rather than starting ad-hoc work.

## Ubiquitous Language

If `.wai/resources/ubiquitous-language/README.md` exists, read it first as the
navigation index, then open only the bounded-context files relevant to the task.
Avoid loading every terminology file unless the work truly spans multiple contexts.

## Autonomous Work Policy

Proceed without routine confirmation when the next step is clear.
Do not ask to continue, fix, or commit — just do it.

**Stop and ask** only when:
- Conflicting requirements or ambiguous intent
- Destructive actions (data loss, force-push, drop table)
- Credentials, secrets, or external services not yet authorized
- Unresolved test failures after two attempts
- Push, deploy, or release — always get explicit authorization
- Context approaching 40% — recommend `wai close` then `/clear`

## Detailed Instructions

Full workflow reference — session lifecycle, capturing work, command cheat
sheets, cross-tool sync, and PARA structure — lives in **`.wai/AGENTS.md`**.
Read it at the start of your first session or when you need detailed guidance.

Keep this managed block so `wai init` can refresh the instructions.


<!-- provenance: generator=wai version=0.12.3 source=WAI sha=77c71824 -->
<!-- WAI:END -->

## Behavioral Constraints

These constraints are **persistent** — they live outside the WAI managed
block so they survive `wai init`. Do not remove or edit them without
deliberate intent.

### Prohibited (DON'T)

- **DON'T** make breaking changes to the claim model or envelope format without an openspec proposal
- **DON'T** push directly to main — all changes go through feature branches with PR review
- **DON'T** introduce claims without grounding them through `dont` itself (dogfood — dont must eat its own food)
- **DON'T** modify managed blocks (`<!-- WAI: -->`, `<!-- OPENSPEC: -->`, `<!-- DONT: -->`)
- **DON'T** skip `just check-claims` — doubted claims block CI and must be resolved
- **DON'T** commit the SQLite/cozo database files — `.dont/` is gitignored; use `dont export` for cross-machine sync
- **DON'T** introduce new parallel patterns for functionality that genesis already provides — file a genesis change first. (dont is a genesis donor: prefer adding shared patterns to genesis before implementing them locally.)

### Stop and Ask

Pause and request human input when any of these triggers fire:
1. **Ambiguity** — the ticket text itself is contradictory or underspecified
2. **Scope uncertainty** — the ticket is clear but the change naturally touches code or features not mentioned in it
3. **Irreversibility** — data loss, schema migration on the cozo DB, destructive CLI changes
4. **Secrets/credentials** — any external service, API key, or credential not yet authorized
5. **Test failure persistence** — unresolved test failure after two repair attempts, or the same failure across 3 different approaches
6. **Push/release** — pushing to remote, creating a release, or deploying
7. **Context saturation** — context approaching ~40%; recommend `wai close` then `/clear`

### Minimal Footprint

- Prefer small, focused changes over large refactors — one ticket, one concern
- Delete unused code, don't leave commented-out code behind
- Keep PRs under 400 lines changed. If you cannot, split the work into multiple PRs before proceeding.
- Use existing abstractions (genesis, wai patterns) before introducing new ones. dont is a genesis donor — prefer adding shared patterns to genesis before implementing them locally.
- dont is a CLI tool — prefer file-based persistence over in-memory state that disappears

### Drift Detection

Proceed without routine confirmation when the next step is clear.
Do not ask to continue, fix, or commit — just do it. After each major
action (edit, test run, commit), pause and self-check:
1. **ALIGNMENT** — does this still serve dont's purpose of catching ungrounded assertions?
2. **SCOPE** — did I stay within the ticket scope or did I expand into unticketed work?
3. **FOOTPRINT** — did I leave dead code, debug prints, or unnecessary changes?
4. **GOVERNANCE** — did I follow openspec workflow for spec changes?

If any check fails: undo the last change (`git checkout -- <files>` for
uncommitted edits, `git revert HEAD` for committed) before proceeding,
or open a follow-up ticket.

<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.


<!-- provenance: generator=wai version=0.12.3 source=WAI:REFLECT:REF sha=73879972 -->
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

<!-- ah:managed:start -->
## espectacular

Run `ah check` to verify spec-test correspondence before committing.

- `ah check` — validate all deployed specs
- `ah check --changes <name>` — validate with a change overlay
- `ah init` — set up or refresh espectacular project files
- `ah doctor` — diagnose setup issues
- `ah explain <topic>` — playbook guidance for finding kinds and suggested actions
- `ah doctor --enable <adapter>` — write adapter config into .espectacular/config.toml
- `ah signals` — emit dont drift signals
<!-- ah:managed:end -->
<!-- DDL:START -->

## dulce-de-leche (ddl v0.8.0)

Orchestrator for the charly-vibes tool ecosystem. Run `ddl init` to
install and configure all tools. Run `ddl status` for a health overview.

Commands:
- init     Bootstrap the toolset
- install  Install a single tool
- status   Show health overview
- doctor   Run diagnostics
- version  Show versions
- upgrade  Update tools
- migrate  Migrate configs
- scope    Show active .ddl/

<!-- DDL:END -->

## Test runner (DDL standard — recorded in DDL-85p, dulce-de-leche repo)

- **TDD loop**: agents run `just test` (testaruda exec loop, working diff)
  instead of `cargo test` directly — it selects the affected test set,
  runs only those tests, and ingests results for future selection.
  Tolerance contract: exit 0/10/20 green; NOT CALIBRATED with 0 actual
  failures is a vacuous pass (tambor-cpd policy), with failures it
  blocks. `just test-blast` previews the affected set without running
  anything.
- **Full suite**: `just test-full` is the plain cargo test run. The push
  gate remains lefthook pre-push (testaruda select/exec + the other
  gates); selection is for fast feedback, never a gate substitute.
