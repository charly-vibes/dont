# ungrounded

**Severity:** strict (non-overridable)

## What the rule checks

`ungrounded` fires when a claim depends on a CURIE reference (e.g. `WB:P001`) that cannot be resolved to any known term in the current project. A CURIE dependency that points to an undefined or unimported term means the claim's meaning is partially undefined — the claim cannot be fully evaluated until all its terms exist.

Note: `term:uuid` ID-format dependencies are handled by the `dangling-definition` rule instead.

## How to satisfy it

1. **Define the term** in the current project: `dont define WB:P001 --doc "..."`.
2. **Import the term** from an external ontology: `dont import obo WB:P001` (or the appropriate adapter).
3. **Remove the dependency** from the claim if it was added by mistake.

## Why it matters

A claim that references undefined concepts cannot be verified or assessed because its meaning is incomplete. `ungrounded` blocks verification until all CURIE dependencies resolve to known, defined terms.
