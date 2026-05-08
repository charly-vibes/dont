# Getting Started with `dont`

This tutorial walks you through the core epistemic workflow of `dont`.

By the end, you will understand how to transition a claim from an unverified assertion to a grounded project fact.

## 1. Initialize the project

First, initialize `dont` in your repository. This creates the `.dont/` directory to store your project's epistemic state.

```bash
dont init
```

## 2. Conclude a claim

Imagine you've just discovered that the project uses `uv` for Python management. Instead of just remembering this in your head (or chat memory), record it as a project claim.

```bash
dont conclude "The project uses uv for Python dependency management"
```

The tool will return an ID for this claim (e.g., `claim-abc`).

## 3. Observe the refusal

`dont` is designed to interrupt unsupported assertions. If you try to see the details or rely on this claim, the tool will remind you that it is currently **unverified**.

```bash
dont show claim-abc
```

You will see a status of `unverified`. If you were using a harness that integrates with `dont`, the harness would refuse to let you use this claim as a "fact" in downstream prompts until it is grounded.

## 4. Ground the claim with evidence

To "ground" the claim, you need to point to evidence in the repository. In this case, the `README.md` likely mentions `uv`.

Use `dont ground` to provide the evidence and move the claim toward verification in one step:

```bash
dont ground "The project uses uv for Python dependency management" --file README.md
```

Wait—`ground` is for *new* claims. If you want to verify the *existing* `claim-abc`, you use `flag`:

```bash
dont flag claim-abc --file README.md
```

## 5. Verify the status

Now, check the claim again:

```bash
dont show claim-abc
```

The status is now `verified`. The claim is now "grounded" and can be safely asserted by an autonomous agent.

## Summary of the lifecycle

- **Conclude**: "I want to say this." (Status: `unverified`)
- **Flag**: "Here is the proof." (Status: `verified`)
- **Lock**: "This is mature and shouldn't change easily." (Status: `locked`)

That's it! You've successfully moved a claim through the `dont` epistemic workflow.
