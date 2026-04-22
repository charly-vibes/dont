### v0.3.3 patches (post-olog-review)

Targeted additions closing a discipline gap in `dont define` identified by a review of Spivak & Kent's "Ologs: A Categorical Framework for Knowledge Representation" (arXiv:1102.1889v2, 7 Aug 2011, hereafter **SK11**) against v0.3.2. No CRITICAL findings; the patch pass adds one optional flag to `dont define`, five verb-level validators with corresponding error codes, one warn-level rule, and two config blocks. No change to the four-verb core, the lifecycle verbs, the data lattice, the spawn protocol, or the envelope schema. `envelope_version` remains `"0.2"` — the patch is additive on the verb signature and error-code table. No field is renamed or removed.

**Find.** v0.3 promoted the soft `vague-reason` rule into two hard verb-level validators on `trust` (§9.3: `reason-required`, `reason-not-hedge`), on the grounds that mixing warn-by-default rules with hard verb-level refusals produced inconsistency. The reasoning applies symmetrically to `define`, which in v0.3.2 accepts any `--doc` prose without shape checks. SK11 §2.1.1 ("Rules of good practice" for types, p. 7–8) and §2.2.1 (for aspects, p. 11) specify a five-item and three-item checklist respectively for the text of a well-formed type or aspect. A mechanically-checkable subset is enforceable at verb-level with the same deterministic, offline, non-overridable semantics as the `trust` validators. The semantic subset (recognisability, documentable instances, functional dependence) is unenforceable from a string alone and stays in the rule layer or at `assume` time.

**Why a patch and not v0.4.** No schema change. No new primitive (the aspects-as-primitive proposal remains open as a §17 item; this patch deliberately does not touch §6's budget). No change to any existing refusal code. The work is additive on the `define` verb and its error-code table, and it follows the exact pattern already established on `trust`.

---

**Layer-1 (verb signature) patch — §9.2.**

`dont define` gains one optional flag:

```
dont define <curie>
        --doc "<prose definition>"          # required
        [--label "<noun phrase>"]           # [v0.3.3] optional
        [--kind-of <parent-curie>]*
        [--related-to <other-curie>]*
        [--attribute <attr-spec>]*
        [--author <id>]
        [--json]
```

`--label` carries the **type text** in the SK11 §2.1 sense — the singular indefinite noun phrase that would appear inside the box in an olog diagram (e.g. `"a Ricci tensor"`, `"an amino acid found in dairy"`). `--doc` continues to carry the extended definitional gloss. Where `--label` is supplied, the §9.2.1 validators apply to it; where it is absent, the same validators apply to a leading-phrase extraction of `--doc` (the substring up to the first `.`, `?`, `!`, or `;`, trimmed) at warn-level rather than refuse-level. The rationale for the asymmetry: retroactively rejecting projects' existing `--doc` values would be hostile; supplying `--label` is the forward path, and the warn-level fallback pressures new calls toward supplying it without breaking old ones.

The seed YAML shape (§7) gains an optional `label` field per term; seed entries without `label` suppress the §9.2.1 checks on that entry (the seed is pre-locked and is not round-tripped through `define`).

---

**Layer-2 (verb-level validators) — new §9.2.1.**

Five checks, all deterministic, all network-free, all running before any rule evaluation. Checks fire in listed order; first failure refuses and skips the remainder. Each refusal carries an `error` envelope with non-empty `remediation[]` (§3.2.5) pointing at the specific fix.

1. **`term-label-empty`** — `--label`, if supplied, is the empty string or whitespace-only after trim. SK11 §2.1.1 is silent on this (the paper assumes a label exists); it is included for completeness with the `--doc` emptiness check already in v0.3.2 wiring.

2. **`term-shape-indefinite`** — the label does not begin with `a` or `an` followed by whitespace. Regex floor: `^(a|an)\s+\S`, case-insensitive. SK11 §2.1.1(i): *"begin with the word 'a' or 'an'"* (p. 7). Refusal example:

   ```json
   {
     "ok": false,
     "envelope_kind": "error",
     "data": {
       "code": "term-shape-indefinite",
       "detail": "label must begin with 'a' or 'an' — got 'Ricci tensor'",
       "remediation": [{
         "command": "dont define proj:RicciTensor --label 'a Ricci tensor' --doc ...",
         "description": "prepend 'a' or 'an' to the label"
       }]
     }
   }
   ```

3. **`term-shape-punctuated`** — the label ends in `.`, `?`, `!`, `;`, or `:`. Regex floor: `[.?!;:]\s*$`. SK11 §2.1.1(iv): *"not end in a punctuation mark"* (p. 7). Sentence-shaped labels read badly when composed into aspect sentences (SK11 §2.2.2, p. 9) and anticipate sentential content that belongs in `--doc`.

4. **`term-compound-undeclared`** — the label matches one of the compound markers and does not declare variables in a following parenthesised list. Markers: `a pair`, `a triple`, `a quadruple`, `a sequence`, `a tuple`, `a set of`, `a list of`, case-insensitive. Check: if the label starts with one of these (up to the head noun), it must contain a parenthesised variable list of matching arity before the first `where` or end-of-string. SK11 §2.1.1(v): *"declare all variables in a compound structure"* (p. 8). Example refusal payload detail: `"compound label 'a pair of integers' does not declare its variables; expected a form like 'a pair (x, y) where x and y are integers'"`. Refusal quotes SK11 §2.1.1 as the remediation reference.

5. **`term-label-sentence`** — the label contains a verb in a position that would make it a sentence rather than a noun phrase. Heuristic floor: label contains the tokens `is`, `are`, `has`, `have`, `does`, `do`, `was`, `were` as standalone tokens (whitespace-bounded) and not enclosed in a parenthesised variable declaration. SK11 §2.1 (p. 7) specifies *"singular indefinite noun phrase"*, and SK11 §2.2.1(ii) (p. 11) reserves verb-led sentence shape for aspects. The heuristic is deliberately shallow (it will false-positive on "a box that is empty" and similar); calibration data from early adoption will inform whether to tighten or loosen.

All five checks run on the label when `--label` is supplied; checks 2, 3, and 5 also run on the extracted leading phrase of `--doc` at warn-level when `--label` is absent. Check 4 does not run on `--doc` extractions — compound-structure detection in unbounded prose is too noisy to be useful.

**Why these five and not SK11's remaining three.** SK11 §2.1.1(ii) *"refer to a distinction made and recognizable by the author"* and §2.1.1(iii) *"refer to a distinction for which instances can be documented"* are semantic and not string-shape checkable. They remain unenforced at verb level. SK11 §2.2.1 (aspects) does not apply in v0.3.3 because aspects are not a primitive (see §6's budget; aspects as a sixth primitive is open as §17 item).

---

**Layer-3 (error codes) — §10.5 additions.**

The §10.5 known-codes list gains five entries in the `refusal` bucket, each with exit code `1` per §10.7.1:

| Code | Fires on | `remediation[0].command` template |
|------|----------|-----------------------------------|
| `term-label-empty` | `--label` present and empty/whitespace | `dont define <curie> --label '<noun phrase>' ...` |
| `term-shape-indefinite` | label does not start with `a`/`an` | `dont define <curie> --label 'a <head-noun>' ...` |
| `term-shape-punctuated` | label ends in `. ? ! ; :` | `dont define <curie> --label '<stripped label>' ...` |
| `term-compound-undeclared` | compound marker without variable list | `dont define <curie> --label 'a pair (x, y) where x ...' ...` |
| `term-label-sentence` | verb token in non-declared position | `dont define <curie> --label 'a <noun phrase without verb>' ...` |

Warn-level sibling codes for when checks run against extracted `--doc` leading phrase: `term-doc-shape-indefinite`, `term-doc-shape-punctuated`, `term-doc-shape-sentence`. These are emitted as `warnings[]` entries, not refusals; the `define` call succeeds and the term enters `:unverified`.

---

**Layer-4 (rule layer) — §13 addition (optional, warn).**

One new rule ships off-by-default, shipped as `.dont/rules/term-nonfunctional-label.dl`:

**`term-nonfunctional-label`** — flags terms whose label text suggests a non-functional relationship has been folded into the label. Pattern match on label tokens: `has a child`, `owns a`, `uses a`, `contains a`, and configurable siblings in `config.toml` under `[rules.term_nonfunctional.patterns]`. SK11 §2.2.3 (p. 10–11) specifies that non-functional relationships must be reshaped into spans — the rule surfaces candidates that would benefit from aspect-shaped redesign, once aspects land. Severity: `warn` in both modes, overridable. This rule is included so that projects adopting v0.3.3 can start collecting candidate aspect-shaped definitions for the eventual §17 aspect-primitive landing.

Not a verb-level check because the pattern is genuinely heuristic and false-positive-prone (e.g. `"a Python library that uses a C backend"` is a perfectly valid noun phrase and should not refuse).

---

**Layer-5 (config) — §14 additions.**

Two new blocks in `config.toml`:

```toml
[define.shape]
# Verb-level validators on `dont define` (§9.2.1).
# All default to enforcing at refuse-level when --label is supplied,
# warn-level when only --doc is available. Set to false to disable entirely.
check_indefinite     = true
check_punctuated     = true
check_compound       = true
check_sentence       = true
# Compound-structure markers recognised by term-compound-undeclared.
# Extend per-project if your domain has its own compound shapes.
compound_markers = [
  "a pair", "a triple", "a quadruple",
  "a sequence", "a tuple",
  "a set of", "a list of"
]

[rules.term_nonfunctional]
# Warn-level rule (§13). Disabled by default; enable per-project.
enabled = false
patterns = [
  "has a child", "has a parent",
  "owns a", "uses a", "contains a"
]
```

---

**Layer-6 (documentation) — §11.1 orientation block addition.**

One line added to the on-session orientation block (§11.1) between the `suggest-term` line and the `help --tutorial` line:

```
When coining, pass --label '<a noun phrase>' alongside --doc.
The label is what appears in diagrams and is shape-checked.
```

§11.3 Step 3 (the `define`-teaching step in the tutorial walkthrough) is revised to show the `--label` flag in its example and to describe the shape checks as a teaching moment for the SK11 rules of good practice. §11.4 how-to guides are not affected. The §11.2 worked example's Call 3 is updated to include `--label "a Ricci tensor"`.

---

**Layer-7 (acceptance criteria) — §19.1 additions.**

Four new criteria:

- **Label-shape enforcement.** For each of the five §10.5 codes, a test-corpus `dont define` invocation with the relevant malformed label exits `1` with the expected `code` in the envelope. Run against a frozen fixture set of 20 malformed labels and 20 well-formed labels; no false positives on the well-formed set.
- **Doc-extraction fallback.** A `dont define` call without `--label` and with a malformed `--doc` leading phrase exits `0` but emits the corresponding `term-doc-shape-*` warning. No refusal when `--label` is absent.
- **Seed immunity.** `dont init` on a fresh project succeeds with all seed terms regardless of whether their seed YAML entries carry `label`; no `term-shape-*` refusal fires during seed installation.
- **Message linting.** All five new error messages and all three new warning messages pass the §19.1 Vale configuration (imperative voice in remediations, no hedge patterns, no undefined jargon).

---

**Deliberately unchanged.**

- The four-verb core and the lifecycle verbs. No new verb.
- The 5-primitive budget in §6. Aspects remain unadopted; this patch does not pre-empt the §17 decision.
- `--doc` remains required on `define`. `--label` is optional.
- `define`'s existing refusal conditions (`dangling-definition`, unresolved `--kind-of` / `--related-to`) are unchanged.
- The CURIE shape validator on `<curie>` — already present in v0.3.2 — is unchanged. Labels and CURIEs are independent; a well-shaped label does not imply a well-shaped CURIE and vice versa.
- The envelope schema. `warnings[]` and `data.code` are pre-existing fields.

---

**Consequence-only changes.**

- §7 seed YAML shape gains optional `label` field per term.
- §9.2 verb signature gains `[--label "<noun phrase>"]`.
- §10.5 known-codes list gains five `term-*` refusal codes and three `term-doc-*` warning codes.
- §13 shipped-rules list notes `term-nonfunctional-label` as an off-by-default rule.
- §14 `config.toml` example gains `[define.shape]` and `[rules.term_nonfunctional]` blocks.
- Appendix A gains entries for `Label` (the SK11 type text), `Compound type`, and `Functional relationship` (the SK11 aspect invariant, referenced forward even though aspects are not a v0.3 primitive).

---

**References.**

SK11 = Spivak, D. I. and Kent, R. E. *Ologs: A Categorical Framework for Knowledge Representation.* arXiv:1102.1889v2 [cs.LO], 7 Aug 2011. Section and page citations above refer to this paper.

Primary SK11 sections relied on by this patch:

- §2.1 *Types* (p. 7): the "type as singular indefinite noun phrase" convention.
- §2.1.1 *Rules of good practice* (p. 7–8): the five-item checklist that maps onto §9.2.1 validators 1–5. This patch adopts items (i), (iv), (v) as hard verb-level checks and items (ii), (iii) as rule-layer / assume-time semantic concerns.
- §2.2 *Aspects* (p. 8–9): the functional-relationship invariant, referenced forward by `term-nonfunctional-label` rule but not enforced as a primitive in v0.3.3.
- §2.2.1 *Rules of good practice* for aspects (p. 11): rationale for the `term-label-sentence` check — reserving verb-led sentence shape for aspects, keeping type labels as noun phrases.
- §2.2.3 *Converting non-functional relationships to aspects* (p. 10–11): rationale for the `term-nonfunctional-label` warn rule.

Primary v0.3.2 spec sections affected: §7 (seed vocabulary), §9.2 (`dont define`), §9.3 (`trust` validators — the pattern this patch mirrors), §10.5 (known error codes), §10.7.1 (exit-code taxonomy), §11.1 (orientation block), §11.2 (worked example), §11.3 (tutorial), §13 (rules), §14 (config), §19.1 (acceptance criteria), §21 (patch history), Appendix A (glossary).

Open, deferred to §17: aspects as a sixth primitive (SK11 §2.2, §2.2.3); commutative-diagram facts as a claim sub-kind (SK11 §2.3); functorial alignment between project ologs and imported ontologies (SK11 §4.2–4.3). v0.3.3 is explicitly scoped to forbid anticipating these.
