## Context

`dont-spec-v0_3_2.md` contains multiple kinds of information in one draft: purpose and positioning, invariants, data model, CLI behaviour, lifecycle rules, derived commands, import behaviour, and UX/documentation requirements. OpenSpec works better when these are split into focused capabilities with explicit requirements and scenarios.

## Goals
- Create the first usable OpenSpec decomposition for `dont`
- Capture the most central behavioural surfaces first
- Preserve room for future capability splits without rewriting these initial specs

## Non-Goals
- Convert the entire monolithic draft in one change
- Finalize implementation details for storage, networking, or output envelopes beyond the core requirements needed here
- Resolve every open question from the draft

## Decisions
- Start with three capabilities: `dont-core`, `dont-status-lifecycle`, and `dont-cli-core`
- Keep each capability broad enough to be coherent, but narrow enough that future changes can extend adjacent areas independently
- Treat the monolithic draft as source material rather than importing it verbatim
- Use these ownership boundaries:
  - `dont-core`: purpose, scope, companion-tool independence, and append-only invariants
  - `dont-status-lifecycle`: status vocabulary, transition semantics, terminal states, stale cascade, and audit context
  - `dont-cli-core`: command contracts for `conclude`, `define`, `trust`, and `dismiss`, including key refusal conditions

## Source Mapping
- `dont-core` derives primarily from v0.3.2 sections 1-3 (`Purpose`, `What dont is not`, and core invariants).
- `dont-status-lifecycle` derives primarily from section 5.1 (`Status lattice`) and related lifecycle semantics in sections 5.2 and 9A.
- `dont-cli-core` derives primarily from section 9 (`Primary CLI: four verbs`) and references lifecycle semantics rather than restating them.

## Alternatives Considered
- **Single giant capability**: rejected because it would reproduce the monolith inside OpenSpec.
- **Many tiny capabilities immediately**: rejected for the first split because it would create too much scaffolding before any usable baseline exists.

## Risks / Trade-offs
- Some requirements could later move between capabilities as the decomposition matures.
  - Mitigation: keep capability boundaries conceptual and explicit.
- The draft includes implementation-leaning detail that may be premature for initial OpenSpec extraction.
  - Mitigation: prioritize behavioural requirements over substrate-specific detail in the first pass.

## Migration Plan
1. Create initial capabilities from the v0.3.2 draft.
2. Validate the change strictly.
3. Use follow-on changes to extract remaining areas in this order:
   - lifecycle-adjacent verbs (`lock`, `reopen`, `ignore`, `verify-evidence`)
   - modes, initialization, and seed vocabulary
   - data model, evidence, and imports
   - envelopes, JSON contract, and error taxonomy
   - integration and operational concerns

## Open Questions
- Which remaining sections should be split next after the core pass?
- Which implementation details belong in future design notes rather than normative capability specs?
