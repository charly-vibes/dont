# Bounded context: claim lifecycle

This context covers the full lifecycle of a claim — from introduction through verification, doubt, and lock.

## Entities

### Claim

An assertion about the codebase or project. Has a unique ID (UUID-based, displayed as `claim:01...`), text, and a status.

Created by `dont conclude "<text>"` or `dont ground "<text>" --file <path>`.

### Evidence

A pointer attached to a claim via `dont flag`. Evidence has two forms:
- **Repository-relative locator** — `--file <path> --lines N-M` or `--file <path> --anchor <heading>`. Preferred.
- **External URI** — `--evidence https://...`. For evidence outside the repo.

### Hypothesis

A competing explanation recorded under a claim via `dont hypothesis add`. Referenced by 0-based index. Assessed as `--supporting` or `--refuting` via `dont hypothesis assess`.

### Atom

A sub-condition of a composite claim. Defined via `dont atom define`, dismissed via `dont atom dismiss`. All atoms must be dismissed before `dont lock` succeeds.

## Status machine

```
             conclude
               │
           unverified ◄──── trust (via dont trust)
               │                        │
            flag                    doubted
          (evidence)                    │
               │                    undoubt (via dont trust --clear)
           verified                     │
               │                        │
           lock ──────────────────► locked
```

### Transition rules

| From | To | Command | Gate |
|---|---|---|---|
| — | `unverified` | `dont conclude` | none |
| `unverified` | `verified` | `dont flag` | dependency gate: all dependencies must be verified |
| `verified` | `locked` | `dont lock` | ≥ 3 assessed hypotheses + ≥ 2 independent evidence sources |
| any | `doubted` | `dont trust` | none |
| `doubted` | `unverified` | `dont trust --clear` | none |
| `doubted` → `locked` | — | — | **forbidden** — no code path exists |

## Invariants

- `doubted` claims block `dont prime` regardless of project mode.
- In permissive mode, `unverified` claims produce warnings but do not block CI.
- In strict mode, `unverified` claims are errors and block `dont flag`.
- The dependency gate runs before any verification is accepted: if a claim depends on an unverified or doubted claim, `dont flag` is refused.
- Hedge language (`"I think"`, `"maybe"`, `"probably"`) in `dont trust` or `dont ignore` text is rejected by the hedge filter.
