# Contributing

## Workflow

1. Start with `wai status`.
2. Search existing project context with `wai search "topic"`.
3. Use `wai add research`, `wai add design`, or `wai add plan` to capture reasoning.
4. Run quality checks before committing:

```bash
just ci
```

## Project conventions

- Prefer `just` recipes for common commands.
- Keep repository context in `wai` artifacts when making decisions.
- Use `prek` hooks for basic hygiene checks.
- Update specs and project context together.

## Key commands

```bash
just status
just doctor
just way
just lint
just ci
```
