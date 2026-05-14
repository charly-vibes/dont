---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-10-sk11-term-label-validation-edge-cases, pipeline-step:red]
---

Red phase: 21 unit tests written for label_validation_tests module. All pass immediately — validator handles: article-alone edge cases, extra whitespace, case insensitivity, trailing whitespace after punctuation, comma not flagged, empty parens in compound, trailing comma arity, sequence/list-of variants, verb after paren before where, verb in where-clause exemption, best_article_for with empty string and vowel/consonant. No production code changes needed.
