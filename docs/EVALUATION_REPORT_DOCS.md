# UX/DX Evaluation Report
## Target: `dont` Documentation
## Scope: Layer 5 (Documentation - Diataxis)
## Date: 2026-05-08

## Layer Summary
| Layer | Framework | Status | Critical Findings |
|-------|-----------|--------|-------------------|
| Documentation | Diataxis | DEGRADED | 1 |

## Critical Findings (must-fix)
1. [Documentation] [L5-01]: **Missing Tutorial (Guided Learning)** — The project lacks a guided "Getting Started" tutorial. For a tool introducing a novel epistemic workflow (interrupted assertions, grounded claims), users need a hands-on path to build mental models. Without this, onboarding relies on reading conceptual explanations, which has a higher cognitive load. — *Remediation: Create a 'Tutorial' section in the mdBook that walks a user through initializing a project, concluding a claim, and grounding it with evidence.*

## High Findings (should-fix)
1. [Documentation] [L5-02]: **Modality Contamination in How-To Guides** — `grounding-workflow.md` contains significant explanatory content (e.g., rationale for repository-relative locators). While the content is valuable, it slows down goal-oriented readers. — *Remediation: Move the "Why" sections to `purpose.md` or a new "Deep Dive" explanation page, leaving only steps and commands in the How-To guide.*

## Cross-Layer Effects
- **Adoption Barrier**: The absence of a tutorial (L5) makes it harder for potential contributors or users to "feel" the tool's value, potentially slowing down the transition from specification to implementation (L2).

## Measurement Gaps
- **User Feedback**: No direct user feedback on documentation clarity was available for this evaluation.
- **API/CLI Reference**: While OpenSpecs exist, a dedicated CLI command reference (e.g., `dont help` output captured in docs) would strengthen the Reference modality.

## Verdict
Overall Health: **DEGRADED** (due to missing onboarding path)
Highest-friction layer: **Documentation (Tutorials)**
Start here: **Create a "Getting Started" tutorial.** The existing explanations are excellent; the project now needs a "learning by doing" entry point.
