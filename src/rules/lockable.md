# lockable

**Severity:** strict (non-overridable)

## What the rule checks

`lockable` enforces the gate that must be satisfied before a claim can be permanently preserved with `dont forget`. It requires:

1. The claim is in `verified` status.
2. At least **3 assessed competing hypotheses** — alternative explanations that have been explicitly evaluated.
3. At least **2 independent supporting evidence items** — from different sources, not correlated.
4. No integrity-compromising derived assessments (stale, compromised-support, dangling-dependency, or unresolved-term).

## How to satisfy it

- Add and assess hypotheses: `dont hypothesis add <id> "alternative explanation"` then `dont hypothesis assess <id> <idx> --supporting <uri>`.
- Add independent evidence: `dont dismiss <id> --evidence <uri1>` and `dont dismiss <id> --evidence <uri2>` from different sources.
- Resolve any integrity issues shown by `dont show <id>`.

## Why it matters

`dont forget` is a strong epistemic commitment. The lockable gate ensures that the claim was genuinely challenged, evaluated against alternatives, and grounded in independent corroboration before being permanently preserved.

## See also

- `stale-cascade` — produces the `stale` derived assessment that blocks `lockable`.
- `correlated-error` — fires when supporting evidence items share a host; resolving it helps satisfy the independent-evidence requirement.
- `dangling-definition` — produces the `dangling-dependency` derived assessment that blocks `lockable`.
- `unresolved-terms` — produces the `unresolved-term` derived assessment that blocks `lockable`.
