# Repository-grounding workflow

Use `dont` as a sidecar when you want repository facts to survive beyond chat memory.

## Fast path: ground a documented fact

When the claim and repository evidence are both in hand, use `dont ground`:

```bash
dont ground "The project builds docs with mdBook" --file justfile --lines 48-49
```

## Use repository-relative locators

For evidence inside the current project, use `--file` with an optional line span or anchor:

```bash
dont ground "The crate exposes a test recipe" --file justfile --lines 12-13
dont flag <id> --file src/main.rs --lines 188-205 --anchor "Ground"
```

Plain URI evidence is still supported for external sources:

```bash
dont flag <id> --evidence https://example.org/source
```

## Diagnose blockers with trace

If `show`, `why`, or `prime` reports stale dependencies, unresolved terms, or blocker labels without enough context, run:

```bash
dont trace <entity-id>
```

`trace` reports the blocker path so you can see which dependency or support relation needs attention before trying to verify or lock the claim.
