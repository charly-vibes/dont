# How to: Rule Claims

Rule claims are `dont` claims that describe the behavior of a lint rule using a
structured six-slot schema. The schema makes rule descriptions machine-checkable
by the `rule-claim-structure` rule.

## Canonical Template

```
[INVOCATION] <rule-name> runs as: background lint | opt-in via `dont check --<flag>`
[CONFIG]     Enabled by default: yes | no
[MODE]       In permissive mode: warn | strict | same as strict | n/a
[TRIGGER]    Fires when: <condition>
[GUARD]      Silently skips: <inputs>   (omit line if no guard)
[EVAL]       Evaluation model: stateless demand | event-driven on <event>   (omit if stateless demand)
[BOUNDARY]   Does not handle: <edge cases>; defers to <other-rule>   (omit if no boundary)
```

## Mandatory Slots

| Slot | Marker | Required |
|------|--------|----------|
| TRIGGER CONDITION | `[TRIGGER]` | Yes |
| CONFIG (enablement) | `[CONFIG]` | Yes — one of CONFIG or MODE |
| MODE (severity) | `[MODE]` | Yes — one of CONFIG or MODE |

`[CONFIG]` and `[MODE]` are distinct sub-markers. Either alone satisfies the
CONFIG/MODE requirement; both may appear in the same claim.

## Optional Slots and Their Defaults

| Slot | Marker | Default when omitted |
|------|--------|----------------------|
| INVOCATION MODEL | `[INVOCATION]` | Background lint, runs with `dont prime` |
| PRECONDITION GUARD | `[GUARD]` | Evaluates all inputs; no silent skip |
| EVALUATION MODEL | `[EVAL]` | Stateless demand-evaluated |
| BOUNDARY | `[BOUNDARY]` | No explicit boundary with sibling rules |

Omit optional slots when their default is correct. Include them when the default
would mislead a reader — for example, `[INVOCATION]` is required for opt-in rules
like `lockable` that do not run with `dont prime`.

## Tagging a Claim as a Rule Claim

Tag every rule-describing claim with the `rule-claim-type` term using its UUID:

```bash
dont conclude "..." --depends-on term:<uuid-of-rule-claim-type>
```

Do **not** use the bare string `rule-claim-type` or a CURIE — that triggers
`unresolved-terms`.

## Enabling the rule-claim-structure Validator

```toml
[rules.rule_claim_structure]
enabled = true
tag_term_id = "term:<uuid-of-rule-claim-type>"
```

Run `dont prime` after enabling to see any violations. The rule is warn-severity
and does not block operations. Disable it again after verifying by removing or
setting `enabled = false`.

## Correcting a Flagged Claim

Since `dont update` is not available, correct a flagged claim by:

1. `dont trust <claim-id> --reason "missing [TRIGGER] slot"` — doubt the old version
2. `dont conclude "..." --depends-on term:<uuid>` — create the corrected claim
