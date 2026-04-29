# Change: add --label flag and verb-level shape validators to `dont define`

## Why

A review of Spivak & Kent's "Ologs: A Categorical Framework for Knowledge
Representation" (arXiv:1102.1889v2, hereafter SK11) against the v0.3.2 spec
revealed a discipline gap in `dont define`. SK11 §2.1 specifies that a type
must be written as a singular indefinite noun phrase, and §2.1.1 provides a
five-item rules-of-good-practice checklist. Three of those five items are
mechanically checkable from a string alone — the same deterministic,
offline, non-overridable pattern already applied to `trust --reason` via the
`reason-required` and `reason-not-hedge` validators. The remaining two items
(recognisability, documentable instances) are semantic and stay at the rule or
assume-time layer.

Without `--label`, `dont define` accepts any `--doc` prose without shape
checks. Harnesses that coin terms with sentence-shaped or punctuated labels
produce diagrams that cannot be composed with SK11 aspect sentences (§2.2.2)
and pre-empt future olog tooling. The fix follows the exact pattern established
on `trust`: promote the mechanically-checkable subset to verb-level validators,
add the supporting config knobs, and introduce one off-by-default rule for the
heuristically-detectable remainder.

`envelope_version` stays `"0.2"` — the patch is additive on the verb signature
and error-code table only.

## What Changes

- **`dont-cli-core`**: `define` gains a strictly required `--label "<noun phrase>"` flag for all new coined terms.
  Five verb-level validators (strict pass/fail refusals) fire on it before any rule evaluation: empty label, indefinite shape, punctuated shape, sentence shape, and compound-undeclared. No "magic extraction" from `--doc` is performed.
- **`dont-errors`**: Five new refusal codes (`term-label-empty`, `term-shape-indefinite`, `term-shape-punctuated`, `term-compound-undeclared`, `term-label-sentence`) are added to the known-code set. Soft warnings are strictly reserved for the rule engine.
- **`dont-rule-engine`**: One new rule (`term-nonfunctional-label`) is added to the shipped catalogue with an absolute minimum severity of `warn`. It cannot be turned off completely.
- **`dont-project-config`**: Two new config blocks: `[define.shape]` for toggling the verb-level validators and extending compound-structure markers; `[rules.term_nonfunctional]` for configuring the heuristic rule patterns.
- **`dont-data-model`**: Term-specific attributes gain a `label` field (the SK11 type-text value), which is strictly required for all local coined terms, remaining optional only for imported/seed terms.
- **`dont-agent-help`**: Orientation prompt contract gains explicit guidance requiring `--label` alongside `--doc` when coining terms.

## Deferred

- Aspects as a sixth primitive (SK11 §2.2, §2.2.3) — deliberately excluded;
  the `5-primitive budget` constraint in `dont-data-model` is unchanged.
- Commutative-diagram facts as a claim sub-kind (SK11 §2.3).
- Functorial alignment between project ologs and imported ontologies
  (SK11 §4.2-4.3).
- Full calibration of the `term-label-sentence` heuristic — expected after
  early adoption data is available. Calibration candidates to be captured via
  the dont-rule-engine §17 backlog item once adoption data is available.

## Traceability

Sourced from `dont-spec-v0_3_3-patch.md` (post-olog-review patch document):
- Layer-1 (verb signature) → `dont-cli-core` delta
- Layer-2 (verb-level validators §9.2.1) → `dont-cli-core` delta
- Layer-3 (error codes §10.5) → `dont-errors` delta
- Layer-4 (rule layer §13) → `dont-rule-engine` delta
- Layer-5 (config §14) → `dont-project-config` delta
- Layer-6 (documentation §11.1) → `dont-agent-help` delta
- Seed YAML shape (§7) → `dont-data-model` delta

SK11 sections:
- §2.1 *Types* — singular indefinite noun phrase convention
- §2.1.1 *Rules of good practice* — five-item checklist; items (i), (iv), (v)
  become verb-level validators; items (ii), (iii) remain semantic/deferred
- §2.2.1 *Rules of good practice* for aspects — rationale for `term-label-sentence`
- §2.2.3 *Converting non-functional relationships to aspects* — rationale for
  the off-by-default `term-nonfunctional-label` rule

## Impact

- Affected capabilities: `dont-cli-core`, `dont-errors`, `dont-rule-engine`,
  `dont-project-config`, `dont-data-model`, `dont-agent-help`
- Existing `--doc`-only invocations for new coined terms will fail, as `--label` is strictly required. Seed/imported installations are immune.
- Soft heuristic warnings are delegated solely to the rule engine, preserving the absolute boundary between verb-level syntax validators and graph-layer trace evaluation.
