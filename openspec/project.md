# Project Context

## Purpose

`dont` is a specification-first project for a proposed CLI that forces autonomous LLM agents to ground claims before asserting them. The repository currently captures design drafts, research inputs, and workflow metadata rather than an implementation.

## Tech Stack
- Markdown specifications and research artifacts
- Planned implementation: Rust single-binary CLI with auxiliary out-of-process import tooling when required
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

## Monolith Coverage Notes
- `dont-spec-v0_3_2.md` remains the archived source draft; OpenSpec is the decomposition target.
- Normative behavioural sections are covered by capability specs under `openspec/changes/*/specs/`:
  - §§1-3 → `add-core-dont-specs`
  - §§4.2-6, 10.4, 10.6 → `add-dont-data-model-specs`
  - §§4.4, 7-9A → `add-dont-operational-specs`
  - §§9, 10.2-10.7 → `add-core-dont-specs` and `add-dont-envelope-specs`
  - §§10-12, 16 → `add-dont-harness-specs`
  - §13 → `add-dont-rules-specs`
  - §14 → `add-dont-project-layout-specs`
  - §15 → `add-dont-import-specs`
- Architectural and informative monolith sections are covered by OpenSpec project/design context rather than capability specs:
  - §4.1 (Rust/single-binary choice) → `openspec/project.md` plus change design docs
  - §4.2 substrate rationale (CozoDB choice) → `add-dont-data-model-specs/design.md`
  - §§17-21 (out of scope, open questions, evaluation, references, glossary/changelog) remain draft/reference context and do not decompose into standalone capabilities in the first pass.

## External Dependencies
- `wai` for workflow artifacts
- `bd` for issue tracking
- `OpenSpec` for structured specification management
