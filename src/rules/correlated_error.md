# correlated-error

**Severity:** warn (configurable)

## What the rule checks

`correlated-error` fires when a claim has at least two evidence items and one or more hosts appear more than once across those items. Evidence items are compared by host: two URLs on the same domain count as the same source regardless of path.

The rule only runs when a claim has at least 2 evidence items total. A claim with 0 or 1 evidence item is silently skipped.

## How to satisfy it

Remove duplicate-host evidence items, or replace them with items from independent sources:
```
dont dismiss <id> --evidence https://different-source.example/reference
```

Choose sources that are genuinely independent — different organizations, methodologies, or data sets.

## Why it matters

Independent corroboration is a core epistemic requirement. Correlated evidence (multiple items from the same host) provides the appearance of multiple sources while actually representing a single point of failure.
