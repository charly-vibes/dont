---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-12-lock-unlock-pre-conditions, pipeline-step:fix]
---

review findings addressed for dont-4k5.12: HIGH fixed (forget output captured and status==locked asserted before reopen attempt); MEDIUM fixed (ok==false assertion added, already present — confirmed consistent); MEDIUM deferred (helper duplication extracted to separate refactor ticket dont-419p at P3); LOW omitted (eprintln debug pattern not used in codebase style)
