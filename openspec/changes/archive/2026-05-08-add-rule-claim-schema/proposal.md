# Change: Add rule claim semantic schema and structural lint rule

## Why

During dogfooding, 3 of 7 rule-describing claims had to be retracted and rewritten because they encoded wrong or misleading facts about rule behavior. The root cause was not carelessness — it was that no shared mental model existed for *what a well-formed rule claim must cover*. Each author filled different slots and omitted others, producing claims that were accurate on one axis but dangerously incomplete on another.

The Ro5 review surfaced 6 recurring semantic slots that, when all addressed, yield claims that are both correct and useful as living documentation. This change formalises those slots as a named schema, publishes a template convention, and adds a structural lint rule so the schema can be machine-checked.

## What Changes

- **New capability `dont-rule-claim-schema`**: defines the 6-slot semantic schema for rule-describing claims, the mandatory/optional status of each slot, and the template convention authors must follow
- **MODIFIED `dont-rule-engine`**: adds `rule-claim-structure` to the shipped rule catalogue — an off-by-default warn-severity rule that validates claims tagged as rule claims against the schema
- **Phase split**: Phase 1 ships the convention (AGENTS.md template, no code); Phase 2 ships the rule

## Slot Reference

| Slot | Symbol | Mandatory | Default when omitted |
|---|---|---|---|
| INVOCATION MODEL | `[INVOCATION]` | No | background lint, runs with `dont prime` |
| TRIGGER CONDITION | `[TRIGGER]` | **Yes** | — |
| PRECONDITION GUARD | `[GUARD]` | No | evaluates all inputs; no silent skip |
| EVALUATION MODEL | `[EVAL]` | No | stateless demand-evaluated |
| CONFIG (enablement) | `[CONFIG]` | **Yes — one of CONFIG or MODE** | — |
| MODE (severity behavior) | `[MODE]` | **Yes — one of CONFIG or MODE** | — |
| BOUNDARY | `[BOUNDARY]` | No | no explicit boundary with sibling rules |

Mandatory: TRIGGER and at least one of CONFIG or MODE. These are two distinct sub-slots: `[CONFIG]` covers whether the rule is enabled by default (e.g. `Enabled by default: no`); `[MODE]` covers how the rule behaves across project modes (e.g. `In permissive mode: warn`). Both may appear in the same claim — write `[CONFIG]` when the rule's default-enabled status is non-obvious, and `[MODE]` when severity differs across modes. Either alone satisfies the mandatory slot requirement. Their combined absence was the proximate cause in all 3 retracted claims.

## Impact

- Affected specs: `dont-rule-claim-schema` (new), `dont-rule-engine` (add `rule-claim-structure` to catalogue)
- Affected code: Phase 2 only — `src/rules/rule_claim_structure.rs` (new file), sibling translation doc, config registration
- No breaking changes; `rule-claim-structure` is off by default
