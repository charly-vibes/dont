## 1. Specify analytics capability

- [x] 1.1 Review `dont-analytics` spec for completeness and scenario coverage
- [x] 1.2 Confirm `StatsView` payload fields align with `dont-envelope` envelope-kind conventions

## 2. Specify ephemeral mode capability

- [x] 2.1 Review `dont-ephemeral-mode` spec for edge cases (read-only commands, environment variable)
- [x] 2.2 Confirm `ephemeral` field placement does not conflict with existing `dont-envelope` fields

## 3. Specify eval-export capability

- [x] 3.1 Review `dont-eval-export` spec for payload completeness
- [x] 3.2 Confirm `EvalExport` scope flags are consistent with `dont-analytics` scope flags

## 4. Validate

- [x] 4.1 Run `openspec validate add-eval-readiness --strict`
- [x] 4.2 Resolve any validation issues reported by `--strict`

## 5. Implementation (post-approval)

- [x] 5.1 Implement `dont stats` command and `StatsView` payload (depends on: 1.x)
- [x] 5.2 Implement caught-contradiction retrospective join query (depends on: 5.1)
- [x] 5.3 Implement `--no-persist` universal flag and `DONT_NO_PERSIST` env var (depends on: 2.x)
- [x] 5.4 Implement `dont export --eval` command and `EvalExport` payload (depends on: 3.x)
- [x] 5.5 Add integration tests covering each scenario from all three specs (depends on: 5.1–5.4)
- [x] 5.6 Update `.dont/AGENTS.md` managed block to document `dont stats`, `dont export --eval`, and `--no-persist` (depends on: 5.1–5.4)
- [x] 5.7 Add time-budget guard and `truncated` marker for large-store performance cases in `dont stats` and `dont export --eval` (depends on: 5.1, 5.4)
