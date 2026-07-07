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

## Analytics and eval

```
dont stats --json                                  # verb mix, dedup hits, idle-skill flag
dont stats --since 2026-06-01T00:00:00Z --json    # scoped to a time window
dont export --eval --json                          # structured JSON for eval harnesses
dont export --eval --session <id> --json           # scoped to a session
```

`dont stats` reports per-scope: verb counts, dedup-hit count, idle-skill flag (no claims
concluded), caught-contradiction count, and claim-verification rate.

`dont export --eval` produces an `EvalExport` payload with claim counts by verb, trust events
with targets, dedup refusals, and wall-clock timestamps — suitable for A/B harnesses.

## Ephemeral mode

Pass `--no-persist` (or set `DONT_NO_PERSIST=1`) to run any command in memory only.
All commands are validated and checked but no events are written to the store.
Use this as the C2 "fake-dont" control arm in eval experiments.

```
dont --no-persist conclude "claim text"   # validate without persisting
DONT_NO_PERSIST=1 dont list               # read-only pass-through
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

## Gate Integration

`dont` survives in a project only when wired into a failing gate. Without one,
usage decays to zero — nothing breaks when claims go ungrounded.

### Pre-push hook (lefthook)

Add this to `lefthook.yml` to reject pushes with ungrounded claims:

```yaml
pre-push:
  parallel: true
  commands:
    check-claims:
      run: dont check
      skip: merge
```

### Pre-push hook (shell script)

For projects without lefthook, a standalone script in `.githooks/pre-push` or
`scripts/gate.sh`:

```bash
#!/bin/sh
# Pre-push gate: reject if any claims are ungrounded
dont check || {
  echo "✗ Blocked: ungrounded claims exist."
  echo "Ground them with: dont flag <id> --evidence <url>"
  dont list --status unverified
  exit 1
}
```

### CI pipeline

```yaml
- name: Check grounded claims
  run: dont check
```

### Why this works

`dont check` exits 0 when all claims are verified, locked, or ignored, and exits
1 when any claim is unverified. No `jq`, `python`, or fragile `grep` required.

> **Note**: This gate is most effective once the `--url` permalink locator is
> available (Option C, shipped in `dont` v0.1.0+). External evidence (sibling
> repos, vendored paths) can be referenced without filesystem access.
