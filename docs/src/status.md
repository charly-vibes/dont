# Status

**beta** — core works and is dogfooded in anger; surface may shift before stable.

## Implemented

| Command | Status | Notes |
|---|---|---|
| `dont check` | stable | epistemic gate — fail on ungrounded claims |
| `dont conclude` | stable | record grounded conclusions |
| `dont define` | stable | define claim vocabulary/terms |
| `dont flag` | stable | flag claims pending grounding |
| `dont trust` | beta | trust boundaries for sources |

## In progress

- Standardization rollout (docs restructure, dogfood matrix) — DDL-u8x epic.

## Mapped to specs

- `openspec/` proposals in-repo; epistemic gate ADR (dont-bpuo) enforced in vampiro's CI too.

## Dogfooding

- **vampiro** runs `dont check` as its epistemic gate in CI (vampiro-bf6)
- **testaruda** pins `dont-cli@0.2.2`
- dont's own CI runs `just ci` (prek, typos, vale, wai)
