# dont — ground your agents before they speak

> *Don't speak*
> *I know just what you're sayin'*
> *So please stop explainin'*
> *Don't tell me 'cause it hurts*
> *Don't speak*
> *I know what you're thinkin'*
> *I don't need your reasons*
> *Don't tell me 'cause it hurts*
>
> — No Doubt, "Don't Speak"

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)

`dont` is a CLI for developers building LLM pipelines who want agents to verify claims before asserting them as fact.

Without a gate like `dont`, an agent composes an answer from plausible-sounding steps and ships it. With `dont`, each claim in the agent's output carries a proof obligation — and the pipeline halts until that obligation is met. The result: fewer confident hallucinations reaching downstream systems or users.

## Install

**Prerequisites**: [Rust toolchain](https://rustup.rs) (stable).

```bash
git clone https://github.com/charly-vibes/dont
cd dont
just install        # cargo install --path . --locked
```

The `dont` binary is installed to `~/.cargo/bin/dont`. Confirm with:

```bash
dont --version
```

## Quick Start

Initialize `dont` in a project, then introduce and verify a claim:

```bash
# 1. Initialize claim tracking in the current project
dont init

# 2. Record an unverified claim
dont conclude "The primary database uses optimistic locking"

# 3. List all claims (output shows claim IDs and statuses)
dont list --human

# 4. Verify a claim with evidence
dont flag claim:<ID> --evidence "https://example.com/db-docs#locking" --human

# 5. See project summary
dont prime --human
```

Claims flow through states: `unverified` → `verified` (via `dont flag`) or `doubted` (via `dont trust`). Use `dont ignore` to suppress a claim; `dont lock` to permanently preserve a verified claim.

## Status

dont is in design phase — the spec is the artifact. Contributions to the specification are welcome.

Active specifications:

- [`openspec/changes/add-rule-claim-schema/`](openspec/changes/add-rule-claim-schema/) — core: claim schema
- [`openspec/changes/add-mdbook-docs-site/`](openspec/changes/add-mdbook-docs-site/) — infrastructure: docs deployment

See [`openspec/AGENTS.md`](openspec/AGENTS.md) for the proposal workflow.

## Documentation

- Read the docs: https://charly-vibes.github.io/dont/
- Source: `docs/`
- Config: `book.toml`
- Local build: `just docs-build`
- Published: GitHub Pages via `.github/workflows/docs.yml`

## Prerequisites

- [wai](https://github.com/charly-vibes/wai) — workflow context and research tracking
- [bd / beads](https://github.com/charly-vibes/beads) — issue tracking (`bd` CLI)
- [just](https://just.systems) — command runner
- [mdBook](https://rust-lang.github.io/mdBook/) — local docs build (`just docs-build`)

## Contributing

This repo uses `wai` for workflow context and `bd` for issue tracking.

Common commands:

```bash
just status      # wai status — active project phase and suggestions
just ready       # bd ready  — unblocked issues to pick up
just doctor      # wai doctor
just sync        # wai sync
just docs-build  # build the mdBook site locally
```

For full output or interactive search, run `wai status`, `wai show`, or `wai search "<topic>"` directly.

---

Apache 2.0 — see [LICENSE](LICENSE)
