# Using `dont` as an LLM agent

## The core pattern

Every time you are about to write something as established fact — in a reply, in a command, in a plan — and you haven't verified it yet, use `dont conclude` first.

Then verify it. Then `dont flag` with the evidence.

This is the only discipline that matters. Everything else follows.

## Trigger conditions

Use `dont conclude` when you find yourself producing text like:

- "The `store.rs` file handles connection pooling"
- "This function is called only from the test harness"
- "The API rate limit is 500 requests per minute"
- "I believe the bug is in the flush path, not the connection path"

Each of these is an unverified claim. Record it before you act on it.

## Do not use `dont` for

- Things you already verified before stating them (just state them)
- Universal facts not specific to this codebase ("Rust is memory-safe")
- Housekeeping claims that will never be doubted ("the commit message is in conventional commits format")

The overhead is worth it when being wrong would send you down a wrong path. Apply it selectively.

## Session start

Always run `dont prime` before beginning investigation work:

```bash
dont prime
```

- If it exits 0: no open doubted claims. Proceed.
- If it exits 1: there are doubted claims from a previous session. Resolve them before introducing new claims.

This is your orientation step. It is not just a CI gate.

## Full pattern for investigating a bug

```bash
# 1. Orient
dont prime

# 2. State what you believe (before you verify)
dont conclude "The write loss is caused by lazy flush in CozoDb"
# → claim-abc

# 3. Add competing explanations (before you go looking for evidence)
dont hypothesis add claim-abc --text "Connection pool exhaustion drops writes silently"
dont hypothesis add claim-abc --text "OS buffer not flushed on SIGTERM"

# 4. Find evidence — file + line span
dont flag claim-abc --file src/store.rs --lines 47-62

# 5. Assess hypotheses as evidence comes in (IDX is 0-based integer)
dont hypothesis assess claim-abc 0 --refuting src/pool.rs:88-103
dont hypothesis assess claim-abc 1 --supporting src/shutdown.rs:12-18

# 6. Check status
dont show claim-abc
```

## What counts as evidence

Prefer repository-relative locators:

```bash
--file src/store.rs --lines 47-62       # file + line span
--file justfile --lines 12-13           # any text file in the repo
--file docs/spec.md --anchor "Flush"    # anchor within a file
```

External URLs are accepted when the evidence lives outside the repo:

```bash
--evidence https://docs.example.com/api#rate-limits
```

## Quick reference

```bash
dont prime                              # orient: any doubted claims?
dont conclude "..."                     # register an unverified claim
dont ground "..." --file F --lines N-M # register and verify in one step
dont flag <id> --file F --lines N-M    # attach evidence to existing claim
dont hypothesis add <id> --text "..."  # add a competing explanation
dont hypothesis assess <id> <idx> --refuting <uri>  # idx is 0-based integer
dont show <id>                         # check current status
dont trust <id> --reason "..."         # mark as doubted (blocks prime)
dont ignore <id> --reason "..."        # set aside a claim (scope change, no longer relevant)
dont lock <id>                         # freeze a mature verified claim
dont list                              # all claims and their statuses
dont trace <id>                        # explain why a claim is blocked
```
