## 1. Extract derived query capability
- [x] 1.1 Write `dont-derived-queries` spec for `list` / `vocab` scope and `--as-of` filtering
- [x] 1.2 Include `show` and `why` query semantics and history/rule context
- [x] 1.3 Include `prime`, `doctor`, `schema`, and `examples` command contracts

## 2. Extract agent-help capability
- [x] 2.1 Write `dont-agent-help` spec for the managed docs block and `.dont/AGENTS.md` primacy
- [x] 2.2 Include the orientation block contract and refusal-handling guidance
- [x] 2.3 Include help/tutorial/how-to entry points and `sync-docs` rewrite behaviour

## 3. Extract spawn protocol capability
- [x] 3.1 Write `dont-spawn-protocol` spec for `guess` / `assume` / `overlook` harness-mode behaviour
- [x] 3.2 Include harness-mode detection order and direct-mode override
- [x] 3.3 Include timeout expiry, `dont spawns` filters, and late-callback handling

## 4. Extract MCP transport capability
- [x] 4.1 Write `dont-mcp-interface` spec for optional `dont mcp` stdio server mode
- [x] 4.2 Include the contract that MCP tool results return the same envelope JSON as the CLI
- [x] 4.3 State that MCP is optional and not the privileged integration surface

## 5. Validate
- [x] 5.1 Run `openspec validate add-dont-harness-specs --strict`
