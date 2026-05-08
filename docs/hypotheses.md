# Competing hypotheses and atoms

## When you have more than one explanation

The most valuable use of `dont` is when you're mid-investigation with two or more competing explanations for something you don't yet understand.

| Use | When |
|---|---|
| `hypothesis` | You have competing explanations — only one will survive |
| `atom` | You have one explanation with multiple sub-conditions — all must hold |

`dont hypothesis` lets you record those competing explanations under a single claim and assess each one as evidence comes in. This prevents the common failure: you go looking for evidence of hypothesis A, find something that looks plausible, and declare it confirmed — without ever seriously checking hypothesis B.

## Hypotheses: step by step

**1. State the claim (the thing you're investigating):**

```bash
dont conclude "API calls to the pricing endpoint are timing out intermittently"
# → claim-abc
```

**2. Add competing hypotheses:**

You can add hypotheses at any point before locking — before or after gathering initial evidence.

```bash
dont hypothesis add claim-abc --text "The upstream rate limit is being hit on burst traffic"
# → hypothesis-001

dont hypothesis add claim-abc --text "A database lock contention slows queries beyond the client timeout"
# → hypothesis-002

dont hypothesis add claim-abc --text "TLS handshake overhead on cold connections is the bottleneck"
# → hypothesis-003
```

**3. Assess each with evidence:**

Both `dont flag` calls and `--supporting` assessments count as independent evidence sources toward the lockability threshold.

```bash
# Found rate limit logs — supports hypothesis-001
dont hypothesis assess claim-abc hypothesis-001 --supporting logs/api-2026-05-07.log --lines 144-151

# Checked DB profiler — no long-running queries during timeout windows
dont hypothesis assess claim-abc hypothesis-002 --refuting reports/db-profile.txt

# TLS session reuse is enabled — refuted
dont hypothesis assess claim-abc hypothesis-003 --refuting src/client.rs --lines 88-94
```

**4. Flag the surviving hypothesis as evidence for the claim:**

```bash
dont flag claim-abc --file logs/api-2026-05-07.log --lines 144-151
```

**5. Lock when mature** (requires ≥ 3 assessed hypotheses + ≥ 2 independent evidence sources):

```bash
dont lock claim-abc
```

## Atoms: when a claim has multiple independently checkable parts

Sometimes a single claim is composite — it holds if and only if several sub-conditions hold. `dont atom` lets you decompose it.

**Example:** "The API client correctly handles all three authentication scenarios" might require:
- evidence that bearer token auth succeeds
- evidence that expired token returns 401 and triggers refresh
- evidence that missing credentials return 403 without retry

```bash
dont conclude "The API client correctly handles all three authentication scenarios"
# → claim-xyz

dont atom define claim-xyz --text "Bearer token auth returns 200 on valid credentials"
# → atom-001

dont atom define claim-xyz --text "Expired token triggers refresh and retries the request"
# → atom-002

dont atom define claim-xyz --text "Missing credentials return 403 without retry"
# → atom-003
```

Dismiss each atom as you verify it:

```bash
dont atom dismiss claim-xyz atom-001 --file tests/auth_test.rs --lines 12-28
dont atom dismiss claim-xyz atom-002 --file tests/auth_test.rs --lines 31-55
dont atom dismiss claim-xyz atom-003 --file tests/auth_test.rs --lines 58-74
```
