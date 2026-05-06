# Change: add `dont ground` as a one-shot sidecar workflow

## Why

The current highest-value use of `dont` is recording documented repository facts as explicit, evidenced claims. That workflow currently requires multiple steps (`conclude` then `dismiss`) even when the operator already has both the claim text and the evidence in hand. For sidecar adoption during repository exploration, that ceremony is too high.

## What Changes
- Add a derived command `dont ground` for one-shot claim capture from statement plus evidence.
- Define the initial `ground` input scope explicitly as statement text plus one or more evidence references, with normal author override support but without introducing new atoms/dependency/refs convenience syntax in the first version.
- Define `ground` as an orchestration command that emits the normal conclude/verify event sequence rather than bypassing core invariants.
- Define single-invocation atomic failure semantics so an unsuccessful grounding attempt does not leave confusing partial side effects by default.
- Add tutorial/how-to coverage positioning `ground` as the fast path for repo-fact capture while keeping the core four verbs normative.

## Deferred
- Grounding terms directly with a one-shot verb
- Multi-claim batch extraction from a document
- Automatic claim synthesis from highlighted text
- Interactive TUI prompting

## Change Type
- New capability and usability strengthening, not merely an implementation repair

## Related Changes
- Prefers the structured repository evidence introduced by `add-evidence-locators`

## Impact
- Affected specs: `dont-ground-command`, `dont-cli-surface`, `dont-payload-types`, `dont-agent-help`
- Affected code: new CLI verb, orchestration layer, input validation, tutorial/help output
- Affected workflow: lowers friction for the strongest current sidecar use case
