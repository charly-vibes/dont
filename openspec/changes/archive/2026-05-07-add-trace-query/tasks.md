## 1. Specify the tracing query
- [x] 1.1 Add `dont-trace-query` requirements for blocker-path inspection
- [x] 1.2 Define the minimum path fields required for actionable diagnosis and constrain remediation to valid next commands
- [x] 1.3 Define success, empty-result, and cycle-safe behaviour for fully healthy or cyclic entities

## 2. Integrate with existing query/payload surfaces
- [x] 2.1 Update derived-query expectations so `trace` is part of the read-only query surface
- [x] 2.2 Add a structured payload contract for trace output

## 3. Teach the diagnostic flow
- [x] 3.1 Update help/tutorial expectations so tracing is the recommended next step when verification is blocked

## 4. Validate
- [x] 4.1 Run `openspec validate add-trace-query --strict`
