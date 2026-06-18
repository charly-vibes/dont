# Implementation status

`dont` has a working Rust implementation. This page summarises what is currently built, what is still in progress, and how the code maps to the OpenSpec specs.

## What is implemented

All core lifecycle commands are functional:

| Command | What it does |
|---|---|
| `dont init` | Initialise the claim store (permissive mode by default; `--strict` for strict mode) |
| `dont prime` | Orient at session start; exits 1 on any doubted claim |
| `dont conclude` | Register an unverified claim |
| `dont ground` | Register and verify a claim in one step |
| `dont flag` | Attach evidence to an existing claim |
| `dont trust` | Mark a claim as doubted (blocks `prime`) |
| `dont ignore` | Set a claim aside (scope change; no longer relevant) |
| `dont lock` | Freeze a mature verified claim (requires ≥ 3 assessed hypotheses + ≥ 2 evidence sources) |
| `dont define` | Record a project vocabulary term |
| `dont hypothesis add/assess` | Track and assess competing explanations under a claim |
| `dont atom define/dismiss` | Decompose a claim into independently checkable sub-conditions |
| `dont show` | Inspect a single claim or term |
| `dont list` | List all claims and their statuses |
| `dont why` | Explain why a claim is in its current status |
| `dont trace` | Walk the blocker path for a claim |
| `dont rules list/explain` | List shipped rules and display their explanations |

The `--json` flag is available on all commands for machine-readable (Envelope-format) output.

## How it maps to specs

The implementation is decomposed across OpenSpec specs in `openspec/specs/`. Key mappings:

| Spec | What it covers |
|---|---|
| `dont-core` | Core claim lifecycle (conclude, flag, trust, lock) |
| `dont-data-model` | Store schema and entity model |
| `dont-envelope` | JSON Envelope and Error protocol |
| `dont-evidence-locators` | Repository-relative file/lines/anchor syntax |
| `dont-status-lifecycle` | Status transition rules and gate conditions |
| `dont-lifecycle-verbs` | Command-by-command verb semantics |
| `dont-cli-surface` | CLI flags, exit codes, output routing |
| `dont-errors` | Structured error types and remediation messages |
| `dont-rule-engine` | Shipped rules, severities, and config |

See [Sources and status](./sources-and-status.md) for the full source map.

## What is not yet built

- `dont import` (LinkML import surface) — spec exists in `openspec/specs/dont-linkml-import`
- MCP interface — spec exists in `openspec/specs/dont-mcp-interface`
- Spawn protocol — spec exists in `openspec/specs/dont-spawn-protocol`
- Completions (shell tab-completion) — wired up but not fully populated
