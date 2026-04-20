# Project Context

## Purpose

`dont` is a specification-first project for a proposed CLI that forces autonomous LLM agents to ground claims before asserting them. The repository currently captures design drafts, research inputs, and workflow metadata rather than an implementation.

## Tech Stack
- Markdown specifications and research artifacts
- `wai` for workflow context and design/research capture
- `bd` for issue tracking
- `OpenSpec` for structured capability specs and change proposals

## Project Conventions

### Code Style
- Primary artifacts are Markdown documents.
- Keep requirements normative and precise.
- Prefer small, capability-focused specs over monolithic documents.
- Use `just` recipes for common repo checks.

### Architecture Patterns
- Separate workflow (`wai`), issue tracking (`bd`), and capability specification (`OpenSpec`).
- Treat `dont-spec-v0_3_2.md` as the source draft to be decomposed into smaller capability specs.
- Prefer decomposition by capability boundary: CLI verbs, lifecycle/status model, data model, envelopes, imports, and integration rules.

### Testing Strategy
- Validate repository hygiene with `just ci`.
- Validate spec changes with `openspec validate <change-id> --strict`.
- As implementation emerges, add capability-level tests that correspond directly to OpenSpec scenarios.

### Git Workflow
- Make atomic commits with descriptive messages.
- Capture reasoning in `wai` before or alongside structural spec changes.
- Use OpenSpec proposals before large structural changes.

## Domain Context
- `dont` is an epistemic forcing-function CLI for LLM harnesses.
- It works alongside `wai` (workflow) and `beads`/`bd` (memory/issues), but remains an independent tool.
- Core concepts include claims, terms, evidence, an append-only event log, a status lattice, and a four-verb CLI core (`conclude`, `define`, `trust`, `dismiss`) plus lifecycle verbs.

## Important Constraints
- The repo does not yet implement `dont`; specs describe intended behaviour.
- Preserve traceability back to `dont-spec-v0_3_2.md` while splitting into capabilities.
- Keep the first OpenSpec changes manageable; do not attempt to migrate the entire monolith in one pass.

## External Dependencies
- `wai` for workflow artifacts
- `bd` for issue tracking
- `OpenSpec` for structured specification management
