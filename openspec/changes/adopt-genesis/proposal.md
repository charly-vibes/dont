# Change: Adopt genesis

## Why

dont is the donor of genesis's `envelope` module (the `dont-2j6o` extraction)
and already ships a managed-block injector. This change makes dont a
*consumer* of genesis: it re-imports the envelope from `genesis::envelope`,
sources its managed-block injector mechanics from `genesis::managed_block`,
and adopts `genesis::suggestions` (dont has no self-healing `Suggestion` enum
today — see tool-craft.md Appendix A.3). This is the `dont-2j6o` supersession
landing in dont.

## What Changes

- Add `genesis` git dependency (pinned by tag `v0.1.0`) to `Cargo.toml`.
- Replace `src/envelope.rs` with a re-export of `genesis::envelope`; the
  deployed `dont-envelope` spec (envelope_version `0.2` semantics, `ok`,
  `envelope_kind`, `hints`, `warnings`) becomes the genesis module's contract.
- Source the `<!-- DONT:START/END -->` injector mechanics from
  `genesis::managed_block`; dont keeps its block *content* (the pointer to
  `.dont/AGENTS.md`, the auto-managed warning).
- Adopt `genesis::suggestions` for typo detection and fix-footers on dont's
  command surface (`conclude`/`define`/`trust`/`flag`/`ground`).
- Keep all domain logic in dont (cozo store, epistemic state machine, rule
  engine). The genesis boundary rule protects this.

## Impact

- Affected specs: `dont-envelope` (MODIFIED — sourced from genesis),
  `dont-agent-help` (MODIFIED — injector from genesis), `dont-cli-core`
  (MODIFIED — suggestions from genesis), `dont-errors` (remediation footer
  now via `genesis::suggestions`).
- Affected code: `Cargo.toml`, `src/envelope.rs`, `src/managed_block.rs`,
  the `main.rs` error sink (suggestion footer).
- Blocked by: genesis tagging `v0.1.0` (envelope + managed_block + suggestions stable).
- **Supersedes** `dont-2j6o` in dont: the envelope adoption moves here.
- No user-visible behavior change except `--json` envelopes are now identical
  in shape to wai/pretender/espectacular/testaruda.
