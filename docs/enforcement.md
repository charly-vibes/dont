# Enforcement model

`dont` is designed so that there is no `--no-verify` equivalent — no flag that
lets a caller skip the verification gates. This page explains why, what the
actual gates are, and how to wire them into your toolchain.

## Why there is no bypass flag

Git's `--no-verify` works by skipping hook scripts — side-car processes that
live outside the binary. Because they're external, they can be bypassed.

`dont`'s gates live *inside* the binary:

- State transitions are exhaustive `match` arms in Rust. The code for a
  forbidden transition (`Doubted → Locked`) simply doesn't exist.
- The dependency integrity check runs in-process before any verification is
  accepted. There is no code path around it.
- `dont prime` is designed to be the terminal check — it exits 1 on doubted
  claims, regardless of project mode.

An agent or user can always choose not to call `dont`, but they cannot call it
while bypassing its rules.

## In-binary gates

### 1. State machine (`src/model.rs`)

All transitions are exhaustive matches. The only path to `Locked` is
`Unverified → Verified → Locked`. `Doubted → Locked` has no match arm.

### 2. Always-strict rules (`src/rules/mod.rs`)

Three rules are hardcoded to `Strict` severity before any config lookup:

| Rule | What it catches |
|------|----------------|
| `unresolved-terms` | dependency reference (CURIE or `term:uuid`) that can't be resolved |
| `stale-cascade` | claim that depends on an unverified or doubted claim |
| `dangling-definition` | explicit `term:uuid` ID reference to a term that no longer exists |

These cannot be downgraded to warnings via project config.

### 3. Dependency integrity gate

Before a claim can be verified with `dont flag` (read: "don't flag this as a
concern"), `dependency_gate_unmet_clauses()`
walks all dependencies. If any are `Unverified` or `Doubted`, the command
emits a structured refusal and exits 1.

### 4. Hedge filter

`dont trust` and `dont ignore` reject assertions that contain hedge language
(`"I think"`, `"maybe"`, `"probably"`, etc.). The default list is compiled in;
config can only extend it, never replace it.

### 5. `dont prime` — orientation and terminal gate

`dont prime` has two roles:

**Orientation** — run it at the start of a session to see what's doubted or unverified. It always prints the project status summary (mode, counts by status, and any blocking entities). When the store is clean it exits 0.

**Terminal gate** — run it at the end (CI, pre-commit, agent stop hook) to confirm nothing was left unresolved. It exits 1 if any claim is `Doubted`, regardless of project mode.

Projects initialize in permissive mode by default (pass `--strict` to `dont init` to start in strict mode). Unverified claims are allowed in permissive mode (where warnings replace errors for most rules); doubted claims are always blocking regardless of mode.

### 6. Lockability rule

To lock a verified claim you need ≥ 3 assessed hypotheses and ≥ 2 independent
evidence sources. This gate is enforced in-code inside the `dont lock` handler
and cannot be bypassed — `dont lock` exits 1 if the gate is unmet, regardless
of project mode or rule-engine config. (The `lockable` rule engine severity
defaults to `Warn` so it does not block non-lock operations in the background;
the hard enforcement is the in-command gate, not the background rule engine.)

## Hook infrastructure

The same `dont prime` check runs at three points in the development cycle.

### Git (prek pre-commit)

`prek.toml` includes a local hook that runs `just check-claims` (which runs
`dont prime`) before every commit. (prek is a pre-commit hook runner — install
it and run `prek install` to activate hooks.)

```toml
[[repos]]
repo = "local"
hooks = [
  { id = "dont-prime", name = "dont: block doubted claims",
    entry = "just check-claims", language = "system",
    pass_filenames = false, always_run = true },
]
```

If any claim is `Doubted`, the commit is rejected. Run `prek install` after
editing `prek.toml` to activate the hook.

### Claude Code / agentic tools

Add a `Stop` hook to `.claude/settings.local.json` so the agent surfaces
doubted claims at the end of every session:

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "just check-claims" }]
      }
    ]
  }
}
```

When `dont prime` exits 1, Claude Code displays the output as an error — a
signal that the session ended with unresolved doubt. When claims are clean it
exits 0 (output is still produced but not treated as an error by Claude Code).

### CI

`just ci` calls `just check-claims`, which calls `dont prime`. A doubted claim
fails the pipeline. See `.github/workflows/ci.yml`.

## The fundamental limit

This infrastructure makes it harder to accidentally skip the check — it does
not make skipping impossible. An agent that never calls `dont` is invisible to
these gates. The discipline works only when agents register claims as they make
them (see the [grounding workflow](./grounding-workflow.md)).
