# rule-claim-structure

Human-readable specification for `rule_claim_structure.rs`.

**Severity:** warn (off by default)
**Configuration:** `[rules.rule_claim_structure]`

## What the rule checks

`rule-claim-structure` validates that claims tagged as rule claims carry the two mandatory slot markers required by the structured slot schema (see `dont help --howto rule-claims` for the full template and slot reference).

A claim is a rule claim when its `depends_on` list includes the `term:uuid` of the `rule-claim-type` anchor term (set via `tag_term_id` in config). The rule evaluates only non-Doubted claims — claims in Doubted status are skipped because they are already being remediated.

For each tagged, non-Doubted claim, the rule checks for:

1. **`[TRIGGER]`** — mandatory. Identifies when the rule fires.
2. **`[CONFIG]` or `[MODE]`** (at least one) — mandatory. Covers enablement (`[CONFIG]`) and/or severity behavior across project modes (`[MODE]`).

The rule validates **marker presence only** using a substring match. It does not evaluate the accuracy of slot content. A structurally complete claim with incorrect content passes this rule; content accuracy is a human responsibility enforced through the claim evidence and doubt mechanisms.

**Known false-negative edge case:** The substring match means a statement that contains a marker string in prose (e.g. "see [TRIGGER] for context") will not fire even though the marker is not being used as a slot. This is accepted behavior at the current scale.

## What the rule does not check

- Optional slots (`[INVOCATION]`, `[GUARD]`, `[EVAL]`, `[BOUNDARY]`) — omitting these is valid; each has a documented default.
- Whether the claim's description of the rule's behavior is accurate.
- Claims not tagged with the `rule-claim-type` term.
- Claims with status `Doubted`.

## Violations

Each missing mandatory slot produces a separate warning entry on the envelope. A claim missing both `[TRIGGER]` and `[CONFIG]`/`[MODE]` produces two warnings.

Warn-severity violations do not change the claim's stored status. A verified claim that triggers a `rule-claim-structure` warning remains verified in the database.

## Configuration

```toml
[rules.rule_claim_structure]
enabled = true
tag_term_id = "term:<uuid-of-rule-claim-type>"
```

To find the correct UUID: run `dont show <namespace-prefix>:rule-claim-type` (substituting your project's namespace prefix), or look at the `depends_on` of any existing rule claim.

**Silent misconfiguration:** If `enabled = true` but `tag_term_id` is unset, empty, or contains a wrong UUID, the rule produces no output with no error or warning. This is intentional — the rule cannot know what the correct term is. If you enable the rule and see no output where you expect violations, verify that `tag_term_id` matches the actual UUID of your `rule-claim-type` anchor term by inspecting an existing tagged claim with `dont show <claim-id>`.

## Marker syntax

Slot markers are **case-sensitive and whitespace-sensitive**. Use the exact uppercase form: `[TRIGGER]`, `[CONFIG]`, `[MODE]`. Lowercase variants (`[trigger]`) or markers with trailing spaces (`[TRIGGER ]`) will not be recognized.

## When to enable

Enable this rule in projects that have adopted the rule claim convention (i.e., projects with a `rule-claim-type` anchor term and multiple tagged rule claims). It is disabled by default because it is only useful to projects that use this convention.

## Remediation

For a claim flagged as missing `[TRIGGER]`: add a `[TRIGGER] Fires when: <condition>` line.
For a claim flagged as missing `[CONFIG]`/`[MODE]`: add at least one of:
- `[CONFIG] Enabled by default: yes | no`
- `[MODE] In permissive mode: warn | strict | same as strict | n/a`

Since `dont update` is not available, correct a flagged claim by doubting the old version and creating a corrected one:

1. `dont trust <claim-id> --reason "missing mandatory slot"` — marks the claim as Doubted. (`dont trust` with a `--reason` flag is the dont command for doubting a previously concluded claim.)
2. `dont conclude "..." --depends-on term:<uuid>` — create the corrected claim.

Once doubted, the old claim will be excluded from `rule-claim-structure` evaluation.
