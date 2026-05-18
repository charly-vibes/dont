# dont

This project uses `dont` for epistemic claim tracking.

For full documentation see the [dont spec](https://github.com/charly-vibes/dont).

## Reading the CLI

Every command is read as a full phrase: `dont <verb>` = "do not `<verb>`".

| Command | Phrase | Meaning |
|---------|--------|---------|
| `dont trust <id>` | "do not trust it" | Register doubt — you lack confidence in this claim |
| `dont flag <id>` | "do not flag it as a concern" | Verify with evidence — it's been checked and cleared |
| `dont undoubt <id>` | retract doubt | Walk back a `trust` — returns entity to `unverified` |
| `dont ignore <id>` | "do not engage with it" | Set aside — out of scope |
| `dont reopen <id>` | re-engage | Restore an `ignored` entity to `unverified` |

**Key distinction**: `trust` and `flag` are opposites. `dont trust` means you distrust the claim; `dont flag` means you're clearing it (not flagging it as problematic). `undoubt` is the correction verb for `trust` — use it when doubt was registered in error.

## Quick start

```
dont conclude "claim text"              # introduce a claim
dont trust <id> --reason "..."          # register doubt (do not trust it)
dont flag <id> --evidence <uri>         # verify with evidence (do not flag it)
dont undoubt <id>                       # retract doubt (return to unverified)
dont show <id>                          # inspect a claim
dont list                               # list all claims
```

## Rule Claim Authoring

When documenting a `dont` rule's behavior as a claim, use the canonical slot-marker
template below. This schema ensures claims are complete and machine-checkable by Phase 2's
`rule-claim-structure` lint rule.

### Canonical Template

```
[INVOCATION] <rule-name> runs as: background lint | opt-in via `dont check --<flag>`
[CONFIG]     Enabled by default: yes | no
[MODE]       In permissive mode: warn | strict | same as strict | n/a
[TRIGGER]    Fires when: <condition>
[GUARD]      Silently skips: <inputs>  (omit if no guard)
[EVAL]       Evaluation model: stateless demand | event-driven on <event>  (omit if stateless demand)
[BOUNDARY]   Does not handle: <edge cases>; defers to <other-rule>  (omit if no boundary)
```

### Slot Reference

| Slot | Symbol | Mandatory | Default when omitted |
|------|--------|-----------|----------------------|
| INVOCATION MODEL | `[INVOCATION]` | No | background lint, runs with `dont prime` |
| TRIGGER CONDITION | `[TRIGGER]` | **Yes** | — |
| PRECONDITION GUARD | `[GUARD]` | No | evaluates all inputs; no silent skip |
| EVALUATION MODEL | `[EVAL]` | No | stateless demand-evaluated |
| CONFIG (enablement) | `[CONFIG]` | **Yes — one of CONFIG or MODE** | — |
| MODE (severity behavior) | `[MODE]` | **Yes — one of CONFIG or MODE** | — |
| BOUNDARY | `[BOUNDARY]` | No | no explicit boundary with sibling rules |

### CONFIG vs MODE

`[CONFIG]` and `[MODE]` are distinct sub-markers that together satisfy the mandatory
CONFIG/MODE requirement. Either alone is sufficient; both may appear in the same claim.

- **`[CONFIG]`** — covers *enablement*: is the rule on by default? Write this when the
  rule is off-by-default (e.g. `term-nonfunctional-label`) or context-dependent.
- **`[MODE]`** — covers *severity behavior*: does severity differ across permissive vs
  strict mode, or is it always the same? Write this when severity differs or when the
  rule is unconditionally warn-only.

Their combined absence was the proximate cause in all 3 retracted rule claims.

### Before You Create

Fill and review the complete template before running `dont conclude`. There is no
pre-creation dry-run — changing a claim requires doubting the old version and creating
a corrected one.

### When Optional Slots Become Load-Bearing

- **`[INVOCATION]`**: Required when the rule is opt-in (e.g. `lockable`). The background-lint default is wrong for opt-in rules.
- **`[GUARD]`**: Required when the rule silently skips a non-obvious subset of inputs.
- **`[EVAL]`**: Required when evaluation is event-driven or stateful. Stateless demand is correct for 6 of 7 current rules.
- **`[BOUNDARY]`**: Required when the rule's scope is defined by exclusion from a sibling rule. The `ungrounded`↔`dangling-definition` boundary is the clearest example.

### Tagging Rule Claims

Tag every rule-describing claim with the `rule-claim-type` term using its UUID in `--depends-on`:

```bash
dont conclude "..." --depends-on term:01KR4TNRGHVPRZQ1Z95GFZN4ZQ
```

Do **not** use the CURIE `local:rule-claim-type` directly — that triggers `unresolved-terms`.

> **Warning**: `rule-claim-type` is a stable anchor term. Doubting it triggers
> `stale-cascade` warnings for all tagged rule claims simultaneously.
