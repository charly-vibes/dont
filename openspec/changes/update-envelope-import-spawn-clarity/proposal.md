# Change: tighten envelope, import, and spawn edge-case contracts

## Why

Quality diagnostics identified several high-impact ambiguities that would produce parser drift, inconsistent importer behaviour, and non-deterministic spawn handling under race conditions.

The highest-risk gaps are:
- contradictory `hints` semantics in success envelopes
- ambiguous `meta.tx` numeric constraints
- underspecified import idempotence identity
- missing import network-destination safety policy (SSRF/private-network fetch)
- unspecified timeout-vs-callback race handling for spawn requests

## What Changes

- Modify `dont-envelope` to make `hints` presence and `meta.tx` constraints unambiguous
- Modify `dont-import-surface` to define importer-specific canonical source identity for idempotence
- Add a network destination safety requirement for HTTP-backed imports
- Modify and extend `dont-spawn-protocol` to define deterministic resolution of timeout/callback races

## Impact

- Affected specs: `dont-envelope`, `dont-import-surface`, `dont-spawn-protocol`
- Affected code (future): envelope builders/parsers, import adapters, spawn sweeper/callback handling
- Breaking change: no (tightens normative behaviour and clarifies edge semantics within v0.2 envelope family)
