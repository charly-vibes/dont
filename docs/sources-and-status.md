# Sources and status

This documentation site is an overview, not the project’s only source of truth.

## What each source is for

- **This mdBook (`docs/`)** gives a reader-friendly explanation of the project’s purpose and research basis.
- **The [OpenSpec change specs](https://github.com/charly-vibes/dont/tree/main/openspec/changes)** are the detailed design documents for the tool's capabilities, decomposed by feature area.
- **Tracked [research notes](https://github.com/charly-vibes/dont/tree/main/.wai/projects/dont/research/)** capture the reasoning and source synthesis behind the project’s design, including the archived monolithic draft.
- **[OpenSpec](https://github.com/charly-vibes/dont/tree/main/openspec/)** is the structured specification workspace with top-level specs and capability-focused change proposals.

## Current project status

`dont` has a working Rust implementation with an active CLI surface. The repository contains:

- a functional implementation (`src/`)
- OpenSpec change specs for planned capabilities (`openspec/changes/*/specs/`)
- top-level formal specs (`openspec/specs/`)
- tracked research artifacts explaining the design rationale (`.wai/projects/dont/research/`)
- this mdBook documentation (`docs/`)

This book should be read as:

- an introduction to the project
- a guide to why the tool is designed this way
- a pointer to the more detailed project artifacts

## How to read the project

If you are new here, read in this order:

1. this book for the overview
2. `openspec/changes/*/specs/` for the detailed capability-by-capability design
3. `.wai/projects/dont/research/` for the justification, research trail, and archived monolithic draft
4. `openspec/` for the broader structured specification workspace

## Important distinction

The mdBook explains the project in narrative form.

The OpenSpec change specs and related artifacts define the project more precisely.

The `.wai` research artifacts explain why those design choices were made.
