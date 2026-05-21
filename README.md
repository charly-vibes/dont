# dont

`dont` is a Rust CLI for forcing epistemic discipline in autonomous LLM workflows.

It gives agents an explicit state machine for claims and terms so ungrounded assertions do not quietly become accepted project truth.

## What it does

- records claims with `dont conclude`
- records project vocabulary with `dont define`
- registers doubt with `dont trust`
- verifies with evidence via `dont flag`
- supports lifecycle and inspection commands like `ground`, `show`, `list`, `why`, `trace`, `lock`, and `prime`

## Why it exists

Prompting a model to “be more careful” is weaker than forcing it through an external workflow.
`dont` makes that workflow explicit: claims, evidence, status transitions, refusals, and remediation all become machine-checkable.

## Project status

This repository contains both:

- a working Rust implementation
- OpenSpec source-of-truth specs in `openspec/`
- user-facing docs in `docs/`
- workflow context in `.wai/`

## Quick start

```bash
git clone https://github.com/charly-vibes/dont.git
cd dont
cargo build
cargo test
```

Common local commands:

```bash
just test
just lint
just ci
```

## Learn more

- Book intro: `docs/introduction.md`
- Tutorial: `docs/tutorial.md`
- Purpose: `docs/purpose.md`
- Grounding workflow: `docs/grounding-workflow.md`
- OpenSpec project context: `openspec/project.md`
- Contributing guide: `CONTRIBUTING.md`
