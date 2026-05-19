# stale-cascade

**Severity:** strict (non-overridable)

## What the rule checks

`stale-cascade` fires when a verified claim has an assessment (e.g. a hypothesis or atom assessment) that was recorded before its supporting evidence was attached. When evidence is added after an assessment, the assessment may have been made without that evidence — making it potentially stale.

## How to satisfy it

Re-assess the affected hypotheses or atoms after reviewing the new evidence:
- For hypotheses: `dont hypothesis assess <claim-id> <idx> --supporting <uri>` (or `--refuting`).
- For atoms: `dont atom dismiss <claim-id> <idx> --evidence <uri>`.

## Why it matters

An assessment made before key evidence was available may reach a different conclusion than one made with the full picture. `stale-cascade` ensures that all assessments reflect the current evidence state.

## See also

- `lockable` — requires no `stale` derived assessments before a claim can be permanently preserved.
