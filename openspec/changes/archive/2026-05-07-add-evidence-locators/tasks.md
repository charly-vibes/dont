## 1. Specify evidence-locator capability
- [x] 1.1 Add `dont-evidence-locators` requirements for repository-relative file locators, optional line spans, and anchors
- [x] 1.2 Define excerpt/fingerprint capture expectations for later audit and drift detection
- [x] 1.3 Define human/JSON projection expectations for structured evidence in inspection views

## 2. Integrate with existing payload and help surfaces
- [x] 2.1 Update payload contracts that expose evidence so structured locators are representable without ambiguity
- [x] 2.2 Update help/tutorial guidance to recommend structured repository evidence when grounding repo facts

## 3. Validate
- [x] 3.1 Run `openspec validate add-evidence-locators --strict`
