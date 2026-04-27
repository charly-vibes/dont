# Change: add core dont glossary spec

## Why
Currently, many load-bearing terms used across the `dont` OpenSpec decomposition are defined only in the source monolith or are used implicitly. This makes the capability specs hard to follow in isolation and creates avoidable ambiguity for implementers and AI agents.

This change establishes a first-class glossary capability so the current core specs can share a single definitional source instead of repeating term explanations inline.

## What Changes
- Add a new `dont-glossary` capability spec containing authoritative definitions for the current core `dont` vocabulary.
- Define canonical names and aliases for terms that are already used inconsistently (for example `epistemic lattice` and `status lattice`).
- Document a cross-linking convention that is valid after the glossary capability is archived into `openspec/specs/`.
- Update `openspec/project.md` to add a Term Index that points readers to the glossary capability.

## Deferred
- Full migration of every Appendix A term from the v0.3.2 monolith in one change
- Retrofitting links into every active change proposal under `openspec/changes/`
- Automated Markdown link verification beyond existing OpenSpec validation

## Impact
- New capability: `dont-glossary`
- Affected docs: `openspec/project.md`
- Affected future canonical specs: `dont-core`, `dont-status-lifecycle`, `dont-cli-core`, and adjacent capabilities after the glossary is archived
