---
tags: [pipeline-run:tdd-ro5-2026-07-08-dont-qau6-allow-prose-punctuation-in-dont-conclude-statement-text, pipeline-step:plan]
---

F2: Allow prose punctuation in dont conclude statement text

## Problem
validate_claim_statement rejects semicolons (;) in statement text as shell
metacharacters. But semicolons are common English prose punctuation. The
text already arrives as a single quoted argument — no injection risk.

## Root cause
SHELL_META constant in validate_claim_statement (src/main.rs:3371) includes
';' alongside genuinely dangerous chars (|, `, $, \, <, >, NUL).

## Fix
Remove ; from SHELL_META. The remaining chars (|, `, $, \, <, >, NUL)
are still rejected — these are actual injection vectors.

Note: : and / are already allowed per existing tests (ground_accepts_prose_statement_with_slash).

## Test strategy
- Update ground_rejects_statement_with_shell_metacharacter: remove foo;bar
  from the rejection list, verify it succeeds instead
- Update conclude_rejects_statement_with_shell_metacharacter: same change
- Add new test ground_accepts_prose_punctuation: verify ; : / in prose pass
- Add new test conclude_accepts_prose_punctuation: same

## Existing tests affected
- Both metacharacter tests need foo;bar removed from rejection list
- ground_accepts_prose_statement_with_slash already tests / — no change needed
