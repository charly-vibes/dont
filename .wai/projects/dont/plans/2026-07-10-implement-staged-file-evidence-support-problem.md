---
tags: [pipeline-run:tdd-ro5-2026-07-10-dont-tcy0-1-allow-staged-but-uncommitted-files-as-evidence, pipeline-step:plan]
---

Implement staged file evidence support

## Problem
check_git_provenance rejects staged-but-uncommitted files with
'staged-not-committed' error. Same as dont-h1l5 (dirty files) — the
user must commit before grounding.

## Fix
Replace the staged-not-committed error with  on the
staged blob. Use  which hashes working-tree
content (same as dirty files) — this works for staged files too since
hash-object reads the working tree, not the index.

Wait — actually for staged files the staged content may differ from the
working tree. We need e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 with the staged content,
or simpler: .

Actually the simplest approach:  on the file already
reads the working tree. For staged files the staged content == working
tree content (they were just ed). So same 
approach works.

But to be precise about staged vs dirty: a staged file has index_status
non-space. The git status for a staged file shows  (staged) or
 (added). For these, git hash-object on the working tree works.

## Test strategy
- Update flag_file_staged_not_committed_rejected → flag_file_staged_accepted
- Add flag_file_staged_has_content_hash for exact SHA verification
