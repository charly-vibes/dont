## Context

The spec decomposition now covers verbs, lifecycle, envelopes, errors, data shapes, harness/help surfaces, rules, and imports. The remaining major monolith section is the persistent project structure and configuration surface in §14. These are foundational because other capabilities already assume `.dont/AGENTS.md`, rules directories, schema locations, managed-doc targets, and config-driven behaviour such as harness mode, rule severities, and evidence verification tuning.

## Goals
- Capture the strictly self-contained `.dont/` on-disk contract as a standalone capability, mandating "convention over configuration" for auto-loading rule files.
- Capture the externally visible `config.toml` surface as a separate capability, requiring the CLI to fail deterministically (no silent defaults) if it is missing or unparsable.
- Preserve cross-feature relationships without restating each dependent spec in full, while guaranteeing safe constraints (like non-regex substring matching for hedges and rate limits for evidence checks).

## Non-Goals
- Specify low-level storage engine implementation details
- Specify future migration commands or out-of-scope security/auth features
- Re-state the behaviour of imported/rule/harness subsystems beyond their config-facing contracts

## Decisions
- **Two capabilities, not one**: layout and config change independently. A new subdirectory should not require editing config semantics, and a new tuning knob should not imply a directory-layout change.
- **Strictly self-contained layout**: The CLI MUST NOT create or rely on persistent state outside the `.dont/` folder (except for rewriting managed blocks in root docs via `sync-docs`).
- **Fail on Missing Config**: The CLI MUST refuse to run if `config.toml` is missing or unparsable. It MUST NOT silently fall back to defaults, ensuring epistemic policy cannot be bypassed.
- **Convention over Configuration for Rules**: Rules in `.dont/rules/` are loaded and evaluated automatically based on their filename base. `config.toml` is used only to configure their severity or explicitly disable them.
- **Harness vs LLM separation**: The `[harness]` block (orchestration, spawn protocols) is strictly separated from the `[llm]` block (API keys, direct-mode models) to reinforce `dont`'s role as an orchestrator of other agents, not a standalone AI assistant.
- **Deterministic Hedges**: The `[trust.hedges]` patterns MUST be evaluated as case-insensitive substrings, not regular expressions, to prevent ReDoS and ensure fast validation.
- **Network Politeness**: The `[verify_evidence]` config MUST include a configurable `max_concurrent_requests` (default 5) to throttle parallel HTTP checks.

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
