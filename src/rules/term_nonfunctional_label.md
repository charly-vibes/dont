# term-nonfunctional-label

**Severity:** warn (configurable)

## What the rule checks

`term-nonfunctional-label` fires when a term's `label` field (the SK11 type-text) does not follow the singular indefinite noun phrase convention. A correct label reads as a type description: "a discovery" or "an experimental result", not "discoveries" or "the result".

The rule checks that the label starts with "a " or "an " (case-insensitive).

## How to satisfy it

Update the term's label to a singular indefinite noun phrase:
```
dont define <curie> --label "a verified experimental result"
```

## Why it matters

In olog-style type boxes, labels conventionally read as "a <type>" or "an <type>" to support natural-language composition of relationships. Non-functional labels break this convention and make olog diagrams harder to read.
