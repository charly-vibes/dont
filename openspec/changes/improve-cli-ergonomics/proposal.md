# Change: improve CLI ergonomics and core lifecycle accessibility

## Why

The current `dont` CLI implementation has several usability friction points that hinder adoption and violate existing specifications. Key issues include inverted verb semantics (`trust` meaning doubt), lack of human-readable output (everything is JSON), and the inability to reach the "Locked" state because commands for adding hypotheses and atoms are missing. Improving these ergonomics is critical for the tool to be usable by human operators and not just autonomous agents.

## What Changes
- Keep `trust` as the doubt verb — `dont trust` reads as "do not trust it", which correctly signals skepticism.
- Rename `dismiss` to `flag` — `dont flag` reads as "do not flag it as a concern", which correctly signals clearance. (`dont verify` and `dont doubt` were rejected because they invert the "dont = do not" phrase semantics.)
- Add `undoubt` as a correction verb for walking back an erroneous `trust` (doubted → unverified). `reopen` remains exclusive to ignored entities.
- Implement human-readable output as the default mode, moving JSON to the `--json` opt-in flag.
- Add subcommands or flags to support adding `hypotheses` and `atoms` to claims, enabling the path to `Locked` status.
- Support CURIE resolution in all entity-targeting commands (e.g., `dont show WB:P001` instead of requiring ULIDs).
- Implement short-ID support (unique ULID prefixes) for easier CLI interaction.
- Add the missing universal flags (`--author`, `--plain`, `--direct`) and stdin ID piping (`-`) as required by `dont-cli-surface`.

## Deferred
- Interactive TUI for hypothesis assessment.
- Automatic CURIE generation for new terms.
- Batch atom extraction from unstructured text.

## Change Type
- Usability strengthening and spec compliance.

## Related Changes
- Complements `add-ground-command` by providing the underlying ergonomic improvements for one-shot workflows.
- Complements `add-evidence-locators` by ensuring structured evidence is readable in the new human-mode output.

## Impact
- Affected specs: `dont-cli-surface`, `dont-lifecycle-verbs`, `dont-status-lifecycle`, `dont-payload-types`.
- Affected code: CLI parser, output emission layer, CURIE resolver, entity lookup logic.
- Affected workflow: Significantly lowers the barrier to entry for human operators and improves the transparency of agent actions.
