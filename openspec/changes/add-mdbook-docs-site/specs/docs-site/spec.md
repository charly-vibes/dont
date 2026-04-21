## ADDED Requirements

### Requirement: mdBook documentation source
The repository SHALL define its narrative documentation site as an `mdBook` project committed in the repository, with a standard summary-driven navigation structure that can be built locally and in CI.

#### Scenario: Local docs source exists
- **WHEN** a contributor inspects the repository
- **THEN** they can find an `mdBook` configuration file and book source files in version control
- **AND** the book defines navigation through a standard `SUMMARY.md`

### Requirement: Purpose-first documentation page
The documentation site SHALL include a page that explains the purpose of `dont` in reader-friendly language, grounded in the repository's existing research and specification artifacts.

#### Scenario: Reader wants to understand why the tool exists
- **WHEN** a reader opens the documentation site
- **THEN** they can find a purpose-oriented page that explains the problem `dont` addresses
- **AND** the page summarizes why autonomous LLMs need external grounding and verification loops before asserting claims
- **AND** the page links to the deeper draft specification and research artifacts for traceability

### Requirement: Build validation in continuous integration
The repository SHALL validate that the documentation site builds successfully in continuous integration before changes are merged.

#### Scenario: Pull request changes docs or workflow
- **WHEN** CI runs for a pull request or push
- **THEN** the workflow builds the `mdBook` site
- **AND** the job fails if the docs site cannot be generated

### Requirement: GitHub Pages publication
The repository SHALL publish the built documentation site to GitHub Pages from the default branch using a dedicated GitHub Actions deployment workflow at `.github/workflows/docs.yml`.

#### Scenario: Main branch updates docs
- **WHEN** documentation changes are merged to `main`
- **THEN** GitHub Actions builds the `mdBook` output and deploys it to GitHub Pages
- **AND** pull requests do not perform a production deployment
