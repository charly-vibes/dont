# Ubiquitous Language

Core terminology for the `dont` project. All contributors — human and agent — should use these terms precisely and consistently across code, specs, docs, and issue descriptions.

## Bounded contexts

- [Claim lifecycle](./contexts/claim-lifecycle.md) — claims, statuses, transitions, and evidence

## Quick glossary

| Term | Definition |
|---|---|
| **claim** | An assertion about the codebase or project that requires grounding before it can be treated as trusted. Created by `dont conclude`. |
| **term** | A vocabulary entry with a canonical definition. Created by `dont define`. |
| **grounding** | The act of attaching sufficient evidence to a claim so it transitions from `unverified` to `verified`. |
| **evidence locator** | A repository-relative pointer to evidence: `--file <path> --lines N-M` or `--file <path> --anchor <heading>`. |
| **hypothesis** | A competing explanation recorded under a claim. Assessed as supporting or refuting. |
| **atom** | An independently checkable sub-condition of a composite claim. Must be dismissed individually before the parent claim can be locked. |
| **CURIE** | Compact URI — a prefixed identifier (`prefix:local`) used to reference terms in claim text. |
| **status** | The lifecycle state of a claim: `unverified`, `verified`, `doubted`, or `locked`. |
| **doubted** | A claim that has been explicitly questioned via `dont trust`. Doubted claims block `dont prime` and CI. |
| **locked** | A verified claim frozen from further modification. Requires ≥ 3 assessed hypotheses and ≥ 2 independent evidence sources. |
| **permissive mode** | Project-level enforcement where `unverified` claims produce warnings, not errors. Default on `dont init`. |
| **strict mode** | Project-level enforcement where all violations are errors. Set with `dont init --strict` or `dont config`. |
| **prime** | Session orientation and terminal gate: `dont prime` exits 1 if any claim is doubted. |
| **Envelope** | The JSON output format wrapping every `dont --json` response: `{ "ok": bool, "payload": ..., "error": ... }`. |
| **remediation** | The `remediation` field in a structured error — the next best command the agent should run. |
