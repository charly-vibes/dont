## Context

Section 10 of `dont-spec-v0_3_2.md` covers derived commands, the output envelope, payload types, input schemas, error handling, and CLI conventions. This is too much for a single capability. The prior decomposition established core (purpose/invariants), lifecycle (status lattice), CLI core (four verbs), and operational (init/modes, lifecycle verbs). This change extracts the next coherent layer: the machine-parseable contract infrastructure.

## Goals
- Capture the envelope contract as a standalone capability that payload-type specs can reference
- Capture error handling (codes, remediation, exit codes) as its own capability so error-related changes don't force envelope-wide review
- Capture CLI shell conventions separately from envelope semantics — these are orthogonal (envelope is JSON structure; CLI surface is flags, colour, stdin, completions)

## Non-Goals
- Specify individual payload shapes (`ClaimView`, `TermView`, etc.) — deferred to data-model change
- Specify input schemas — deferred with payload types
- Specify derived commands (`guess`, `assume`, `overlook`) — separate orchestration layer

## Decisions
- **Three capabilities, not one or two**: Envelope versioning, error taxonomy, and CLI conventions change independently and have different consumers. Envelope is consumed by all JSON parsers; errors by harness retry logic; CLI surface by shell integrations.
- **Exit codes live with errors, not CLI surface**: Exit codes (§10.7.1) are the shell projection of the error taxonomy. A harness branches on exit code to decide "retry via remediation" vs "stop and check config" — this logic is error-centric, not shell-centric.
- **Payload type list in envelope, shapes deferred**: The `envelope_kind` discriminator values are listed in `dont-envelope` because they're part of the envelope contract. But the actual `data` shapes (`ClaimView`, etc.) are deferred to the data-model change.
- **Forward-compatibility rules are normative**: Parsers MUST have default branches for unknown `envelope_kind`, unknown error codes, and unknown rule `kind` values.

## Source Mapping
- `dont-envelope`: §10.2 (envelope shape, fields, versioning), §10.3 (identity and format conventions)
- `dont-errors`: §10.5 (error envelope, error codes, remediation invariant), §10.7.1 (exit codes)
- `dont-cli-surface`: §10.7.2 (universal flags), §10.7.3 (colour/terminal), §10.7.4 (stdin piping), §10.7.5 (completions), §10.7.6 (help surface)

## Risks / Trade-offs
- Error exit codes straddle the boundary between error taxonomy and CLI surface. Placing them with errors keeps the "what does exit 1 mean?" question in one place but means `dont-cli-surface` doesn't fully describe the shell contract alone.
  - Mitigation: `dont-cli-surface` references `dont-errors` for exit-code semantics.
- Some payload-type detail (like `applicable_rules` structure with `kind: "gate"` / `kind: "flag"`) is interleaved with envelope conventions in §10.4. This change defers those to the data-model capability.
  - Mitigation: envelope spec notes that `data` is typed by `envelope_kind` without specifying shapes.

## Open Questions
- Should `envelope_version` bumping rules (what constitutes a minor vs major change) be specified here or deferred to a versioning/migration capability?
