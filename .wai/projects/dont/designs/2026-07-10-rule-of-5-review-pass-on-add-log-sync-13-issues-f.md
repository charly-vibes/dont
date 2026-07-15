---
tags: [pipeline-run:tdd-ro5-2026-07-10-dont-tcy0-1-allow-staged-but-uncommitted-files-as-evidence]
---

rule-of-5 review pass on add-log-sync: 13 issues found and fixed. Major: removed spec duplication (dont-log-sync vs dont-init-modes), added doctor check spec, added EventLine JSON schema table. Medium: added empty store/file scenarios, malformed-line validation strategy, snapshot semantics. Low: resolved --dry-run as MAY, documented merge=union last-writer-wins and concurrent export as undefined, added doctor --fix scaffolding scenarios.
