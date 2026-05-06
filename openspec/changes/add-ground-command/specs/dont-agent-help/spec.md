## ADDED Requirements
### Requirement: Help positions ground as the fast path for documented repository facts
The system SHALL teach `dont ground` as the fast path for recording a documented repository fact when the operator already has both the claim text and its supporting evidence. This teaching MUST preserve the core four verbs as the canonical underlying model rather than implying that `ground` replaces them conceptually.

#### Scenario: tutorial presents ground as sidecar fast path
- **WHEN** the caller reads the first-session tutorial or a repository-grounding how-to
- **THEN** the material may recommend `dont ground` for quick documented-fact capture while still explaining that it composes the underlying `conclude` and `dismiss` semantics
