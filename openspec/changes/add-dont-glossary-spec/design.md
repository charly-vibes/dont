## Context
The `dont` project is being decomposed from a large monolithic draft into smaller OpenSpec capabilities. During the initial pass, the glossary (Appendix A of the monolith) was left out of scope, leaving many terms in `dont-core`, `dont-status-lifecycle`, and `dont-cli-core` without formal definitions inside the new structure.

The immediate problem is not that every Appendix A term is missing everywhere. The immediate problem is that the current core capability set lacks a shared definitional home for the terms it already depends on.

## Goals
- Provide a single source of truth for the current core `dont` vocabulary.
- Make canonical specs easier to navigate once the glossary capability is archived into `openspec/specs/`.
- Reduce duplicated explanations in behavior-owning specs by moving definitions into one focused capability.
- Establish canonical term names and alias policy where usage has already drifted.

## Non-Goals
- Migrate every monolith glossary term in this one change.
- Retrofit every active proposal in `openspec/changes/` to link to the glossary before archival.
- Invent a custom glossary renderer or OpenSpec extension.

## Decisions
- **Glossary as a capability**: The glossary will be a standalone capability (`dont-glossary`) rather than a long appendix in `project.md`.
- **Definitional requirements**: Each glossary entry will be written as a definitional requirement (`The glossary SHALL define X as ...`) rather than as the owning behavioral rule. Behavioral semantics remain owned by the relevant capability specs.
- **Core-first scope**: This change covers the vocabulary currently needed by the core decomposition (`dont-core`, `dont-status-lifecycle`, `dont-cli-core`) plus immediately adjacent load-bearing terms such as `hedge pattern`, `rule`, `evidence`, and `author string`.
- **Alias policy**: Each concept gets one canonical glossary term. Existing alternate phrasings are recorded as aliases in the relevant entry. For example, `status lattice` is treated as an alias of `epistemic lattice`.
- **Archive-safe linking convention**: Cross-links are documented for canonical specs only, after archive into `openspec/specs/`. The standard relative form is:
  - From one canonical capability spec to another: `[Atom](../dont-glossary/spec.md#requirement-atom)`
  - Link the first occurrence of a glossary term in each spec section or requirement block where the term is load-bearing.
- **Proposal-phase linking policy**: Active change proposals under `openspec/changes/` are not required to link to `dont-glossary` before archive, because the canonical target path does not yet exist.
- **Validation claim narrowed**: `openspec validate` remains the structural validator for the change. Markdown link verification is treated as separate follow-on hygiene, not as a current guarantee.
- **Project index**: `openspec/project.md` will gain a concise Term Index that points to `dont-glossary` and names the highest-value canonical terms.

## Alternatives Considered
- **Inline definitions in each spec**: Rejected because they duplicate prose and create drift.
- **Glossary only in `project.md`**: Rejected because it mixes domain vocabulary with meta-project conventions and is harder to evolve with capability-level changes.
- **Full Appendix A migration now**: Rejected because it would turn a focused fix into a monolith re-import.

## Risks / Trade-offs
- **Partial glossary risk**: Readers may assume the glossary is exhaustive.
  - Mitigation: Scope the proposal explicitly as a core glossary slice and list full Appendix A migration as deferred.
- **Alias drift**: Different specs may keep using alternate names.
  - Mitigation: Record aliases in the glossary and prefer the canonical name in future spec edits.
- **Link rot after requirement renames**: Header-derived anchors can change if a glossary term is renamed.
  - Mitigation: Keep glossary requirement names stable; if renaming becomes necessary, update inbound links as part of the same change.
