---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-10-sk11-term-label-validation-edge-cases, pipeline-step:plan]
---

dont-4k5.10 SK11 label validation edge cases

Audit scope: 4 helper functions in src/main.rs
- label_has_indefinite_article (line 1921)
- label_ends_with_sentence_punctuation (line 1929)
- label_compound_undeclared (line 1934)
- label_contains_sentence_verb (line 2016)

SK11 format: singular indefinite noun phrase starting with 'a'/'an', no trailing punctuation, compound labels declare variables, no sentence verbs outside parens/where-clauses.

Existing tests: 16 integration tests in tests/define.rs covering happy/sad paths but missing edge cases.

Test strategy: unit tests in src/main.rs for the helper fns directly.

Test cases to write:
1. label_has_indefinite_article: 'a' alone (no noun) -> false
2. label_has_indefinite_article: 'an' alone -> false
3. label_has_indefinite_article: '  a  noun  ' (extra whitespace) -> true
4. label_ends_with_sentence_punctuation: trailing whitespace after period -> true
5. label_ends_with_sentence_punctuation: comma not in set -> false
6. label_compound_undeclared: 'a pair ()' (empty parens) -> true
7. label_compound_undeclared: 'a pair (x,)' (trailing comma, arity 1) -> true
8. label_compound_undeclared: 'a sequence ()' (None required, empty) -> true
9. label_compound_undeclared: 'a list of (x)' (None required, 1 var) -> false
10. label_contains_sentence_verb: verb appears after closing paren but before 'where' -> true
11. label_contains_sentence_verb: verb in where-clause only -> false (no spurious flag)
12. best_article_for: empty string -> 'a' (no panic)
13. best_article_for: 'umbrella' starts with vowel -> 'an'
14. best_article_for: 'unicorn' starts with vowel but sounds like consonant -> 'an' (phonetic not modeled)

Expected: all pass without code changes if validator is sound; any failure = real bug.
