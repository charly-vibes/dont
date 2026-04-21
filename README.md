# dont

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)

Specification-first repository for `dont`, a proposed CLI that forces autonomous LLM agents to ground claims before asserting them.

## Repo status

This repository currently contains design drafts and supporting notes, not an implementation yet.

Primary spec:
- `dont-spec-v0_3_2.md`

## Documentation

This repository now includes an `mdBook` documentation site.

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
