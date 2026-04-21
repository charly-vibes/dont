## Context

The repo now has focused specs for the core verbs, lifecycle verbs, envelope/error contracts, and data shapes. The next gap is the harness-facing behaviour that stitches those pieces together: what the read-only derived commands mean, how the CLI teaches the LLM to use itself, how spawn requests are emitted and resolved, and how the optional MCP transport exposes the same command surface. The monolith currently spreads these concerns across §§10–12 and §16.

## Goals
- Capture the read-only derived command surface as a standalone capability separate from write verbs
- Capture the agent-facing documentation/help contract as a first-class capability rather than prose-only guidance
- Capture spawn-request orchestration and timeout rules without dragging in the whole project-layout or import surface
- Capture the optional MCP server mode as a transport contract rather than leaving it as a monolith-only note

## Non-Goals
- Specify importer behaviour or source adapters
- Specify the rule engine internals or Datalog file format
- Specify full project layout or `config.toml` schemas beyond what the harness contract depends on

## Decisions
- **Four capabilities, not one**: query commands, agent-help/docs, spawn orchestration, and MCP transport evolve on different axes. A new help topic should not require editing the spawn protocol spec; a timeout policy change should not touch `list` semantics; an MCP transport tweak should not restate query semantics.
- **Read-only commands grouped together**: `list`, `vocab`, `show`, `why`, `prime`, `doctor`, `schema`, and `examples` are all derived, non-mutating, and primarily return previously defined payload types. Keeping them together avoids tiny per-command specs with repetitive cross-references.
- **Teaching surface treated as normative UX contract**: the managed docs block, orientation text, tutorial, and how-to entry points are not implementation notes; they are part of how `dont` enforces disciplined agent behaviour.
- **Spawn protocol owns command orchestration, not payload shape**: `dont-payload-types` defines `SpawnRequest`; this change defines when and why that payload is emitted, how harness/direct mode is selected, and what timeout recovery means.
- **MCP is a transport wrapper, not a privileged API**: the MCP capability specifies how the CLI surface is re-exposed over stdio and how tool results carry envelopes, while reusing the same underlying command contracts as the direct CLI.

## Source Mapping
- `dont-derived-queries`: §10 command summary lines for `list`, `vocab`, `show`, `why`, `prime`, `doctor`, `schema`, and `examples`; payload cross-refs in §10.4; help references in §10.7.6
- `dont-agent-help`: §10.7.6 help surface, §11 managed block, §11.1 orientation block, §11.2 worked example, §11.3 tutorial, §11.4 how-to guides
- `dont-spawn-protocol`: §10 command summary lines for `guess`, `assume`, `overlook`, and `spawns`; §12 harness transports, detection, timeouts, and late-callback semantics
- `dont-mcp-interface`: §16 optional MCP interface

## Risks / Trade-offs
- The derived-query spec may look broad because it covers several commands.
  - Mitigation: keep each command family in its own requirement with explicit scenarios.
- The help surface is partly documentation, which can tempt non-normative wording.
  - Mitigation: use SHALL/MUST language only for externally visible behaviours and entry points.
- Spawn behaviour overlaps with CLI flags and payload types.
  - Mitigation: reference `dont-cli-surface` for flags and `dont-payload-types` for shapes instead of restating them.
- MCP transport could drift from the direct CLI contract.
  - Mitigation: specify MCP as a transport layer that reuses the same envelopes, commands, and semantics rather than a separate API.

## Open Questions
- Whether `dont explain <rule>` should live with the help surface or the future rule-system spec. This change defers it except for help-entry routing.
