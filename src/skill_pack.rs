use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

pub type PackFiles = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackState {
    Pass,
    Stale,
    Missing,
}

impl PackState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Stale => "stale",
            Self::Missing => "missing",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackHealth {
    pub name: String,
    pub state: PackState,
    pub detail: String,
}

/// Generate all files for a named first-party managed skill pack.
/// Returns an error if `pack_name` is not a recognized first-party pack.
pub fn generate_pack(pack_name: &str) -> Result<PackFiles, String> {
    match pack_name {
        "dont-grill" => Ok(generate_dont_grill()),
        other => Err(format!("unknown managed skill pack: {other:?}")),
    }
}

/// SHA-256 of all files sorted by relative path, concatenated (path + "\0" + content).
pub fn pack_content_hash(files: &PackFiles) -> String {
    let mut hasher = Sha256::new();
    for (path, content) in files {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(content.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// SHA-256 of the on-disk files under `dir`, using the same algorithm as `pack_content_hash`.
/// Returns `Ok(hash)` or an I/O error.
pub fn disk_content_hash(dir: &Path) -> io::Result<String> {
    let files = read_dir_files(dir)?;
    Ok(pack_content_hash(&files))
}

fn read_dir_files(dir: &Path) -> io::Result<PackFiles> {
    let mut result = BTreeMap::new();
    collect_files(dir, dir, &mut result)?;
    Ok(result)
}

fn collect_files(base: &Path, current: &Path, out: &mut PackFiles) -> io::Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(base)
                .map_err(io::Error::other)?
                .to_string_lossy()
                .to_string();
            let content = std::fs::read_to_string(&path)?;
            out.insert(rel, content);
        }
    }
    Ok(())
}

fn generate_dont_grill() -> PackFiles {
    let mut files = BTreeMap::new();

    files.insert("dont-grill.md".to_string(), router_skill());
    files.insert("subs/conclude.md".to_string(), conclude_skill());
    files.insert("subs/define.md".to_string(), define_skill());
    files.insert("subs/flag.md".to_string(), flag_skill());
    files.insert("subs/trust.md".to_string(), trust_skill());
    files.insert("subs/lock.md".to_string(), lock_skill());
    files.insert("subs/ignore.md".to_string(), ignore_skill());
    files.insert("subs/trace.md".to_string(), trace_skill());
    files.insert("subs/scenarios.md".to_string(), scenarios_skill());
    files.insert(
        "subs/conclude-worthiness.md".to_string(),
        conclude_worthiness_skill(),
    );

    files
}

fn router_skill() -> String {
    r#"---
description: dont-grill — structured claim discipline interview for dont projects. Routes to sub-skills for each claim lifecycle step.
disable-model-invocation: false
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->
<!-- Re-generate: dont doctor --fix -->

# dont-grill

A structured interview protocol that enforces `dont` claim discipline before each assertion.

## When to invoke

Invoke this skill before any `dont conclude` to verify the claim is ready. The router inspects
the project state and delegates to the appropriate sub-skill.

## Routing

| Intent | Sub-skill |
|--------|-----------|
| Introduce a new claim | `/dont-grill/conclude` |
| Define a term | `/dont-grill/define` |
| Verify a claim with evidence | `/dont-grill/flag` |
| Register doubt about a claim | `/dont-grill/trust` |
| Lock a verified claim | `/dont-grill/lock` |
| Set aside an out-of-scope claim | `/dont-grill/ignore` |
| Trace claim history | `/dont-grill/trace` |
| Enumerate scenarios | `/dont-grill/scenarios` |
| Assess claim worthiness | `/dont-grill/conclude-worthiness` |

## Core verbs (canonical)

- `dont conclude "<text>"` — introduce a claim
- `dont flag <id> --evidence <uri>` — verify with evidence (do not flag it as a concern)
- `dont trust <id> --reason "..."` — register doubt (do not trust it)
- `dont lock <id>` — lock a verified claim
- `dont ignore <id>` — set aside out-of-scope
- `dont undoubt <id>` — retract doubt

## Protocol

1. Inspect existing claims: `dont list --json`
2. Ask one clarifying question at a time
3. Always supply a recommended answer
4. Crystallise output as a `dont` command, not a prose summary
"#.to_string()
}

fn conclude_skill() -> String {
    r#"---
description: dont-grill/conclude — guide the agent through introducing a new claim
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/conclude

Guides the agent through the pre-flight checklist before running `dont conclude`.

## Protocol

1. Inspect current claims: `dont list --json`
2. Check for near-duplicate: would this claim deduplicate against an existing one?
3. One question at a time — recommended answer always supplied
4. Output: `dont conclude "<final-text>"` with any required `--depends-on` flags
"#
    .to_string()
}

fn define_skill() -> String {
    r#"---
description: dont-grill/define — guide the agent through defining a new term
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/define

Guides the agent through the pre-flight checklist before running `dont define`.

## Protocol

1. Check existing vocabulary: `dont list --json` (filter terms)
2. Verify the label is a singular indefinite noun phrase
3. Output: `dont define <namespace>:<Label> --doc "<definition>"`
"#
    .to_string()
}

fn flag_skill() -> String {
    r#"---
description: dont-grill/flag — guide the agent through verifying a claim with evidence
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/flag

Guides the agent through providing evidence to clear a claim.

`dont flag` = "do not flag it as a concern" — you are clearing the claim, not flagging it as problematic.

## Protocol

1. Identify the target claim: `dont show <id> --json`
2. Confirm evidence URI is accessible and specific
3. Output: `dont flag <id> --evidence <uri>`
"#.to_string()
}

fn trust_skill() -> String {
    r#"---
description: dont-grill/trust — guide the agent through registering doubt about a claim
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/trust

Guides the agent through registering doubt.

`dont trust` = "do not trust it" — you are expressing distrust, not endorsing the claim.

## Protocol

1. Identify the target claim: `dont show <id> --json`
2. Confirm the reason for doubt is specific and actionable
3. Output: `dont trust <id> --reason "<reason>"`
4. To retract later: `dont undoubt <id>`
"#
    .to_string()
}

fn lock_skill() -> String {
    r#"---
description: dont-grill/lock — guide the agent through locking a verified claim
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/lock

Guides the agent through locking a claim that has been fully verified.

## Protocol

1. Confirm the claim is in `verified` status: `dont show <id> --json`
2. Confirm no open doubts remain
3. Output: `dont lock <id>`
"#
    .to_string()
}

fn ignore_skill() -> String {
    r#"---
description: dont-grill/ignore — guide the agent through setting aside an out-of-scope claim
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/ignore

Guides the agent through ignoring a claim that is out of scope.

`dont ignore` = "do not engage with it" — scope exclusion, not deletion.

## Protocol

1. Confirm the claim is genuinely out of scope (not just low priority)
2. Output: `dont ignore <id>`
3. To re-engage later: `dont reopen <id>`
"#
    .to_string()
}

fn trace_skill() -> String {
    r#"---
description: dont-grill/trace — guide the agent through tracing claim history
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/trace

Guides the agent through inspecting a claim's full event history.

## Protocol

1. Fetch claim detail: `dont show <id> --json`
2. Walk through each event in `history` chronologically
3. Summarise current status and any unmet rule clauses
"#
    .to_string()
}

fn scenarios_skill() -> String {
    r#"---
description: dont-grill/scenarios — enumerate scenario coverage for a claim
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/scenarios

Guides the agent through enumerating the scenarios a claim must cover.

## Protocol

1. Review the claim text: `dont show <id> --json`
2. List the edge cases and boundary conditions the claim implies
3. For each scenario, ask: is there a corresponding `dont conclude` or test?
4. Output a checklist of missing coverage
"#
    .to_string()
}

fn conclude_worthiness_skill() -> String {
    r#"---
description: dont-grill/conclude-worthiness — assess whether a statement warrants a dont claim
disable-model-invocation: true
---

<!-- MANAGED BY dont — DO NOT EDIT MANUALLY -->

# dont-grill/conclude-worthiness

Assesses whether a candidate statement is worth introducing as a `dont` claim.

## Worthiness criteria

A statement is claim-worthy if it:
- Is falsifiable (can be doubted with `dont trust`)
- Is verifiable (can be cleared with `dont flag --evidence`)
- Would affect decisions or downstream claims if it turned out to be wrong
- Is not already captured by an existing claim

## Protocol

1. State the candidate claim text
2. Apply each criterion above; ask one question at a time
3. Output: `WORTHY` or `NOT WORTHY` with the deciding criterion
4. If worthy: hand off to `dont-grill/conclude`
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn generate_dont_grill_is_deterministic() {
        let a = generate_pack("dont-grill").unwrap();
        let b = generate_pack("dont-grill").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn generate_dont_grill_contains_router_and_all_sub_skills() {
        let files = generate_pack("dont-grill").unwrap();
        assert_eq!(files.len(), 10, "expected router + 9 sub-skills");
        assert!(
            files.contains_key("dont-grill.md"),
            "router must be present"
        );
        for sub in &[
            "subs/conclude.md",
            "subs/define.md",
            "subs/flag.md",
            "subs/trust.md",
            "subs/lock.md",
            "subs/ignore.md",
            "subs/trace.md",
            "subs/scenarios.md",
            "subs/conclude-worthiness.md",
        ] {
            assert!(files.contains_key(*sub), "sub-skill {sub} must be present");
        }
    }

    #[test]
    fn generate_dont_grill_router_uses_canonical_verbs() {
        let files = generate_pack("dont-grill").unwrap();
        let router = &files["dont-grill.md"];
        assert!(
            router.contains("dont flag"),
            "should use canonical 'dont flag'"
        );
        assert!(
            router.contains("dont lock"),
            "should use canonical 'dont lock'"
        );
        assert!(
            !router.contains("dont dismiss"),
            "must not use deprecated 'dont dismiss'"
        );
        assert!(
            !router.contains("dont forget"),
            "must not use deprecated 'dont forget'"
        );
    }

    #[test]
    fn pack_content_hash_changes_when_content_differs() {
        let mut files = generate_pack("dont-grill").unwrap();
        let original_hash = pack_content_hash(&files);
        files.insert("dont-grill.md".to_string(), "mutated content".to_string());
        let mutated_hash = pack_content_hash(&files);
        assert_ne!(original_hash, mutated_hash);
    }

    #[test]
    fn disk_content_hash_matches_pack_content_hash() {
        let dir = TempDir::new().unwrap();
        let files = generate_pack("dont-grill").unwrap();
        let expected_hash = pack_content_hash(&files);

        for (rel, content) in &files {
            let dest = dir.path().join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut f = std::fs::File::create(&dest).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }

        let disk_hash = disk_content_hash(dir.path()).unwrap();
        assert_eq!(expected_hash, disk_hash);
    }

    #[test]
    fn generate_pack_returns_error_for_unknown_name() {
        let result = generate_pack("not-a-real-pack");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not-a-real-pack"));
    }
}
