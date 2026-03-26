---
name: ratatui-cheese-review
description: 'Review the ratatui-cheese repository from three distinct perspectives: correctness, Rust best practices, and API design. Use this skill when the user asks for a repo review, design review, code audit, API critique, or requests written review output for this project. Trigger even when the user only mentions reviewing "this repo", "the codebase", "the library API", or asks for findings to be written into files.'
---

# Ratatui Cheese Review

## Overview

Review the full ratatui-cheese repository and write findings into a `review/` directory with one file per review angle. Treat this as a code review task: findings first, ordered by severity, with file/line references and concrete impact.

## Scope

Review the whole repository, including:

- `crates/ratatui-cheese/` library code
- `crates/showcase/` demo app
- examples, tests, docs, and ADRs when relevant
- workspace manifests and tooling files if they affect developer workflow or API quality

Do not review only the diff unless the user explicitly narrows scope.

## Workflow

### 1. Build context first

- Inspect the repo structure before judging details.
- Read the main library modules, examples, showcase, and docs to understand intended behavior.
- Run targeted validation when useful, usually `cargo test` and `cargo clippy --all-targets`.

### 2. Review from three separate angles

Keep the perspectives separate. A single issue can appear in more than one perspective only if the reasoning is genuinely different.

#### Correctness

Focus on:

- Bugs, panics, out-of-bounds behavior, invalid assumptions
- Behavioral regressions
- Incorrect rendering or state transitions
- Missing tests for fragile behavior
- Mismatches between docs/examples and actual implementation

This file should answer: "What can break or already behaves incorrectly?"

#### Rust Best Practices

Focus on:

- Idiomatic ownership and borrowing
- Panic resistance and boundary handling
- Trait impl quality
- Builder/state patterns
- Test quality and coverage gaps
- Clippy-worthy patterns, unnecessary clones, needless allocations, and maintainability concerns

This file should answer: "What is valid Rust, but not the best way to implement or maintain this?"

#### API Design

Focus on the public crate surface:

- Naming and conceptual clarity
- Separation between widget configuration and mutable state
- Extensibility and forward compatibility
- Consistency across modules
- Ease of use for downstream callers
- Whether defaults, constructors, and trait impls create surprising behavior

This file should answer: "Is this a good library API for users to depend on?"

### 3. Write the review output

Create `review/` at the repo root and write exactly these files:

- `review/correctness.md`
- `review/rust-best-practices.md`
- `review/api-design.md`

Each file should:

- Start with `#` title naming the perspective
- List findings first, ordered by severity
- Include file and line references for each finding
- Explain why the issue matters
- Mention missing tests when they materially increase risk
- State explicitly if no findings were found

Keep summaries brief and secondary. Do not bury findings under long introductions.

## Output Format

Use this structure inside each file:

```md
# <Perspective>

## Findings

### 1. <Short issue title>

- Severity: high|medium|low
- File: `path:line`

<Why this is a problem, with concrete impact.>
```

If there are no findings, use:

```md
# <Perspective>

## Findings

No findings.

## Residual Risks

- <short note about untested or unreviewed areas, if any>
```

## Review Standards

- Prefer evidence over style opinions.
- Do not invent intent; infer it from code, tests, examples, and docs.
- Avoid duplicate findings across the three files unless the perspective changes the reasoning.
- Call out when a problem is covered by existing tests versus missing regression coverage.
- If you run validation commands, reflect that in the written review when relevant.
