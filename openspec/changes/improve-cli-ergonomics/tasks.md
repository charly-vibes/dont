## 1. Ergonomic Verb Renaming
- [ ] 1.1 Update `dont-lifecycle-verbs` and `dont-status-lifecycle` to rename `trust` -> `doubt` and `dismiss` -> `verify`
- [ ] 1.2 Implement aliases in the CLI for backward compatibility during the transition period
- [ ] 1.3 Update help and tutorial documentation to reflect the new naming convention

## 2. Human-Readable Output Mode
- [ ] 2.1 Implement a default human-mode output formatter for all subcommands
- [ ] 2.2 Respect `--json` and `--plain` flags as per `dont-cli-surface` requirements
- [ ] 2.3 Ensure structured evidence (from `add-evidence-locators`) is rendered clearly in human-mode

## 3. Core Lifecycle Accessibility (Hypotheses & Atoms)
- [ ] 3.1 Add subcommands for `hypothesis` (add, assess) and `atom` (define, dismiss) management
- [ ] 3.2 Update `dont-payload-types` to ensure these new entities/attributes are properly exposed

## 4. Entity Lookup & ID Ergonomics
- [ ] 4.1 Implement a CURIE-to-ULID resolver for all command arguments
- [ ] 4.2 Support short-ULID prefixes (e.g., `claim:01KQZ`) in entity lookups
- [ ] 4.3 Implement stdin ID piping (`-`) for bulk operations

## 5. Universal Flag Compliance
- [ ] 5.1 Implement `--author <id>` and `$DONT_AUTHOR` environment variable support
- [ ] 5.2 Implement `--direct` flag to bypass harness detection
- [ ] 5.3 Implement `--version --json` for structured version envelopes

## 6. Validate
- [ ] 6.1 Run `openspec validate improve-cli-ergonomics --strict`
- [ ] 6.2 Verify that the "Locked" state is now reachable using only CLI commands
