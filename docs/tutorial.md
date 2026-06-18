# Getting Started with `dont`

## Prerequisites

Build `dont` from source and install it to your PATH:

```bash
git clone https://github.com/charly-vibes/dont.git
cd dont
cargo install --path .
```

Verify:

```bash
dont --version
```

## When to reach for `dont`

The trigger is a thought that starts with *"I think..."*, *"I believe..."*, or *"I'm assuming..."* — combined with the fact that you haven't verified it yet, and that being wrong would cost you time.

If you find yourself holding two competing explanations for a bug, that's the clearest signal. `dont` gives those competing explanations a name, lets you track evidence for each, and forces you to resolve them before moving on.

## A real session

You're debugging a write-loss bug. Writes succeed (no errors returned) but data disappears after process restart. You have two theories.

**First time? Initialize first:**

```bash
dont init
```

**Returning session? Orient first:**

```bash
dont prime
```

This shows what claims are currently doubted or unverified — your open questions from the last session. If the store is clean, it exits 0 silently. If something is doubted, it exits 1 and tells you what to resolve first.

**Step 1: Record the claim you're investigating**

```bash
dont conclude "CozoDb flushes writes lazily, causing data loss on unclean shutdown"
# → claim:01ABC123...
```

**Step 2: Add a competing hypothesis**

You're not sure. Connection pool exhaustion could also cause silent write drops before any flush happens.

```bash
dont hypothesis add claim:01ABC123... --text "Connection pool exhaustion silently drops writes before flush"
# hypothesis added at index 0
```

**Step 3: Gather evidence**

You find the flush behavior in source:

```bash
dont flag claim:01ABC123... --file src/store.rs --lines 47-62
```

You check connection pool behavior and find it refutes the competing hypothesis. Hypotheses are referenced by 0-based index:

```bash
dont hypothesis assess claim:01ABC123... 0 --refuting src/pool.rs
```

**Step 4: Confirm**

Flagging evidence (`dont flag`) transitions the claim from `unverified` to `verified`. `dont show` lets you confirm the new state:

```bash
dont show claim:01ABC123...
```

Status is `verified`. The claim is grounded and the competing hypothesis is marked refuted.

**Step 5: Lock when mature**

`dont lock` requires ≥ 3 assessed hypotheses and ≥ 2 independent evidence sources. This tutorial used one hypothesis for brevity — see [Competing hypotheses and atoms](./hypotheses.md) for the full lock pattern.

```bash
dont lock claim:01ABC123...
```

## Lifecycle summary

| Step | Command | Status after |
|---|---|---|
| State the claim | `dont conclude "..."` | `unverified` |
| Add a competing explanation | `dont hypothesis add <id> --text "..."` | — |
| Attach evidence | `dont flag <id> --file <path> --lines N-M` | `verified` |
| Assess a hypothesis | `dont hypothesis assess <id> <idx> --refuting <path>` | — |
| Check status | `dont show <id>` | (read-only) |
| Freeze a mature claim | `dont lock <id>` | `locked` |
| Register doubt | `dont trust <id> --reason "..."` | `doubted` |
| Orient at session start | `dont prime` | exits 0 or 1 |

## Fast path: one-shot grounding

When you already have both the claim and the evidence in hand:

```bash
dont ground "The crate exposes a test recipe" --file justfile --lines 12-13
```

This concludes and verifies in a single command.
