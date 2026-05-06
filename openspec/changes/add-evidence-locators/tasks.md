## 1. Specify evidence-locator capability
- [ ] 1.1 Add `dont-evidence-locators` requirements for repository-relative file locators, optional line spans, and anchors
- [ ] 1.2 Define excerpt/fingerprint capture expectations for later audit and drift detection
- [ ] 1.3 Define human/JSON projection expectations for structured evidence in inspection views

## 2. Integrate with existing payload and help surfaces
- [ ] 2.1 Update payload contracts that expose evidence so structured locators are representable without ambiguity
- [ ] 2.2 Update help/tutorial guidance to recommend structured repository evidence when grounding repo facts

## 3. Validate
- [ ] 3.1 Run `openspec validate add-evidence-locators --strict`
