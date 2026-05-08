# rule-claim-structure

**Severity:** warn (off by default)
**Configuration:** `[rules.rule_claim_structure]`

## What the rule checks

`rule-claim-structure` validates that claims tagged as rule claims carry the two mandatory slot markers required by the six-slot semantic schema defined in `dont-rule-claim-schema`.

A claim is a rule claim when its `depends_on` list includes the `term:uuid` of the `rule-claim-type` anchor term (set via `tag_term_id` in config). The rule ignores all other claims.

For each tagged claim, the rule checks for:

1. **`[TRIGGER]`** — mandatory. Identifies when the rule fires.
2. **`[CONFIG]` or `[MODE]`** (at least one) — mandatory. Covers enablement (`[CONFIG]`) and/or severity behavior across project modes (`[MODE]`).

The rule validates **marker presence only**. It does not evaluate the accuracy of slot content. A structurally complete claim with incorrect content passes this rule; content accuracy is a human responsibility enforced through the claim evidence and doubt mechanisms.

## What the rule does not check

- Optional slots (`[INVOCATION]`, `[GUARD]`, `[EVAL]`, `[BOUNDARY]`) — omitting these is valid; each has a documented default.
- Whether the claim's description of the rule's behavior is accurate.
- Claims not tagged with the `rule-claim-type` term.

## Violations

Each missing mandatory slot produces a separate warning entry on the envelope. A claim missing both `[TRIGGER]` and `[CONFIG]`/`[MODE]` produces two warnings.

Warn-severity violations do not change the claim's stored status. A verified claim that triggers a `rule-claim-structure` warning remains verified in the database.

## Configuration

```toml
[rules.rule_claim_structure]
enabled = true
tag_term_id = "term:<uuid-of-rule-claim-type>"
```

To find the correct UUID: run `dont show <curie>` on the term defined as `rule-claim-type`, or look at the `depends_on` of any existing rule claim.

## When to enable

Enable this rule in projects that have adopted the rule claim convention (i.e., projects with a `rule-claim-type` anchor term and multiple tagged rule claims). It is disabled by default because it is only useful to projects that use this convention.

## Remediation

For a claim flagged as missing `[TRIGGER]`: add a `[TRIGGER] Fires when: <condition>` line.
For a claim flagged as missing `[CONFIG]`/`[MODE]`: add at least one of:
- `[CONFIG] Enabled by default: yes | no`
- `[MODE] In permissive mode: warn | strict | same as strict | n/a`

Since `dont update` is not available, correct a flagged claim by doubting the old version (`dont trust <claim-id> --reason "missing mandatory slot"`) and creating a corrected claim with `dont conclude`.
