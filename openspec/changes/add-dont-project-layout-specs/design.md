## Context

The spec decomposition now covers verbs, lifecycle, envelopes, errors, data shapes, harness/help surfaces, rules, and imports. The remaining major monolith section is the persistent project structure and configuration surface in §14. These are foundational because other capabilities already assume `.dont/AGENTS.md`, rules directories, schema locations, managed-doc targets, and config-driven behaviour such as harness mode, rule severities, and evidence verification tuning.

## Goals
- Capture the `.dont/` on-disk contract as a standalone capability
- Capture the externally visible `config.toml` surface as a separate capability
- Preserve cross-feature relationships without restating each dependent spec in full

## Non-Goals
- Specify low-level storage engine implementation details
- Specify future migration commands or out-of-scope security/auth features
- Re-state the behaviour of imported/rule/harness subsystems beyond their config-facing contracts

## Decisions
- **Two capabilities, not one**: layout and config change independently. A new subdirectory should not require editing config semantics, and a new tuning knob should not imply a directory-layout change.
- **Filesystem roles are normative, implementations are not**: the spec names the directories and what they are for, but does not lock in internal file formats beyond what the operator or adjacent capability observes.
- **Config expresses behavioural seams**: the config spec captures knobs that alter externally visible behaviour, such as project mode, rule severity, spawn timeout, and evidence verification politeness.
- **Cross-referenced side effects stay visible**: some config fields imply runtime behaviour already specified elsewhere (e.g. mode changes, direct-mode only LLM config). Those effects are referenced so the config surface is meaningful in isolation.

## Source Mapping
- `dont-project-layout`: §14 directory tree and comments about canonical docs / managed root docs
- `dont-project-config`: §14 `config.toml` example plus linked behaviour in §§8, 9, 9A, 12, 13, and 15

## Risks / Trade-offs
- The config surface touches many other capabilities and could become duplicative.
  - Mitigation: reference dependent capabilities for behaviour and focus this spec on exposed configuration contracts.
- The layout spec may feel static, but it is important for init/migration and for harnesses locating managed docs.
  - Mitigation: make each directory's role explicit and testable.

## Open Questions
- Whether future seed-migration or multi-user features will require splitting the layout capability further.
