# Change: add explicit dependency/blocker tracing

## Why

A graph-aware epistemic tool only adds durable value if blocked claims and terms are explainable. Labels such as `stale`, `dangling-dependency`, or `unresolved-term` are not enough on their own. Operators need a short, actionable account of *which path caused the blocker* and *what to do next*.

## What Changes
- Add a read-only tracing query for dependency/support fallout.
- Define the trace as traversing the same dependency/support relationships used by current blocker and derived-assessment analysis, rather than inventing a second graph model.
- Define structured blocker paths that explain why an entity is currently stale, unresolved, or otherwise gated.
- Define remediation-rich output suitable for both humans and harnesses.
- Add documentation positioning tracing as the main diagnostic tool when a verification workflow is blocked.

## Deferred
- Full graph visualization
- Automatic repair actions
- Cross-project dependency tracing
- Weighting or ranking of alternative support paths

## Change Type
- New capability and usability strengthening, not merely an implementation repair

## Related Changes
- Complements current lifecycle/query behavior and pairs well with `add-ground-command` when grounded claims later become blocked

## Impact
- Affected specs: `dont-trace-query`, `dont-derived-queries`, `dont-payload-types`, `dont-agent-help`
- Affected code: new read-only query, dependency analysis projection, structured trace payloads, tutorial/how-to output
- Affected workflow: blocked knowledge graphs become diagnosable instead of opaque
