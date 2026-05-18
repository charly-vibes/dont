# Change: add eval readiness features

## Why

The evaluation research (`dont-evaluation.md`) identifies four concrete infrastructure gaps that block
any defensible A/B validation of `dont`'s epistemic value:

1. **No adoption telemetry.** The research names "idle-skill rate, verb mix, and dedup-hit rate" as
   the primary adoption metric family. Without a `dont stats` command, neither users nor evaluators
   can observe whether the agent is using the tool at the right frequency and in the right proportion
   of verbs.

2. **No verbosity-control baseline (C2 arm).** The research calls a fake-`dont` that accepts all
   commands but persists nothing "non-negotiable" for separating epistemic-state value from
   thinking-room value. Without an ephemeral mode, any positive A/B result is confounded by chain-of-
   thought induction rather than claim-management semantics.

3. **No caught-contradiction metric.** The research's central mechanistic claim is that `dont` catches
   contradictions before they cascade into wasted edits ("23.4% cascading-failed-edits" in SWE-agent
   data). This is computable from the existing claim graph but currently unexposed.

4. **No eval-export format.** Running the MVP experiment (four conditions × 100 tasks × k=3 reruns)
   requires a structured per-session dump of claim counts, verb events, dedup refusals, and wall-clock
   timestamps. Without this, each eval harness must reverse-engineer the store schema independently.

## What Changes

- New capability: `dont-analytics` — introduces the `dont stats` command, which reports per-session
  verb mix, dedup-hit count, idle-skill flag, claim-verification rate, and caught-contradiction count.
- New capability: `dont-ephemeral-mode` — introduces the `--no-persist` global flag, which runs any
  `dont` invocation in memory only; all commands are accepted and validated but no events are written
  to the store. Acts as the C2 "fake-dont" control arm.
- New capability: `dont-eval-export` — introduces `dont export --eval`, which produces a structured
  JSON document suitable for eval harnesses, covering claim counts by verb, trust events with targets,
  dedup refusals, and wall-clock timestamps across a session or date range.

## Deferred

- Integration with an external benchmark runner or harness (SWE-agent, OpenHands) — that is external
  eval infrastructure, not a `dont` feature.
- A/B harness code, failure-mode classifier, or replay-study tooling — also external.
- Per-verb ablation mode (disabling individual verbs) — separate proposal if needed.
- Real-time streaming telemetry or Prometheus/OpenTelemetry export — out of scope for the MVP eval.

## Traceability

- Adoption metric family → §3, §5, §8 of `dont-evaluation.md` ("adoption telemetry", "idle-skill
  rate", "verb mix", "dedup-hit rate").
- C2 verbosity-control arm → §2, §6, §8 of `dont-evaluation.md` ("fake-dont", "non-negotiable",
  "verbosity confounder").
- Caught-contradiction metric → §2, §3 of `dont-evaluation.md` ("caught-contradiction rate",
  "doubted-claim precision").
- Eval-export format → §8 of `dont-evaluation.md` ("per-task metrics", "aggregate metrics").

## Impact

- Affected specs: `dont-analytics` (new), `dont-ephemeral-mode` (new), `dont-eval-export` (new)
- Affected docs: `.dont/AGENTS.md` managed block will need to reference `dont stats` and `dont export`
- Affected workflow: none — all features are additive read-only commands or a pass-through flag
