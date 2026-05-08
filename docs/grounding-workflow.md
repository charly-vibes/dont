# Repository-grounding workflow

Use `dont` as a sidecar when you want repository facts to survive beyond chat memory.

## Fast path: ground a documented fact

When the claim and repository evidence are both in hand, prefer `dont ground` with a repository-relative locator:

```bash
dont ground "The project builds docs with mdBook" --file justfile --lines 48-49
```

This is the shortest trustworthy path for documented project facts. It records the claim and verifies it in one invocation.

`ground` is a convenience command, not a separate epistemic model. The underlying lifecycle remains:

1. `conclude` — introduce an unverified claim
2. `trust` — register doubt
3. `flag` — verify with evidence
4. `lock` — freeze a mature verified claim when the lockable gate is met

Internally, `ground` composes `conclude` and `dismiss` so normal event history and status rules still apply.

## Prefer repository-relative locators

For evidence inside the current project, use `--file` with an optional line span or anchor:

```bash
dont ground "The crate exposes a test recipe" --file justfile --lines 12-13
dont flag <id> --file src/main.rs --lines 188-205 --anchor "Ground"
```

Repository-relative locators are preferred over opaque absolute `file://` URIs because they remain readable, auditable, and scoped to the project root. Absolute paths, `..` traversal, and symlink escapes are refused for project evidence.

Plain URI evidence is still supported for compatibility and external sources:

```bash
dont flag <id> --evidence https://example.org/source
```

## Diagnose blockers with trace

If `show`, `why`, or `prime` reports stale dependencies, unresolved terms, or blocker labels without enough context, run:

```bash
dont trace <entity-id>
```

`trace` reports the blocker path so you can see which dependency or support relation needs attention before trying to verify or lock the claim.
