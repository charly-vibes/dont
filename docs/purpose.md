# Why `dont` exists

## The problem

Large language models are good at producing fluent answers, but fluency is not the same thing as epistemic discipline.

Once a model has started down a line of reasoning, it often tends to elaborate on that line instead of questioning it. In practice, this means an autonomous agent can:

- state a claim too early
- defend the claim after weak checking
- fold the claim into project memory or downstream actions
- create a trail of confident but poorly grounded work

The core idea behind `dont` is that this failure mode should not be handled as a style problem. It should be handled as a tooling problem.

## The research-backed claim

The research corpus in this repository points to a consistent conclusion:

1. LLMs do not reliably self-correct just because they are asked to reconsider.
2. Better results usually come from **external checks** such as retrieval, tests, verifiers, or independent critics.
3. Durable human institutions solve similar problems by separating assertion from evaluation.

So the design stance of `dont` is simple:

> Do not let an agent assert what it has not yet grounded.

## What `dont` does

`dont` is a forcing function for autonomous LLM harnesses.

It gives the model a mechanical path between "I want to say this" and "this is now grounded enough to keep."

Instead of letting every plausible sentence become accepted project state, `dont` introduces explicit transitions such as:

- conclude a claim
- define a term
- add evidence or challenge
- move entities through statuses like unverified, doubted, verified, or locked

The point is not to make the model sound cautious. The point is to make the workflow itself require justification.

## Why a tool instead of a prompt

Prompting alone is weak because the same model that produced the claim is often asked to judge its own claim in the same context.

The draft spec and research notes both argue for the opposite pattern:

- make claims explicit
- record state changes
- surface unmet conditions as actionable refusals
- use fresh-context verification paths when independence matters

That is why `dont` is proposed as a peer CLI tool, not just a prompt template.

## The role of `dont` in a harness

A harness can use three different kinds of support:

- **memory tools** to remember what happened
- **workflow tools** to know what stage of work is happening
- **epistemic tools** to control what is allowed to count as grounded

`dont` is the third category.

It is meant to sit beside workflow and memory tools, but remain independent from them. Its concern is narrower: claims, terms, evidence, rules, and the right to assert.

## Non-goals

`dont` is not meant to be:

- a general ontology editor
- a replacement for external validators or retrieval systems
- a memory system
- a workflow planner
- a magical truth machine

It is a guardrail and protocol layer.

## Read next

- [Research basis](./research.md)
