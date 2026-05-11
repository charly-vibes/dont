# correlated-error

**Severity:** warn (configurable)

## What the rule checks

`correlated-error` fires when a claim's supporting evidence items all come from the same host or domain. Evidence from a single source (even multiple URLs on the same site) does not provide independent corroboration — if the source is wrong, all evidence fails together.

## How to satisfy it

Add evidence from at least one additional independent source:
```
dont dismiss <id> --evidence https://different-source.example/reference
```

Choose sources that are genuinely independent — different organizations, methodologies, or data sets.

## Why it matters

Independent corroboration is a core epistemic requirement. Correlated evidence (all from the same host) provides the appearance of multiple sources while actually representing a single point of failure.
