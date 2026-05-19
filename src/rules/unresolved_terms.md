# unresolved-terms

**Severity:** strict (non-overridable)

## What the rule checks

`unresolved-terms` fires when a verified claim depends on a term that is not itself in `verified` or `locked` status. A claim is only as reliable as its foundational definitions — if a term is still `unverified`, `doubted`, or `ignored`, any claim that relies on it inherits that uncertainty.

## How to satisfy it

1. **Verify the blocking term**: run `dont why <term-id>` to see what evidence is needed, then `dont dismiss <term-id> --evidence <uri>`.
2. **Resolve doubt on the term**: if the term is doubted, address the doubt with `dont undoubt <term-id>` after investigation.
3. **Remove the dependency** from the claim if the term relationship is not essential.

## Why it matters

Claims are only as strong as their conceptual foundations. `unresolved-terms` ensures that the dependency chain is fully verified before a claim is treated as settled.

## See also

- `lockable` — requires no `unresolved-term` derived assessments before a claim can be permanently preserved.
