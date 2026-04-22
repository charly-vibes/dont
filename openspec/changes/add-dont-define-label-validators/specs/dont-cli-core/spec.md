## MODIFIED Requirements

### Requirement: Define introduces coined terms
The system SHALL let `define` introduce a project term with a CURIE and prose definition. `define` SHALL accept an optional `--label "<noun phrase>"` flag carrying the SK11 type-text — the singular indefinite noun phrase that would appear inside the box in an olog diagram (e.g. `"a Ricci tensor"`). When `--label` is supplied, the five verb-level shape validators (see "Define label shape validators" below) fire on it before any rule evaluation. When `--label` is absent, a warn-level subset of those validators fires on a leading-phrase extraction of `--doc` (the substring up to the first `.`, `?`, `!`, or `;`, trimmed, and capped at 80 characters or the first 15 tokens, whichever is shorter) without refusing the call.

#### Scenario: define creates an unverified term
- **WHEN** an actor invokes `define` with a CURIE and definition
- **THEN** the tool creates a term entity
- **AND** the entity enters the `unverified` state as defined by the lifecycle specification

#### Scenario: define accepts an optional label
- **WHEN** an actor invokes `define` with both `--doc` and `--label`
- **THEN** the label is stored as the SK11 type-text and the shape validators fire on it

#### Scenario: define without label emits doc-extraction warnings, not refusals
- **WHEN** an actor invokes `define` without `--label` and the leading phrase of `--doc` fails a shape check
- **THEN** the command succeeds and the envelope carries a `term-doc-shape-*` warning rather than refusing

#### Scenario: define refuses unresolved referenced terms
- **WHEN** an actor invokes `define` with parent, related, or attribute references that do not resolve
- **THEN** the command is refused

#### Scenario: define may append a redefinition for an existing coined CURIE
- **WHEN** an actor invokes `define` for a CURIE that already exists as a coined project term
- **THEN** the command may append a new definition event for that CURIE instead of creating an unrelated duplicate term

## ADDED Requirements

### Requirement: Define label shape validators
The system SHALL apply five deterministic, network-free validators to `--label` when it is supplied, in listed order, before any rule evaluation. The validators are individually disableable globally via the `[define.shape]` project configuration block and are NOT overridable per invocation. The first failure SHALL refuse the command and skip the remainder. Each refusal SHALL carry an error envelope with non-empty `remediation[]` pointing at the specific fix. The validators SHALL also fire at warn-level on the leading-phrase extraction of `--doc` when `--label` is absent, except for `term-compound-undeclared` which does not fire on doc extractions.

1. **`term-label-empty`** — the label is the empty string or whitespace-only after trim.

2. **`term-shape-indefinite`** — the label does not begin with `a` or `an` followed by whitespace. Regex floor: `^(a|an)\s+\S`, case-insensitive. Source: SK11 §2.1.1(i).

3. **`term-shape-punctuated`** — the label ends in `.`, `?`, `!`, `;`, or `:`. Regex floor: `[.?!;:]\s*$`. Source: SK11 §2.1.1(iv).

4. **`term-compound-undeclared`** — the label starts with a compound marker (`a pair`, `a triple`, `a quadruple`, `a sequence`, `a tuple`, `a set of`, `a list of`, case-insensitive) and does not contain a valid parenthesised variable list before the first `where` or end-of-string. For fixed-arity markers the list must contain exactly the required number of variables (`a pair` → 2, `a triple` → 3, `a quadruple` → 4). For open-arity markers (`a sequence`, `a tuple`, `a set of`, `a list of`), the list must contain at least one variable. Source: SK11 §2.1.1(v).

5. **`term-label-sentence`** — the label contains `is`, `are`, `has`, `have`, `does`, `do`, `was`, or `were` as whitespace-bounded tokens outside a parenthesised variable declaration or a `where`-clause (the text following the first occurrence of the token `where` that appears after a parenthesised variable list). Source: SK11 §2.1 and §2.2.1(ii), which reserves verb-led sentence shape for aspects.

#### Scenario: empty label is refused
- **WHEN** an actor invokes `define` with `--label ""`
- **THEN** the command is refused with `code: "term-label-empty"`
- **AND** `remediation[0]` suggests supplying a non-empty noun phrase

#### Scenario: label without article is refused
- **WHEN** an actor invokes `define` with `--label "Ricci tensor"`
- **THEN** the command is refused with `code: "term-shape-indefinite"`
- **AND** `remediation[0]` suggests prepending `a` or `an`

#### Scenario: sentence-punctuated label is refused
- **WHEN** an actor invokes `define` with `--label "a Ricci tensor."`
- **THEN** the command is refused with `code: "term-shape-punctuated"`
- **AND** `remediation[0]` suggests removing the trailing punctuation

#### Scenario: compound label without variable declaration is refused
- **WHEN** an actor invokes `define` with `--label "a pair of integers"` (no variable list)
- **THEN** the command is refused with `code: "term-compound-undeclared"`
- **AND** `remediation[0]` suggests a form like `"a pair (x, y) where x and y are integers"`

#### Scenario: sentence-shaped label is refused
- **WHEN** an actor invokes `define` with `--label "a tensor that is Ricci"`
- **THEN** the command is refused with `code: "term-label-sentence"`
- **AND** `remediation[0]` suggests rephrasing as a noun phrase without a verb token

#### Scenario: validators fire in order and stop at first failure
- **WHEN** an actor invokes `define` with a label that fails `term-shape-indefinite`
- **THEN** only the `term-shape-indefinite` error is returned and subsequent validators are not evaluated

#### Scenario: well-formed label passes all validators
- **WHEN** an actor invokes `define` with `--label "a Ricci tensor"` and valid `--doc`
- **THEN** all five validators pass and the term is created in `:unverified`

#### Scenario: well-formed label with 'an' passes all validators
- **WHEN** an actor invokes `define` with `--label "an amino acid found in dairy"` and valid `--doc`
- **THEN** all five validators pass and the term is created in `:unverified`

#### Scenario: doc-extraction check fires warn not refuse
- **WHEN** an actor invokes `define` without `--label` and `--doc "Ricci tensor. Extended explanation..."`
- **THEN** the command succeeds
- **AND** the envelope carries a `term-doc-shape-indefinite` warning

#### Scenario: doc-extraction does not fire compound check
- **WHEN** an actor invokes `define` without `--label` and `--doc` that starts with `a pair of integers`
- **THEN** the `term-compound-undeclared` validator does not fire on the doc extraction

#### Scenario: verb token in where-clause does not fire term-label-sentence
- **WHEN** an actor invokes `define` with `--label "a pair (x, y) where x is an integer and y is a real number"`
- **THEN** the `term-label-sentence` validator does not fire because the verb tokens appear inside a `where`-clause following a parenthesised variable list
