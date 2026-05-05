## 1. Extract envelope contract capability
- [x] 1.1 Write `dont-envelope` spec with envelope shape, versioning rules (major vs minor semantics), field semantics, and forward-compatibility rules
- [x] 1.2 Include identity and format conventions (ID prefixes, naming, timestamps)
- [x] 1.3 Establish `dont-envelope` as the exclusive normative owner of the canonical `envelope_kind` discriminator values

## 2. Extract error taxonomy capability
- [x] 2.1 Write `dont-errors` spec with `ErrorResult` shape and globally unique string literal error codes
- [x] 2.2 Include strict remediation invariant (non-empty actionable `remediation[]` on every handled error)
- [x] 2.3 Include complete v0.3.2 error-code set with scope boundaries
- [x] 2.4 Include exit-code contract distinguishing epistemic/domain errors (`1`) from systemic/infrastructure failures (`2`)

## 3. Extract CLI surface capability
- [x] 3.1 Write `dont-cli-surface` spec with universal `--json` support across all commands
- [x] 3.2 Include auto-stripping of ANSI colours on non-TTY and explicit `dont completions` generation
- [x] 3.3 Include strict stdin prose consumption for `conclude`/`define` and completely silent stdout/stderr when `--json` is active
- [x] 3.4 Explicitly exclude automatic terminal paging (e.g. `less`) from the help surface

## 4. Validate
- [x] 4.1 Run `openspec validate add-dont-envelope-specs --strict`
