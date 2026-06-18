# Specs

The OpenSpec specs define `dont`'s capabilities precisely and are the authoritative source for implementation decisions. They live in `openspec/specs/` in the repository.

## Core specs

| Spec | Description |
|---|---|
| [dont-core](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-core/spec.md) | Core claim lifecycle: conclude, flag, trust, lock |
| [dont-data-model](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-data-model/spec.md) | Store schema and entity relationships |
| [dont-status-lifecycle](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-status-lifecycle/spec.md) | Status transitions, gate conditions, and mode enforcement |
| [dont-lifecycle-verbs](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-lifecycle-verbs/spec.md) | Command-by-command verb semantics |
| [dont-errors](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-errors/spec.md) | Structured error types and remediation messages |

## Interface specs

| Spec | Description |
|---|---|
| [dont-cli-surface](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-cli-surface/spec.md) | CLI flags, subcommands, exit codes, output routing |
| [dont-cli-core](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-cli-core/spec.md) | Core CLI wiring and dispatch |
| [dont-envelope](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-envelope/spec.md) | JSON Envelope and Error protocol (`--json` output) |
| [dont-payload-types](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-payload-types/spec.md) | Envelope payload schemas |
| [dont-agent-help](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-agent-help/spec.md) | Help text and agent-readable command descriptions |

## Evidence and grounding specs

| Spec | Description |
|---|---|
| [dont-evidence-locators](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-evidence-locators/spec.md) | Repository-relative file/lines/anchor syntax |
| [dont-ground-command](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-ground-command/spec.md) | `dont ground` fast-path semantics |
| [dont-trace-query](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-trace-query/spec.md) | Blocker-path tracing for `dont trace` |

## Rule engine specs

| Spec | Description |
|---|---|
| [dont-rule-engine](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-rule-engine/spec.md) | Shipped rules, severities, and project config |
| [dont-rule-cli](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-rule-cli/spec.md) | `dont rules` subcommand surface |
| [dont-rule-claim-schema](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-rule-claim-schema/spec.md) | Claim schema validation rule |
| [dont-derived-queries](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-derived-queries/spec.md) | Datalog-based derived query layer |

## Infrastructure specs

| Spec | Description |
|---|---|
| [dont-build](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-build/spec.md) | Build, CI, and release pipeline |
| [dont-project-layout](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-project-layout/spec.md) | Repository layout conventions |
| [dont-project-config](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-project-config/spec.md) | Per-project `.dont/` configuration |
| [dont-init-modes](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-init-modes/spec.md) | Strict vs permissive initialisation modes |
| [dont-glossary](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-glossary/spec.md) | Canonical terminology |

## Future / in-progress specs

| Spec | Description |
|---|---|
| [dont-harness-cli](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-harness-cli/spec.md) | Agent harness integration |
| [dont-spawn-protocol](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-spawn-protocol/spec.md) | Clean-context spawn for independent verification |
| [dont-mcp-interface](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-mcp-interface/spec.md) | MCP server interface |
| [dont-import-surface](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-import-surface/spec.md) | External data import surface |
| [dont-linkml-import](https://github.com/charly-vibes/dont/tree/main/openspec/specs/dont-linkml-import/spec.md) | LinkML schema import |
