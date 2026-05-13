## MODIFIED Requirements

### Requirement: Managed agent-document block

The system SHALL own a managed documentation block describing `dont` usage in project docs. `.dont/AGENTS.md` MUST be the canonical LLM-facing document and MUST be a fully-managed file overwritten in whole by `dont init` and by `dont doctor --fix`. Root-level managed blocks in files such as `AGENTS.md` and `CLAUDE.md` MUST remain shorter pointers to `.dont/AGENTS.md`. The managed block boundaries MUST be delimited by the sentinel pair `<!-- DONT:START -->` and `<!-- DONT:END -->`, and the content between those sentinels MUST be treated as tool-owned and overwritable by `dont doctor --fix`. The root block content MUST be minimal but agent-prominent: it MUST contain (a) a clearly visible warning that the region is auto-managed and that edits inside the markers will be overwritten, (b) an actionable session-start command instructing the agent to run `dont prime --json`, and (c) an explicit pointer to `.dont/AGENTS.md` as the canonical document. The root block MUST NOT contain the full canonical instructions; the full instructions live only in `.dont/AGENTS.md`.

#### Scenario: managed block points to canonical docs

- **WHEN** `dont` renders the managed block into a project document
- **THEN** the block tells the reader to run `dont prime --json` at session start
- **AND** it points to `.dont/AGENTS.md` as the canonical document
- **AND** it does not duplicate the full canonical instructions inline

#### Scenario: managed block marks overwrite boundary

- **WHEN** a project document contains the managed block markers `<!-- DONT:START -->` and `<!-- DONT:END -->`
- **THEN** the content between those markers is treated as tool-owned and overwritable by `dont doctor --fix`

#### Scenario: managed block carries a prominent do-not-edit warning

- **WHEN** an agent or operator reads a root document containing the managed block
- **THEN** the block opens with a visible warning that the region is auto-managed
- **AND** the warning is prominent enough that an agent scanning the file cannot reasonably skip the session-start instruction

#### Scenario: canonical file is fully managed

- **WHEN** `dont init` or `dont doctor --fix` rewrites `.dont/AGENTS.md`
- **THEN** the entire file is replaced with the generator output
- **AND** the file's header declares it is managed by `dont` and must not be hand-edited

## REMOVED Requirements

### Requirement: Managed-doc sync command

**Reason**: The `dont sync-docs` verb is removed. Its responsibilities — rewriting the managed block in configured root documents and overwriting `.dont/AGENTS.md` — fold into `dont doctor --fix`, which already owns project health. This eliminates a single-purpose verb and gives operators one command for diagnose-and-repair.

**Migration**: No installed-base migration is required because `dont` has no implementation yet at the time of this change. Operators or scripts that would have called `dont sync-docs` MUST call `dont doctor --fix` instead. Spec references to `sync-docs` in other capabilities are updated by the same change set (see `dont-project-config`).

## ADDED Requirements

### Requirement: Managed-docs staleness check

The system SHALL include a managed-docs staleness check in `dont doctor`. The check MUST regenerate the expected content for the root managed block and for `.dont/AGENTS.md` from the current project state, normalize both expected and on-disk content by converting line endings to `\n` and trimming trailing whitespace, compare the normalized outputs exactly (between the sentinels for the root block; whole-file for `.dont/AGENTS.md`), and report a `warn` status when any configured target diverges. The check MUST be read-only and MUST NOT modify any file.

#### Scenario: clean project produces an ok check

- **WHEN** the caller runs `dont doctor --json` on a project whose managed blocks match the generator output
- **THEN** the doctor envelope reports the managed-docs check with `status: "pass"`
- **AND** no file on disk is modified

#### Scenario: edited block produces a warn check

- **WHEN** a configured root managed block diverges from the generator output (for example, an operator has edited content between the sentinels)
- **AND** the caller runs `dont doctor --json`
- **THEN** the doctor envelope reports the managed-docs check with `status: "warn"`
- **AND** the check's remediation entry instructs the operator to run `dont doctor --fix`
- **AND** no file on disk is modified

#### Scenario: missing block produces a warn check

- **WHEN** a configured root document exists but does not contain the `<!-- DONT:START -->` / `<!-- DONT:END -->` markers
- **AND** the caller runs `dont doctor --json`
- **THEN** the doctor envelope reports the managed-docs check with `status: "warn"`
- **AND** the remediation entry instructs the operator to run `dont doctor --fix` to inject the block

#### Scenario: canonical file drift produces a warn check

- **WHEN** `.dont/AGENTS.md` differs from the current generator output
- **AND** the caller runs `dont doctor --json`
- **THEN** the doctor envelope reports the managed-docs check with `status: "warn"`
- **AND** the remediation entry instructs the operator to run `dont doctor --fix`

#### Scenario: whitespace-only drift does not produce a warn

- **WHEN** the only difference between on-disk content and generator output is trailing whitespace or `\r\n` versus `\n` line endings
- **AND** the caller runs `dont doctor --json`
- **THEN** the managed-docs check reports `status: "pass"`

### Requirement: Managed-docs repair via `doctor --fix`

The system SHALL provide a `--fix` flag on `dont doctor` that rewrites the managed block in each configured root document and overwrites `.dont/AGENTS.md` with the current generator output. `--fix` MUST preserve content outside the managed block markers byte-for-byte in every configured root document. `--fix` MUST be idempotent with respect to detected project state: running `dont doctor --fix` twice in succession with no intervening change to the inputs the generator reads (configured `managed_docs` targets, `dont` version, installed rules, project mode) MUST produce identical files after the first run. Without `--fix`, `dont doctor` MUST remain read-only and MUST NOT touch any file.

#### Scenario: fix rewrites only the managed region

- **WHEN** the caller runs `dont doctor --fix` on a project whose root `AGENTS.md` has stale content between the managed markers
- **THEN** the command rewrites the bytes between the `<!-- DONT:START -->` and `<!-- DONT:END -->` markers with the current generator output
- **AND** all bytes outside those markers in that file are preserved exactly

#### Scenario: fix overwrites the canonical file

- **WHEN** the caller runs `dont doctor --fix` and `.dont/AGENTS.md` differs from the current generator output
- **THEN** the command overwrites `.dont/AGENTS.md` in whole with the current generator output

#### Scenario: fix is idempotent

- **WHEN** the caller runs `dont doctor --fix` and then immediately runs `dont doctor --fix` again with no intervening change to project state
- **THEN** the second invocation makes no further byte-level changes to any managed file

#### Scenario: doctor without fix is read-only

- **WHEN** the caller runs `dont doctor` or `dont doctor --json` without `--fix`
- **THEN** no file on disk is modified, regardless of how many managed-docs checks report `warn`

#### Scenario: fix produces output identical to init

- **WHEN** a project is freshly initialized by `dont init` and then `dont doctor --fix` is run with no other state changes
- **THEN** `dont doctor --fix` makes no byte-level changes to any managed file
- **AND** the managed-docs check subsequently reports `status: "pass"`

#### Scenario: fix injects the block when missing

- **WHEN** the caller runs `dont doctor --fix` on a project whose root `AGENTS.md` exists but does not contain the managed-block sentinels
- **THEN** the command injects the managed block into the file using the same placement rules `dont init` uses
- **AND** all preexisting content in the file is preserved exactly outside the inserted block
