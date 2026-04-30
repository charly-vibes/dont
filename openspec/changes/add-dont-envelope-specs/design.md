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
- **Versioning rules defined here**: The envelope spec explicitly defines what constitutes a major vs. minor version bump (e.g. changing field types is major, adding optional fields is minor).
- **Exclusive canonical discriminator list**: `dont-envelope` owns the canonical list of `envelope_kind` discriminators (e.g. `"claim"`, `"error"`). Payload specs merely reference them.
- **Exit codes live with errors, not CLI surface**: Exit codes (§10.7.1) are the shell projection of the error taxonomy. A harness branches on exit code to decide "retry via remediation" (`1` - epistemic error) vs "stop and check config" (`2` - systemic error). This logic is error-centric, not shell-centric.
- **Remediation invariant**: Every handled error envelope MUST contain at least one actionable recovery string in its `remediation` array.
- **String literal error codes**: Error codes MUST be globally unique string literals (e.g. `term-label-empty`), not HTTP-style numeric codes.
- **Universal silent JSON**: Every single CLI command (including `init` and `doctor`) MUST support `--json`. When active, the single JSON object is the *only* output emitted to stdout, and stderr MUST be completely silent.
- **Native Completions**: The binary MUST natively generate shell completions (e.g., `dont completions <shell>`).
- **Auto-Color Stripping**: ANSI escape sequences MUST be stripped automatically when stdout is not a TTY, but can be forced via env vars (`CLICOLOR_FORCE=1`).
- **Stdin Prose Consumption**: If `--doc` or `--statement` is omitted, the commands `conclude` and `define` MUST consume standard input (if not a TTY) as the prose body.
- **Forward-compatibility rules are normative**: Parsers MUST have default branches for unknown `envelope_kind`, unknown error codes, and unknown rule `kind` values.
- **No Automatic Paging**: The CLI MUST NOT automatically pipe help or explanations to a pager (like `less`).

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
- None remain for the high-level boundary of these three capabilities; boundaries and semantics resolved by design interview.
