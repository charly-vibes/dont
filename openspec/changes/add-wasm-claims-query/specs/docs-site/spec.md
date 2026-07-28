# docs-site Specification Delta

## ADDED Requirements

### Requirement: Interactive claims-query page in the documentation site
The documentation site SHALL include a page that loads the `dont` WASM query
module and a committed example snapshot, and lets the reader browse claims and
terms, run shipped rules, and execute read-only Datalog queries against the
snapshot — all in the browser with no binary installation.

#### Scenario: Reader opens the claims explorer
- **WHEN** a reader navigates to the claims-query page in the published
  mdBook site
- **THEN** the page loads the WASM module and an example snapshot
- **AND** the reader can list claims and terms without installing the `dont`
  binary

#### Scenario: Reader runs a shipped rule in the browser
- **WHEN** the reader selects a shipped rule (e.g. `ungrounded`) on the
  claims-query page
- **THEN** the page displays the rule's matches against the loaded snapshot
- **AND** the result matches what the native CLI would produce on the same
  snapshot

#### Scenario: Reader runs a read-only Datalog query
- **WHEN** the reader enters a `?[...] :=` Datalog query on the claims-query
  page
- **THEN** the page displays the result rows
- **AND** a mutating script is rejected with an error message rather than
  applied

### Requirement: Snapshot is cached across page reloads
The claims-query page SHALL cache the snapshot blob in browser storage
(e.g. IndexedDB) keyed
by a content hash, so that navigating away and returning does not re-fetch or
re-parse the snapshot unless it has changed.

#### Scenario: Return visit reuses cached snapshot
- **WHEN** a reader returns to the claims-query page and the committed
  snapshot hash is unchanged
- **THEN** the page SHALL rehydrate from the cached snapshot without
  re-fetching it from the server

#### Scenario: Changed snapshot triggers re-fetch
- **WHEN** the committed snapshot hash differs from the cached one
- **THEN** the page SHALL fetch the new snapshot and update the cache
