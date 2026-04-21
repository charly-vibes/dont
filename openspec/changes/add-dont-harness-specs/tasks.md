## 1. Extract derived query capability
- [ ] 1.1 Write `dont-derived-queries` spec for `list` / `vocab` scope and `--as-of` filtering
- [ ] 1.2 Include `show` and `why` query semantics and history/rule context
- [ ] 1.3 Include `prime`, `doctor`, `schema`, and `examples` command contracts

## 2. Extract agent-help capability
- [ ] 2.1 Write `dont-agent-help` spec for the managed docs block and `.dont/AGENTS.md` primacy
- [ ] 2.2 Include the orientation block contract and refusal-handling guidance
- [ ] 2.3 Include help/tutorial/how-to entry points and `sync-docs` rewrite behaviour

## 3. Extract spawn protocol capability
- [ ] 3.1 Write `dont-spawn-protocol` spec for `guess` / `assume` / `overlook` harness-mode behaviour
- [ ] 3.2 Include harness-mode detection order and direct-mode override
- [ ] 3.3 Include timeout expiry, `dont spawns` filters, and late-callback handling

## 4. Validate
- [ ] 4.1 Run `openspec validate add-dont-harness-specs --strict`
