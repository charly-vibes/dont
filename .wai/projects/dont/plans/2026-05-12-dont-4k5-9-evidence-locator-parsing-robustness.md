---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-9-evidence-locator-parsing-robustness, pipeline-step:plan]
---

## dont-4k5.9: Evidence Locator Parsing Robustness

### What
Harden parse_line_span() and current_locator_text() against all malformed inputs.
The parser returns Result so panics are not expected, but untested edge cases exist.

### Parser location
- src/main.rs:1243-1260 — parse_line_span(s: &str) -> Result<(u32,u32),String>
- src/main.rs:951-983 — current_locator_text() — contains slice op at line 979 (bounds-checked but untested edge cases)
- Tests: src/tests/evidence_drift.rs

### Risk surface
parse_line_span: handles single numbers and ranges via split_once('-'), returns Err on non-numeric. Gaps:
- Negative numbers: '-5', '10--2'
- Non-numeric: 'abc', '10-x', '10.5'
- Whitespace: '10 - 20', ' 10'
- Very large numbers (u32 overflow via .parse() — actually safe, returns Err)
- Empty string, '-' alone, null bytes
current_locator_text slice at line 979 is bounds-guarded but validation logic untested under edge inputs.

### Test strategy (TDD — write failing tests first)
Unit tests in src/tests/evidence_drift.rs or a new unit mod in main.rs:

1. parse_line_span('') -> Err, message mentions empty
2. parse_line_span('-') -> Err
3. parse_line_span('abc') -> Err
4. parse_line_span('10-x') -> Err, identifies bad segment
5. parse_line_span('-5') -> Err (negative start)
6. parse_line_span('10--2') -> Err
7. parse_line_span('10.5') -> Err
8. parse_line_span(' 10 ') -> Ok((10,10)) or Err — define expected behavior
9. parse_line_span('4294967295') -> Ok or defined Err (u32::MAX edge)
10. parse_line_span('99999999999999') -> Err (overflow)

Integration test: dont add claim with locator containing malformed line span returns error, not panic.

### No fuzz infrastructure — out of scope for this ticket.

### Pass criteria
All 10+ malformed input unit tests pass; no test calls panic.
Existing 16+ tests continue green.
