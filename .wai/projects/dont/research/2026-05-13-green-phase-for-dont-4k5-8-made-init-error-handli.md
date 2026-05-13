---
tags: [pipeline-run:tdd-ro5-2026-05-13-dont-4k5-8-project-initialization-error-paths, pipeline-step:green]
---

Green phase for dont-4k5.8: made init error handling path-aware for create/write failures, removed the expect() from init_event by emitting the JSONL line directly, and verified full cargo test passes.
