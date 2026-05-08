## Phase 1 — Convention (no code changes)

- [x] 1.1 [AFK] Define `rule-claim-type` term in the project with `dont define rule-claim-type`. Note the resulting `term:uuid` — all subsequent steps use this ID, not the bare string or a CURIE.
  - Result: `term:01KR4TNRGHVPRZQ1Z95GFZN4ZQ`
- [x] 1.2 [HITL] Add rule claim authoring section to `.dont/AGENTS.md` including canonical template, slot reference table, CONFIG vs MODE sub-marker guidance, the validate-before-create note ("fill and review the template before running `dont conclude`; there is no pre-creation dry-run"), and guidance on when optional slots become load-bearing.
- [x] 1.3 [AFK] Re-tag the 7 existing rule claims by adding the `term:uuid` from step 1.1 to each claim's `depends_on` via `dont update <claim-id> --dep <term:uuid>`. Do not use the string `rule-claim-type` or a CURIE directly — that would trigger `unresolved-terms`.
  - Note: `dont update` does not exist; tasks 1.3 and 1.4 were merged — all 7 claims were re-created with slot markers + `--depends-on term:01KR4TNRGHVPRZQ1Z95GFZN4ZQ`. Old verified claims doubted with supersession reason.
- [x] 1.4 [HITL] Verify each of the 7 rule claims covers both mandatory slots (TRIGGER and at least one of CONFIG/MODE). Rewrite any that do not using the canonical `[SLOT]` marker template from step 1.2, not free-form prose — Phase 2 validation depends on marker presence. Doubt the old version before creating the corrected claim.
  - New claim IDs: dangling-definition `claim:01KR4V3FVQRYQBBREH637N6CN7`, correlated-error `claim:01KR4V3HKMFTRZJJCC2QDGSE4S`, ungrounded `claim:01KR4V3KP8ETNMA6KRJEJVR8AK`, term-nonfunctional-label `claim:01KR4V3PQF5KDNF8K94BW1YT7M`, stale-cascade `claim:01KR4V3SDH7P2EW9RDQN9RBMVF`, lockable `claim:01KR4V3WG3XM7GQFN6JV8S9SMF`, unresolved-terms `claim:01KR4V3YRAS28PDGCTGCNRFA7F`

## Phase 2 — Structural lint rule (code changes)

*Phase 2 readiness criterion: implement when ≥15 rule claims exist or when manual template compliance during PR review becomes recurring friction. With only 7 claims, the convention is sufficient.*

- [ ] 2.1 [AFK] Write failing test: a claim tagged with the `rule-claim-type` term:uuid and missing `[TRIGGER]` should produce a `rule-claim-structure` warning when the rule is enabled; a claim with both mandatory slots filled should not. A claim without the tag should produce no warning regardless of content.
- [ ] 2.2 [AFK] Create `src/rules/rule_claim_structure.rs` implementing the marker-presence check; register in the rule catalogue as off-by-default warn severity.
- [ ] 2.3 [HITL] Create sibling translation document `src/rules/rule_claim_structure.md` explaining what the rule checks and how to satisfy it. Include the constraint: the rule validates marker presence only — it does not evaluate the accuracy of slot content.
- [ ] 2.4 [AFK] Add `rule-claim-structure` to the config schema under `dont-project-config` so it can be enabled with `rules.rule-claim-structure.enabled = true`.
- [ ] 2.5 [AFK] Add `rule-claim-structure` warning code to the error taxonomy in `dont-errors` (follow `term-nonfunctional-label` as the template).
- [ ] 2.6 [AFK] Enable `rule-claim-structure` in project config, run `dont prime`, and verify 0 warnings appear for the 7 tagged rule claims. Disable again after verification (rule is off by default).
- [ ] 2.7 [AFK] Update `dont help --howto rule-claims` to include the canonical template (accessible via the help system).
- [ ] 2.8 [AFK] Archive this change once Phase 2 ships.
