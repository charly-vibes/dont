# Introduction

`dont` is a command-line tool for autonomous LLM harnesses. Its job is narrow and opinionated: stop an agent from turning ungrounded text into asserted project truth.

This book explains why the problem exists, how `dont` solves it, and where to find detailed specs and research.

In this repo's workflow (not required to understand or use `dont`):

- `wai` helps track workflow and design intent
- `bd` tracks concrete work
- `dont` governs epistemic discipline

This book is the reader-friendly front door for the project. It does not replace the fuller OpenSpec decomposition or the archived monolithic draft in `wai` research. Instead, it explains the problem the tool is meant to solve and points to the deeper source material in the repository, especially the tracked research notes under `.wai/projects/dont/research/`. These notes are part of the project's workflow and research record managed with `wai`.

## Read this book if you want to know

- what `dont` is for
- why ordinary “please double-check yourself” prompting is not enough
- how to ground repository facts with `dont ground`, repository-relative locators, and `dont trace`
- how the project’s research maps onto the proposed tool

## Key sources

*(Note: External links point to the `main` branch of the repository.)*

- [OpenSpec change specs (`openspec/changes/*/specs/`)](https://github.com/charly-vibes/dont/tree/main/openspec/changes)
- [Research: forcing doubt in minds and machines](https://github.com/charly-vibes/dont/blob/main/.wai/projects/dont/research/2026-04-20-forcing-doubt-in-minds-and-machines-why-llms-de.md)
- [Research: designing doubt into AI interaction](https://github.com/charly-vibes/dont/blob/main/.wai/projects/dont/research/2026-04-20-designing-doubt-into-ai-interaction-what-the.md)
- [Repository README](https://github.com/charly-vibes/dont/blob/main/README.md)
