# Change: add structured evidence locators for repository grounding

## Why

`dont` is most compelling when an agent can ground a claim in concrete repository evidence rather than in chat memory. The current evidence shape is sufficient for coarse URIs, but too weak for high-trust repository workflows in projects like Chacana and XAct.jl. Operators need to point to a specific file, line span, anchor, or excerpt and later audit exactly what justified a claim.

## What Changes
- Add a capability for structured evidence locators aimed at repository grounding.
- Define repository-relative file locators with optional line spans and anchors.
- Define optional captured excerpts and stability fingerprints so later readers can see what text was relied on and detect drift.
- Define how structured evidence is surfaced in claim/term inspection payloads.
- Add operator-facing documentation for grounding claims from repository sources.

## Deferred
- Full semantic indexing of source files
- Automatic AST-aware code slicing
- Remote VCS blame/history integration
- Automatic re-verification on file drift

## Change Type
- New capability and usability strengthening, not merely an implementation repair

## Related Changes
- `add-ground-command` can build on these locators for its preferred repository-grounding path

## Impact
- Affected specs: `dont-evidence-locators`, `dont-payload-types`, `dont-agent-help`
- Affected code: dismiss input parsing, evidence storage/projection, help/tutorial output, drift-reporting surfaces
- Affected workflow: repository-grounded claims become more auditable and more useful as long-lived epistemic memory
