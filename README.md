# dont

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)

Specification-first repository for `dont`, a proposed CLI that forces autonomous LLM agents to ground claims before asserting them.

## Repo status

This repository currently contains design drafts and supporting notes, not an implementation yet.

Primary spec set:
- `openspec/changes/*/specs/`

Archived monolithic draft:
- `.wai/projects/dont/research/2026-04-21-dont-specification-v0-3-2-draft-post-ux-dx.md`

## Documentation

This repository now includes an `mdBook` documentation site.

- Read the docs: https://charly-vibes.github.io/dont/
- Source: `docs/`
- Config: `book.toml`
- Local build: `just docs-build`
- Published site: GitHub Pages via `.github/workflows/docs.yml`

## Workflow

This repo is tracked with `wai` for workflow context and `bd` for issue tracking.

Common commands:

```bash
just status   # wai status
just doctor   # wai doctor
just way      # wai way
just sync     # wai sync
just ready    # bd ready
just docs-build
```

For deeper project context:

```bash
wai status
wai show
wai search "topic"
```
