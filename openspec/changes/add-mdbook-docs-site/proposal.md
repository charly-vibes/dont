# Change: add mdBook docs site

## Why

The repository has strong research and specification material, but no reader-friendly documentation site that explains what `dont` is for. We need a public docs surface that turns the existing research into an accessible purpose page and publishes automatically to GitHub Pages.

## What Changes
- Add a documentation-site capability based on `mdBook`, the standard Rust documentation tool for book-style project docs.
- Define a first public-purpose page that explains the problem `dont` addresses using the existing research corpus and spec draft.
- Define build and deployment requirements for GitHub Pages.
- Separate local docs build validation from production publishing.

## Deferred
- API/reference docs generated from a Rust implementation
- Versioned documentation
- Search customization, theming, or custom plugins
- A full contributor handbook migration into the site

## Traceability
- Purpose and positioning derive mainly from section 1 and section 20 of `dont-spec-v0_3_2.md`.
- Research synthesis draws mainly from `0-dont-claude.md` and related `0-*.md` research artifacts.
- Workflow constraints derive from the existing GitHub Actions CI and repository conventions.

## Impact
- Affected specs: `docs-site`
- Affected docs: new `book/` content, `README.md`
- Affected workflow: `.github/workflows/ci.yml`, new `.github/workflows/docs.yml` Pages deployment workflow, mdBook config
