# dont-agent-help Specification

## Purpose
TBD - created by archiving change add-dont-harness-specs. Update Purpose after archive.
## Requirements
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

### Requirement: Orientation prompt contract
The system SHALL provide a minimum-viable orientation prompt for LLM sessions. The orientation text MUST instruct the LLM to use `--json`, distinguish the core verbs from lifecycle verbs, explain permissive versus strict mode, require remediation-driven recovery on refusal, require harness fulfilment of spawn requests, recommend `dont suggest-term` before `define`, recommend supplying `--label "<a noun phrase>"` alongside `--doc` when coining terms (noting that the label is shape-checked and appears in diagrams), and point to `dont help --tutorial` for the full teaching walkthrough.

#### Scenario: refusal guidance in orientation block
- **WHEN** the reader consults the orientation block
- **THEN** it instructs them to read `data.remediation[0].command` and run it rather than guessing reformulations

#### Scenario: spawn guidance in orientation block
- **WHEN** the orientation block describes `spawn_request` envelopes
- **THEN** it tells the reader to invoke the harness subagent mechanism rather than performing the verification in the original session

#### Scenario: label coining guidance in orientation block
- **WHEN** the reader consults the orientation block
- **THEN** it explicitly recommends passing `--label '<a noun phrase>'` alongside `--doc` when coining terms
- **AND** it explains that the label is what appears in diagrams and is shape-checked
- **AND** the guidance appears between the `suggest-term` recommendation line and the `dont help --tutorial` pointer

#### Scenario: orientation points to deeper docs
- **WHEN** the orientation block reaches the end of its quick-start guidance
- **THEN** it points to `dont help <cmd>`, `.dont/AGENTS.md`, `dont help --tutorial`, and `dont help --howto <topic>` for more detail

### Requirement: Help and teaching entry points
The system SHALL provide `dont help` as the primary agent-addressed help surface. Bare `dont help` MUST list the available commands and major help entry points. `dont help <cmd>` and `<cmd> --help` MUST route to the same command-specific help content. `dont help --tutorial` MUST print the first-session tutorial, `dont help --howto <topic>` MUST print a goal-oriented how-to guide, and `dont help --topics` MUST list the available tutorial and how-to topics.

#### Scenario: bare help lists commands and entry points
- **WHEN** the caller runs `dont help`
- **THEN** the output lists the available commands and the major tutorial/how-to entry points

#### Scenario: subcommand help routing matches help verb
- **WHEN** the caller runs `dont lock --help`
- **THEN** the output matches the content of `dont help lock`

#### Scenario: tutorial help entry point
- **WHEN** the caller runs `dont help --tutorial`
- **THEN** the output is the sequenced first-session walkthrough rather than per-command reference text

#### Scenario: how-to topic selection
- **WHEN** the caller runs `dont help --howto harness-integration`
- **THEN** the output is the corresponding goal-oriented guide if that topic exists

#### Scenario: help topics listing
- **WHEN** the caller runs `dont help --topics`
- **THEN** the output lists the available tutorial and how-to entry points

### Requirement: Canonical teaching artifacts
The system SHALL ship normative teaching artifacts beyond command reference. It MUST include a worked example showing the canonical define → conclude → spawn → flag → lock flow, a first-session tutorial that explains why each step is taken, and goal-oriented how-to guides for project-specific rule authoring, harness integration, and `.dont/` store recovery.

#### Scenario: worked example teaches canonical flow
- **WHEN** the caller reads the worked example artifact
- **THEN** it shows a representative session beginning with `dont prime --json`
- **AND** it demonstrates spawn-based verification rather than self-verification

#### Scenario: tutorial emphasises orient-search-coin-conclude-spawn loop
- **WHEN** the caller reads the first-session tutorial
- **THEN** it presents the workflow as a sequenced learning path rather than as isolated command reference entries

#### Scenario: how-to corpus covers the three named operator goals
- **WHEN** the caller browses the how-to guides
- **THEN** the corpus includes guides for authoring a project-specific rule, integrating `dont` into a new harness, and recovering a corrupted `.dont/` store

### Requirement: Help teaches structured repository evidence as the preferred grounding mode
The system SHALL teach structured repository evidence as the preferred mode for grounding repository facts. Operator-facing help and tutorials MUST recommend repository-relative file locators over opaque absolute `file://` paths when the evidence source is inside the current project, while still noting that URI-only evidence remains supported for compatibility.

#### Scenario: tutorial recommends repository-relative evidence
- **WHEN** the caller reads the grounding-oriented tutorial or how-to material
- **THEN** the examples prefer repository-relative evidence locators over absolute `file://` paths for project files

#### Scenario: compatibility path remains documented
- **WHEN** the help text describes repository evidence locators
- **THEN** it still notes that plain URI-only evidence remains supported as a compatibility path

### Requirement: Help positions ground as the fast path for documented repository facts
The system SHALL teach `dont ground` as the fast path for recording a documented repository fact when the operator already has both the claim text and its supporting evidence. This teaching MUST preserve the core four verbs as the canonical underlying model rather than implying that `ground` replaces them conceptually.

#### Scenario: tutorial presents ground as sidecar fast path
- **WHEN** the caller reads the first-session tutorial or a repository-grounding how-to
- **THEN** the material may recommend `dont ground` for quick documented-fact capture while still explaining that it composes the underlying `conclude` and `flag` semantics

### Requirement: Help recommends trace when blocker labels are not enough
Operator-facing help and tutorials SHALL recommend `dont trace <entity-id>` as the next diagnostic step when a claim or term is blocked by dependency/support fallout and `show` or `why` alone do not explain the causal path clearly.

#### Scenario: blocked verification guidance points to trace
- **WHEN** the tutorial or a refusal-oriented how-to explains what to do after seeing a blocker such as `stale` or `unresolved-term`
- **THEN** it recommends `dont trace <entity-id>` as the path-oriented diagnostic command

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
