---
tags: [pipeline-run:tdd-ro5-2026-07-08-dont-h1l5-dont-flag-file-rejects-evidence-with-unstaged-edits, pipeline-step:plan]
---

F21: Allow  to accept tracked files with unstaged modifications

## Problem
 fails when the evidence file has unstaged
(working-tree dirty) modifications. The error forces users to commit-before-ground,
inverting the natural workflow where findings are grounded *while* being written up.

## Root cause
 parses  M .wai/.pipeline-run
 M .wai/projects/dont/.pending-resume
 M .wai/resources/pipelines/.last-run
?? .wai/pipeline-runs/tdd-ro5-2026-07-08-dont-h1l5-dont-flag-file-rejects-evidence-with-unstaged-edits.yml
?? .wai/projects/dont/handoffs/2026-07-08-session-end.md output. When
 (dirty file), it calls  with
a  refusal — no recovery path.

## Fix
For dirty tracked files, use  to compute the
working-tree content SHA-1, then return a  provenance ref.
This is conceptually similar to the existing  for clean files,
but anchored to content rather than a commit.

## Test strategy
- **Existing test updated**:  → 
  — expects success (ok=true), a  commit_ref, and the 
  code no longer appears
- **New test**:  — verifies the exact format
  of the content hash using  from the test itself
- **Existing tests unchanged**: ,
  , 
  (clean file), and all non-git tests

## Interface change
-  field in the repo-file locator JSON accepts a new prefix:
   for dirty files (vs  for clean/committed files)
- No CLI flag changes, no new commands
