# Executive Summary: Why Agents Don't Use `dont`

**Across 80 sessions and 4,655 tool calls in the `dont` project itself, the core `dont` lifecycle (conclude → ground → flag → lock) was used exactly zero times. All 6 pre-investigation `dont` shell commands were `dont init` (project setup, 2×) and `dont list` (CI diagnostics, 4×). Agents are building the tool but never using it.**

## Definitive Truth Table

| Metric | Value | Notes |
|--------|-------|-------|
| Total sessions | 80 | Full corpus, dont project |
| Total tool calls (bash/read/edit/write) | 4,655 | Across all 80 sessions |
| **Actual dont shell commands** | **13** | Pre-this-investigation: **6** |
| Sessions with any dont command | 3 of 80 (3.8%) | Pre-investigation: 2 of 79 (2.5%) |
| Sessions using core lifecycle | 0 | **Zero ever** |
| | | |
| **dont conclude** | 0 | Claim creation — never used |
| **dont ground** | 0 | Fast path — never used |
| **dont flag** | 0 | Evidence — never used |
| **dont dismiss** | 0 | Evidence (alias) — never used |
| **dont lock** | 0 | Lock verified — never used |
| | | |
| dont init | 2 | Project setup (July 7, July 10) |
| dont list | 7 | Querying (4× pre-investigation, 3× this session) |
| dont prime | 1 | Orientation (this session) |
| dont help/explain/stats | 3 | Documentation (this session) |
| | | |
| dont commands / total tool calls | 0.28% | = 13/4655 |
| Core lifecycle / total tool calls | 0.00% | = 0/4655 |
| Pre-investigation core lifecycle / total | 0.00% | = 0/4655 |

## Methodology Note: False-Positive Filtering Cascade

The initial analysis using naive string matching (`"dont "` in any bash argument) reported 515 "dont invocations" across 29 sessions — a **97% false-positive rate** caused by:

1. **Naive "dont " match → 515** — Matched `dont-` prefixed issue IDs (dont-71y, dont-cfki), spec paths (`openspec/changes/add-core-dont-specs`), heredoc content (writing about the tool), `bd show dont-*` commands, `git commit` messages, `rg` searches
2. **Exclude grep/rg/search commands → ~400** — Removed `rg 'dont'`, `grep -r dont`, etc.
3. **Exclude heredocs (<< markers) → ~200** — Removed commands writing markdown/text that mentions dont
4. **Exclude wai add / git commit / cat / echo → ~50** — Removed commands that write about dont but don't invoke it
5. **Strict regex: dont as shell command only → 13** — Only matches `dont <subcommand>` at line start or after `&&`, `;`, `||`, `|`, `` ` ``

## Baseline Comparison

| Tool | Est. invocations | Sessions using | Session ritual? |
|------|-----------------|----------------|-----------------|
| bd | ~1,850 | ~90% | `bd prime` / `bd ready` |
| wai | ~1,180 | ~100% | `wai sync` / `status` / `close` |
| openspec | ~368 | ~50% | `openspec list` |
| **dont (core lifecycle)** | **0** | **0%** | **None** |
| **dont (any command)** | **13** | **3.8%** | **None** |

## Root Causes

1. **No session ritual** — `dont prime` is never part of the start-of-session sequence. Compare: `wai sync`/`wai status`/`wai close` is a forced lifecycle; `bd prime`/`bd ready` is a forced ritual. `dont` has neither.

2. **Weak or absent AGENTS.md instructions** — Projects with `.dont` say "also uses dont for evidence-grounded claims" (one line, buried) or nothing at all. Compare: `wai` gets a full WAI block in every project.

3. **No forced gate** — Permissive mode allows unverified claims indefinitely. CI never catches missing claims. The `dont check` and `dont prime` exit codes never trigger.

4. **Tool UX friction** — The core workflow requires multiple commands (`conclude` → `define` → `flag` → `lock`) with CURIE/term:uuid syntax. No agent has ever tried it.

5. **dont-grill skill pack exists but is undiscoverable** — The structured interview protocol for agents exists but nothing tells agents to invoke it.

## Why This Matters

`dont` exists to prevent ungrounded assertions. The tool that was built to solve this problem was itself built with **zero grounded assertions** during development. Every design decision, every correctness claim, every architectural assumption made during 80 sessions of development — none were tracked. The database contains only test claims ("The sky is blue") and rule-description claims added during CI/feature work.

## Fix

Three-phase approach tracked in epic `dont-m2vy`:
- **Phase 1**: Fix the investigation report itself (this document)
- **Phase 2**: Close evidence gaps (agent meta-cognition, external project audit, CI gate, UX friction)
- **Phase 3**: Implement mitigations (session ritual, stronger instructions, discoverable skill pack)
## Baseline Comparison Table (Dont Project)

| Tool | Total invocations | Sessions using | % sessions | Calls/session | Session ritual |
|------|------------------|----------------|------------|--------------|----------------|
| wai | 365 | 39 | 49% | 4.6 | wai sync/status/close |
| bd | 310 | 32 | 40% | 3.9 | bd prime / bd ready |
| git | 377 | 36 | 45% | 4.7 | — |
| rg | 222 | 36 | 45% | 2.8 | — |
| cargo | 253 | 23 | 29% | 3.2 | — |
| openspec | 121 | 25 | 31% | 1.5 | openspec list |
| just | 70 | 25 | 31% | 0.9 | just recipes |
| **dont** | **16** | **3** | **4%** | **0.2** | **None** |

**Verdict:** dont usage is **19× to 23× below** other workflow tools. wai and bd are used in ~40-50% of sessions; dont in 4%. The gap is not "agents don't use any tools" — they use wai, bd, openspec routinely. They specifically don't use `dont`.

## CI Gate Analysis

| Aspect | Status | Details |
|--------|--------|---------|
| CI workflow runs `just check-claims`? | ✓ Yes | Via `just ci` → `just check-claims` → `dont prime` |
| Lefthook pre-commit runs it? | ✓ Yes | `dont-prime` hook in lefthook.yml |
| `dont prime` ever exits 1? | ✗ No | Permissive mode: exit 0 unless claims are doubted |
| Doubted claims in database? | ✗ 0 | Never doubted |
| Effective gate? | ✗ **No** | Passes every time, catches nothing |

**Root cause:** The project is in permissive mode, which allows unverified claims indefinitely. The gate only fires on doubted claims — but no one has ever doubted a claim. The pre-push hook (`just check-claims`) runs every time but never fails.

**Fix:** Either (a) switch to strict mode so unverified claims block CI, or (b) make `dont prime` warn on stale unverified claims (no activity for 30+ days), or (c) keep permissive mode but add a `dont check` gate that enforces a minimum verification rate over a rolling window.

## Agent Meta-Cognition Analysis

Sampled 10 heavy-work sessions (597 thinking blocks) from the dont project. Key finding:

| Metric | Value |
|--------|-------|
| Thinking blocks sampled | 597 |
| Design/correctness assertions found | 7 |
| Claim-worthy assertions (falsifiable + consequential) | 0 |
| Agent explicitly considered logging/recording | 2 |
| **Missed-opportunity rate** | **~1 per session, none claim-worthy** |

### Interpretation

**Agents don't make the kind of strong, falsifiable claims in their thinking that `dont` is designed to track.** Their thinking is predominantly:
- **Exploration** (reading code, understanding structure)
- **Planning** (deciding what to do next, sequencing steps)
- **Debugging** (interpreting error output, trying alternative commands)
- **Reflection** (summarizing what they've done, noting open questions)

The "assertions" found were exploratory ("I think the issue might be related to X") rather than definitive ("The system guarantees X because..."). None were truly claim-worthy by `dont`'s criteria.

### Implications for Root Causes

- **Root cause A (no session ritual)** — Supported. Agents never think "I should run `dont prime` because their frame of reference doesn't include it.
- **Root cause B (weak instructions)** — Supported. Agents never consider `dont conclude` because nothing in their instructions tells them to.
- **Root cause D (Tool UX friction)** — Partially refuted. The barrier isn't just friction — it's that agents don't form the kind of assertions `dont` expects in their thinking. The real assertions are in the **code edits** (writing `edit`/`write` calls), not in thinking blocks.
- **New insight:** The real missed-opportunity is in the **edit/write tool calls**, not in thinking blocks. An agent writing `edit` on a critical correctness fix is making an assertion about the code being correct — but that assertion is implicit in the code change, not explicit in their thinking. `dont` would need to intercept at the edit/write level, not the thinking level.

### Recommendation

The `dont-grill` skill pack (which interviews agents before making claims) is the right approach. The interview protocol would surface the implicit assertions that agents make in their code edits but never articulate in thinking blocks.

## UX Friction Analysis

Tested the complete `dont` workflow from an agent's perspective in a fresh project.

### Workflow Steps

| Step | Command | Result | Time | Friction |
|------|---------|--------|------|----------|
| 1 | `dont init` | ✅ | 0.01s | None |
| 2 | `dont prime` | ✅ | 0.01s | None |
| 3 | `dont conclude "..."` | ✅ | 0.01s | None |
| 4 | `dont list --json` | ✅ | 0.01s | None |
| 5 | `dont flag <id> --evidence <uri>` | ✅ | 0.01s | None |
| 6 | `dont lock <id>` | **❌** | 0.00s | **CRITICAL: "unknown command 'lock'"** |
| 7 | `dont ground "..." --file <path>` | **❌** | — | **HIGH: file must exist** |
| 8 | `dont ground "..." --depends-on <id>` | **❌** | — | **HIGH: --depends-on not supported** |

### Friction Points Ranked

1. **[CRITICAL] `dont lock` is broken** — The CLI help lists `lock` as a command, it's documented as the "canonical lifecycle verb" in AGENTS.md, but running `dont lock <id>` returns "unknown command 'lock'. Did you mean 'dont forget'?" The `forget` alias works, but every documentation page says `lock`.

2. **[HIGH] `dont ground` requires existing files** — The `--file` argument insists the file exists on disk. For the "write-and-ground" workflow (ground a claim while editing a file), you must save the file first, then ground. One extra step, but the error message is clear.

3. **[HIGH] `dont ground` doesn't support `--depends-on`** — You cannot ground a claim with term dependencies in one step. Must use `conclude` → `define` → `flag` (3+ commands) instead of `ground` (1 command). This breaks the "fast path" promise.

4. **[MEDIUM] `dont ground` help is misleading** — `--file` and `--url` are described as optional but at least one must be provided. The help says `-e, --evidence` as an option but it's not clear how it interacts with `--file`.

5. **[LOW] `dont lock` vs `dont forget` naming confusion** — The help says `lock` is the canonical verb and `forget` is the legacy alias, but only `forget` actually works. The canonical lifecycle in AGENTS.md says `conclude → trust → dismiss → forget` — but `dismiss` is an alias for `flag`, and `lock` doesn't work.

### Implications

The core workflow (conclude → flag) works fine. The friction is at the **lock** step (broken) and the **ground** fast path (no --depends-on, requires existing files). These are all fixable, but they explain why an agent trying to follow the lifecycle would get confused and give up.

## External Project Audit: espectacular

**Project:** charly-vibes/espectacular — 18 sessions, `.dont` initialized, strict mode

### Current State

| Metric | Value |
|--------|-------|
| Sessions | 18 |
| Sessions with `dont` command | 1 (May 12: `dont init` + `dont prime`) |
| Claims in database | 0 |
| Terms in database | 0 |
| Project mode | `strict` |
| `.dont/AGENTS.md` quality | **Good** — thorough quick start, workflow, grounding examples |
| AGENTS.md mention | "This project **also uses** dont for evidence-grounded claims" (one line, buried) |

### Why It's Not Used

The `.dont/AGENTS.md` is actually better than the `dont` project's own — it has clear examples, explains `dont ground` as the fast path, and shows the full lifecycle. But the **root AGENTS.md** only says "also uses" — it's a single line, not a section. Compare:

- **bd**: "This project uses **bd** (beads) for issue tracking. Run `bd prime` for full workflow context." (line 73)
- **dont**: "This project also uses **dont** for evidence-grounded claims." (line 75 — immediately after bd, no call to action)

The word "also" is the problem. It signals that `dont` is secondary. No agent has ever been told to run `dont prime` at session start, or to `dont conclude` when making a design assertion.

### Root Cause Confirmed

The `espectacular` audit confirms root cause B (weak instructions) as the dominant factor. The `.dont/AGENTS.md` is good, but agents never get there because the root AGENTS.md doesn't make them.

### Contrast with pretender

pretender has `.dont` initialized but **zero mentions** of `dont` in AGENTS.md or CLAUDE.md. Same pattern: 16 sessions, 1 session with `dont init` + `dont prime`, zero claims ever.
