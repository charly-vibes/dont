## 1. Extract envelope contract capability
- [ ] 1.1 Write `dont-envelope` spec with envelope shape, versioning, field semantics, and forward-compatibility rules
- [ ] 1.2 Include identity and format conventions (ID prefixes, naming, timestamps)
- [ ] 1.3 Include the canonical `envelope_kind` discriminator values

## 2. Extract error taxonomy capability
- [ ] 2.1 Write `dont-errors` spec with `ErrorResult` shape and field semantics
- [ ] 2.2 Include remediation invariant (non-empty `remediation[]` on every error)
- [ ] 2.3 Include complete v0.3.2 error-code set with scope boundaries
- [ ] 2.4 Include exit-code contract with harness-decision semantics

## 3. Extract CLI surface capability
- [ ] 3.1 Write `dont-cli-surface` spec with universal flags
- [ ] 3.2 Include colour/terminal awareness and stdin piping conventions
- [ ] 3.3 Include shell completion and help surface requirements

## 4. Validate
- [ ] 4.1 Run `openspec validate add-dont-envelope-specs --strict`
