---
tags: [pipeline-run:tdd-ro5-2026-06-17-add-managed-agent-skills, pipeline-step:plan]
---

## Feature: add-managed-agent-skills

### What is being added

`dont` gains the ability to install and maintain first-party agent skill packs under `.agents/skills/`. The first pack is `dont-grill`: a router skill + 9 sub-skills (conclude, define, flag, trust, lock, ignore, trace, scenarios, conclude-worthiness) that turn claim discipline into a structured interview protocol for agents.

Config surface: `[harness].managed_skill_packs = ["dont-grill"]` in `.dont/config.toml`. Default is empty list (no packs installed).

Writers are `dont init` and `dont doctor --fix` — same pair used for managed docs. `dont doctor --json` adds a `managed_skills` check reporting `pass`, `stale`, or `missing` per pack.

Boundary: `dont` only rewrites packs it owns. Unmanaged sibling skills in `.agents/skills/` are preserved byte-for-byte.

### Architecture / existing pattern to follow

`project.rs` has parallel `refresh_managed_docs()` / `managed_docs_status()` methods backed by `managed_block.rs` primitives. The new `refresh_managed_skill_packs()` / `managed_skill_packs_status()` methods follow the same shape:
- staleness = SHA-256 of all files in pack dir (sorted by relative path, concatenated) vs SHA-256 of what generator would produce
- `file_matches()` / `write_canonical()` from `managed_block.rs` apply per-file

`main.rs` `Command::Doctor` adds a `managed_skills` check in the checks vec after calling the new project methods. `Command::Init` calls `refresh_managed_skill_packs()` after `refresh_managed_docs()`.

`config.rs` `HarnessConfig` gets `managed_skill_packs: Vec<String>` with default `[]`.

### New module: `skill_pack.rs`

Holds:
- `generate_dont_grill_pack() -> BTreeMap<RelativePath, String>` — deterministic template renderer; returns each file's path (relative to `.agents/skills/dont-grill/`) mapped to its content
- `pack_content_hash(files: &BTreeMap<..., String>) -> String` — SHA-256 of sorted-by-path concatenated content
- `disk_content_hash(dir: &Path) -> io::Result<String>` — same hash from on-disk files

### dont-grill pack structure

```
.agents/skills/dont-grill/
  dont-grill.md          ← router (auto-loadable)
  subs/
    conclude.md
    define.md
    flag.md
    trust.md
    lock.md
    ignore.md
    trace.md
    scenarios.md
    conclude-worthiness.md
```

Sub-skills live under `subs/` so harnesses scanning `.agents/skills/` only surface the top-level `dont-grill.md` router.

### Test strategy

**Unit tests (skill_pack.rs)**
- `generate_dont_grill_pack_is_deterministic`: call twice, assert equal output
- `generate_dont_grill_pack_contains_all_files`: assert 10 files present (router + 9 sub-skills)
- `generate_dont_grill_pack_router_uses_canonical_verbs`: router content contains `dont flag` and `dont lock`, not `dont dismiss` / `dont forget`
- `generate_dont_grill_pack_subs_in_subdirectory`: all sub-skill paths start with `subs/`
- `pack_content_hash_changes_with_content`: mutate one file, assert hash differs
- `disk_content_hash_matches_generated_hash`: write pack to tmpdir, assert `disk_content_hash == pack_content_hash`

**Integration tests (tests/managed_skills_integration.rs)**
- `init_installs_managed_skill_pack_when_configured`: init project with `managed_skill_packs = ["dont-grill"]`; assert `.agents/skills/dont-grill/dont-grill.md` exists
- `init_no_pack_when_not_configured`: init with empty `managed_skill_packs`; assert `.agents/skills/` absent or empty
- `doctor_reports_pass_when_pack_matches_generator`: install pack, run doctor --json; assert managed_skills check status = "pass"
- `doctor_reports_stale_when_pack_modified`: install pack, mutate one file, run doctor; assert "stale"
- `doctor_reports_missing_when_pack_absent`: configure pack, delete dir, run doctor; assert "missing"
- `doctor_fix_repairs_stale_pack`: mutate file, run doctor --fix; assert file content matches generator
- `doctor_fix_preserves_unmanaged_sibling`: write custom skill file, run doctor --fix; assert file unchanged

**Affected existing tests**: doctor tests that assert check count/names (if any hardcode the checks vec length — search before writing).

### Input/output shapes

`dont doctor --json` payload gains:
```json
{"name": "managed_skills", "status": "pass|stale|missing", "detail": "..."}
```
Per-pack detail message on stale: `"managed pack dont-grill is stale; run dont doctor --fix"`
Per-pack detail message on missing: `"managed pack dont-grill is missing; run dont doctor --fix"`

No new CLI subcommand. No changes to any existing envelope kind.
