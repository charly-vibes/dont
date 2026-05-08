# Design: Rule Claim Semantic Schema

## Context

`dont` ships 7 rules. During dogfooding, rule behavior was documented as claims in the `dont` database — using `dont` itself to enforce epistemic discipline on `dont`. Three of those 7 claims had to be retracted after a Ro5 review found factual errors.

The errors mapped to a common failure mode: claim authors covered *some* behavioral dimensions of a rule but not others, producing statements that were locally accurate but globally misleading. For example, a claim about `lockable` correctly stated the threshold requirements but never mentioned that the rule is opt-in — the most operationally important constraint.

Mapping the 7 corrected claims against their original failures reveals 6 recurring dimensions ("slots") that rule-describing claims reliably need to address:

1. **INVOCATION MODEL** — the lockable claim was wrong here (stated thresholds as unconditional requirements; omitted opt-in nature)
2. **TRIGGER CONDITION** — the ungrounded claim was incomplete here (missed malformed-dep path)
3. **PRECONDITION GUARD** — the correlated-error claim was incomplete here (missed ≥2 evidence threshold)
4. **EVALUATION MODEL** — the stale-cascade claim was wrong here (stated event-driven; correct is demand-evaluated)
5. **MODE/CONFIG** — the term-nonfunctional-label claim was wrong here (foregrounded severity, buried disabled-by-default)
6. **BOUNDARY** — the dangling-definition claim was misleading here (implied it doesn't check active claims; it does)

## Goals / Non-Goals

**Goals:**
- Define a shared schema with named slots that rule claim authors can use as a checklist
- Publish the schema as a template in `.dont/AGENTS.md` and `dont help --howto rule-claims`
- Add a shipped rule (`rule-claim-structure`) that validates claims against the schema automatically, so violations surface without human review

**Non-Goals:**
- Enforcing the schema on non-rule claims — the schema is specific to the rule-describing claim type
- Making the schema mandatory for *all* claims — too broad; the 6 slots are domain-specific to rules
- Structural validation at `conclude` time — this is a lint rule, not a verb-level validator; it runs with `dont prime`, not at claim creation

## Decisions

**Decision: Two mandatory slots (TRIGGER + MODE/CONFIG), four optional with explicit defaults**

Rationale: Every rule claim error traced to one of TRIGGER or MODE/CONFIG being missing or wrong. The other four slots are only load-bearing for specific rule kinds. Requiring all 6 would produce verbose claims for simple rules, degrading signal-to-noise.

MODE/CONFIG is treated as one mandatory slot with two distinct sub-markers:

- `[CONFIG]` — covers *enablement*: is the rule on by default? Write this when the rule is off-by-default (e.g., `term-nonfunctional-label`, `rule-claim-structure`) or context-dependent. Omit when the rule is always on.
- `[MODE]` — covers *severity behavior across project modes*: does the rule warn in permissive and block in strict, or is it always the same? Write this when severity differs across modes or when the rule is unconditionally warn-only.

Both may appear in the same claim — `term-nonfunctional-label` warrants both: `[CONFIG] Enabled by default: no` and `[MODE] In permissive mode: warn`. Either sub-marker alone satisfies the mandatory slot; neither is a violation. Their combined absence is the violation.

INVOCATION MODEL defaults to "background lint, runs with `dont prime`" when omitted — a correct default for 5 of 7 rules. Only `lockable` deviates, so authors of non-lockable rule claims don't need to state it.

EVALUATION MODEL defaults to "stateless demand-evaluated" when omitted. stale-cascade was the only rule where this was non-obvious.

PRECONDITION GUARD defaults to "evaluates all inputs" when omitted. Only correlated-error had a meaningful threshold guard.

BOUNDARY defaults to "no explicit sibling boundary" when omitted. The ungrounded↔dangling-definition boundary is the clearest case, but most rules don't have a sibling relationship that changes operational expectations.

**Decision: Rule claim tagging via term dependency rather than text prefix**

To enable machine validation, rule claims must be distinguished from other claims. The approach is to define a `rule-claim-type` term with `dont define`, which produces a `term:uuid` ID. Rule claims list this `term:uuid` in their `depends_on` entries. This uses existing `dont` machinery (depends_on, term resolution) rather than requiring new claim fields.

Important: the `depends_on` entry must be the `term:uuid` produced by `dont define rule-claim-type`, **not** the bare string `rule-claim-type` and not a CURIE like `local:rule-claim-type`. Using a bare string or an unregistered CURIE would immediately trigger `unresolved-terms`. Task 1.1 captures the term creation step; task 1.3 uses the resulting ID.

The term name `rule-claim-type` was chosen to match the pattern of existing `dont` term names. It reads as "a term that classifies this claim as a rule-describing claim" — not as an entity-type definition. If this naming proves confusing, `rule-description-tag` is the alternative, but renaming is a breaking change after claims depend on it.

Alternative considered: text convention only (claims start with "The `<rule-name>` rule..."). Rejected because brittle to reformulation and requires regex heuristics rather than structural query.

**Decision: Off by default, warn severity**

The rule is in the same category as `term-nonfunctional-label` — useful for teams adopting the convention, noise for teams that don't. Default-off lets projects opt in.

**Decision: Phase 1 = convention only, Phase 2 = code**

The schema is useful immediately as a process constraint. Waiting for the lint rule to be implemented before documenting the schema would delay the epistemic benefit.

## Slot Marker Language

The template uses bracket markers that are both human-readable and machine-matchable without NLP:

```
[INVOCATION] <rule-name> runs as: background lint | opt-in via `dont check --<flag>`
[CONFIG]     Enabled by default: yes | no
[MODE]       In permissive mode: warn | strict | same as strict | n/a
[TRIGGER]    Fires when: <condition>
[GUARD]      Silently skips: <inputs> (omit if no guard)
[EVAL]       Evaluation model: stateless demand | event-driven on <event> (omit if stateless demand)
[BOUNDARY]   Does not handle: <edge cases>; defers to <other-rule> (omit if no boundary)
```

The rule validates by checking for `[TRIGGER]` and `[MODE]` or `[CONFIG]` markers in the claim text. Future strictness can be added by config flag once the convention is established.

## Risks / Trade-offs

- **Slot marker grammar may get stale**: as rules evolve, their slot content changes. The claim still needs to be doubted and re-created. The schema doesn't solve drift — it only makes incompleteness detectable.
- **Term tagging is manual ceremony**: authors must remember to add the `rule-claim-type` term:uuid to depends_on. The lint rule only fires if claims are tagged. Under-tagging means the rule is silent on unchecked claims. Mitigated by template convention and AGENTS.md guidance.
- **Marker matching is lexical, not semantic**: `[TRIGGER] Fires when: ...` is a string check. A well-formed marker with incorrect content passes. `dont prime` showing 0 `rule-claim-structure` warnings means claims are structurally complete, not that they are accurate. Content accuracy remains a human responsibility enforced through the claim evidence and doubt mechanisms.
- **Stale-cascade amplification on `rule-claim-type` term**: all tagged rule claims list the `rule-claim-type` term in `depends_on`. If that term is ever doubted, `stale-cascade` will surface stale assessment for all tagged rule claims simultaneously — potentially 7+ warnings in `dont prime` from a single term doubt. This is correct behavior but may be surprising. Mitigation: treat `rule-claim-type` as a stable, rarely-changed anchor term; document in AGENTS.md that doubting it has wide effect.

## Open Questions

- Should `rule-claim-structure` also validate that at least one evidence entry pointing to the rule's source file exists? This would overlap with the `ungrounded` rule but would add a rule-specific evidence requirement. Deferred — keep the new rule's scope minimal.
- When the `rule-claim-type` term is defined, should it be part of the seed vocabulary installed at `dont init`? Probably yes, but only if rule claims are a first-class pattern in the standard workflow. Out of scope for this change.
