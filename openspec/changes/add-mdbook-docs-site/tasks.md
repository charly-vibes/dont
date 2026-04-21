## 1. Define the documentation capability
- [x] 1.1 Specify the mdBook-based documentation source and navigation structure
- [x] 1.2 Specify the purpose page requirements and source traceability to research/spec materials
- [x] 1.3 Specify local build validation and GitHub Pages publishing requirements

## 2. Implement the docs site
- [x] 2.1 Add mdBook configuration and book source files
- [x] 2.2 Write the initial purpose-focused documentation page
- [x] 2.3 Link the repository README to the published/local docs entry point

## 3. Automate build and publish
- [x] 3.1 Update CI to build the mdBook site as part of repository checks
- [x] 3.2 Add `.github/workflows/docs.yml` to publish the built site to GitHub Pages on `main`

## 4. Validate
- [x] 4.1 Run `openspec validate add-mdbook-docs-site --strict`
- [x] 4.2 Run the relevant local docs build/check commands successfully
