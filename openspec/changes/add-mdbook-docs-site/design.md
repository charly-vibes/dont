## Context

`dont` is currently documented as a monolithic specification draft plus research notes. That material is valuable but difficult to scan, and it is not published as a navigable site. A docs site should explain the tool's purpose in a reader-friendly way without replacing the draft spec or research corpus.

## Goals
- Publish a simple public documentation site with standard Rust tooling
- Lead with a concise explanation of why `dont` exists
- Preserve traceability back to the draft spec and research artifacts
- Keep the publishing workflow boring and low-maintenance

## Non-Goals
- Generate Rust API docs from code that does not exist yet
- Reformat the entire repository into the docs site in one pass
- Introduce custom JavaScript, plugins, or a complex theme

## Decisions
- Use `mdBook` as the documentation generator because it is the conventional Rust tool for project-book documentation and works well with GitHub Pages
- Keep docs source in a dedicated book directory with standard `SUMMARY.md` navigation
- Start with a small information architecture: home page, purpose page, and pointers to the draft spec/research corpus
- Use a dedicated Pages workflow at `.github/workflows/docs.yml` for deployment and keep CI responsible for build verification

## Alternatives Considered
- `cargo doc`: rejected because it focuses on Rust API/reference documentation rather than narrative project documentation
- MkDocs: rejected because the user explicitly requested the standard Rust docs tool
- Hand-written static HTML: rejected because mdBook already solves navigation, builds, and Pages-friendly output

## Risks / Trade-offs
- Research documents are long and dense, so the purpose page could become too verbose
  - Mitigation: synthesize the purpose page and link to the full research artifacts for deep reading
- GitHub Pages deployment details can vary by repo settings
  - Mitigation: use the standard `actions/configure-pages`, `actions/upload-pages-artifact`, and `actions/deploy-pages` workflow shape

## Migration Plan
1. Define the docs-site capability in OpenSpec
2. Add minimal mdBook scaffolding and initial content
3. Build docs in CI
4. Publish docs from `main` to GitHub Pages

## Open Questions
- Whether the published URL should be surfaced in the README badge set
- Whether future docs should mirror the OpenSpec capability split or the monolithic draft structure
