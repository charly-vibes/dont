# dangling-definition

**Severity:** strict (non-overridable)

## What the rule checks

`dangling-definition` fires when a claim depends on a `term:uuid` reference that no longer exists in the store. This can happen if a term was deleted after claims were created that depended on it.

Note: CURIE-format references (e.g. `WB:P001`) are handled by the `ungrounded` rule instead.

## How to satisfy it

1. **Restore the deleted term**: redefine it with `dont define <curie> --doc "..."` and relink the claim.
2. **Remove the dependency**: if the term relationship is no longer valid, update the claim.
3. **Investigate the deletion**: use `dont list --kind term` to see current terms.

## Why it matters

A claim that points to a non-existent term has an undefined foundational concept. `dangling-definition` catches referential integrity violations before they propagate through dependent claims.

## See also

- `ungrounded` — handles CURIE-format dependency references that cannot be resolved; `dangling-definition` handles `term:uuid` references specifically.
