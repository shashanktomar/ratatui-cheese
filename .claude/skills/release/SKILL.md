---
name: release
description: Create a new release of ratatui-cheese. Use this skill when the user says "make a release", "bump version", "publish", "release", "cut a release", or similar. Assumes the user has already merged any pending PRs — this skill handles version bumping, verification, tagging, and pushing.
---

# Release

Create a new versioned release. The GitHub Action (`.github/workflows/release.yml`) handles publishing to crates.io and creating the GitHub release — this skill just prepares and pushes the tag.

## Prerequisites

- You must be on the `main` branch with a clean working tree
- All PRs should already be merged

## Steps

### 1. Verify state

```bash
git checkout main
git pull origin main
git status  # must be clean
```

If not on main or working tree is dirty, stop and tell the user.

### 2. Determine version

Read the current version from `crates/ratatui-cheese/Cargo.toml`. Look at the commits since the last tag to understand what changed, then suggest a version bump with reasoning:

- **patch** (x.y.Z) — bug fixes, docs, minor tweaks
- **minor** (x.Y.0) — new features, new widgets, API additions
- **major** (X.0.0) — breaking API changes

Present the suggestion using AskUserQuestion with the three options and the suggested one marked as recommended. The user picks the final version. Do NOT proceed without their confirmation.

Current version is in:
- `crates/ratatui-cheese/Cargo.toml` (line 3, `version = "x.y.z"`)

### 3. Bump version

Update the version in `crates/ratatui-cheese/Cargo.toml`. Then run `cargo check` to regenerate `Cargo.lock`.

If the **major** version changed, also update the version in:
- `README.md` (the `ratatui-cheese = "X.Y"` in the install section)
- `crates/ratatui-cheese/README.md` (same)

### 4. Verify

```bash
just all
```

All tests, clippy, formatting, and dead-code checks must pass.

### 5. Commit and tag

```bash
git add crates/ratatui-cheese/Cargo.toml Cargo.lock
# Also add READMEs if major version changed
git commit -m "chore: bump version to X.Y.Z"
git tag vX.Y.Z
```

### 6. Push

```bash
git push origin main
git push origin vX.Y.Z
```

Pushing the tag triggers the release workflow which:
- Runs CI checks
- Publishes to crates.io
- Creates a GitHub release with auto-generated notes

### 7. Verify the release

```bash
gh run list --limit 1
```

Tell the user to check the GitHub Actions tab for the release workflow status.
