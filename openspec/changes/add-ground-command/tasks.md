## 1. Specify the one-shot grounding command
- [ ] 1.1 Add `dont-ground-command` requirements for statement-plus-evidence one-shot claim capture
- [ ] 1.2 Define explicit input scope, duplicate handling, and event-sequence semantics so `ground` composes existing lifecycle rules instead of bypassing them
- [ ] 1.3 Define atomic single-invocation failure semantics for unsuccessful grounding attempts

## 2. Integrate with existing command surfaces
- [ ] 2.1 Update CLI/help expectations so `ground` appears as a standard subcommand and remains outside stdin-ID bulk mode
- [ ] 2.2 Add input-schema expectations for `ground` in payload/schema contracts

## 3. Teach the workflow
- [x] 3.1 Update tutorial/how-to expectations so `ground` is documented as the fast path for repository-fact capture
- [x] 3.2 Preserve the core four verbs as the canonical underlying model in docs and help

## 4. Validate
- [x] 4.1 Run `openspec validate add-ground-command --strict`
