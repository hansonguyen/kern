---
name: kern-release
description: Use when releasing a new version of kern to crates.io. Covers pre-flight checks, semantic version determination from conventional commits, Cargo.toml bump, git tag, and crates.io publish.
---

# kern Release

## Overview

Guides a full kern release: pre-flight → semver determination → version bump → commit → tag → push → crates.io publish.

**This skill is rigid. Follow every step in order. Do not skip pre-flight. Do not push or publish before user confirmation.**

---

## Step 1: Pre-Flight

Run all four checks. **All must pass before continuing.** If any fail, stop and fix them.

```bash
# 1. Clean working tree (must be empty output)
git status --porcelain

# 2. Full test suite
cargo nextest run

# 3. Lint
cargo clippy -- -D warnings

# 4. Format
cargo fmt --check

# 5. Publish dry-run
cargo publish --dry-run
```

If `git status --porcelain` has output → uncommitted changes exist. Tell the user to commit or stash before releasing.

---

## Step 2: Determine Version Bump

Get the last release tag and list all commits since it:

```bash
# Get last tag (if none exist yet, use the first commit)
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)

# List commit subjects since last tag
git log ${LAST_TAG}..HEAD --format="%s"
```

**Semver rules (conventional commits):**

| Commit type | Bump |
|-------------|------|
| Any subject ending in `!` (e.g. `feat!:`, `fix!:`) | MAJOR |
| Any commit body containing `BREAKING CHANGE:` | MAJOR |
| `feat:` or `feat(…):` | MINOR |
| Everything else (`fix:`, `chore:`, `docs:`, `refactor:`, `test:`, `ci:`, `style:`, `perf:`) | PATCH |

Apply the **highest** bump across all commits. MAJOR > MINOR > PATCH.

Show the user:
- The current version (from `Cargo.toml`)
- The commit list since the last tag
- The derived bump type (PATCH / MINOR / MAJOR)
- The proposed new version

Then ask:
> "Proposed release: **vX.Y.Z** (PATCH/MINOR/MAJOR bump). Does this look right, or would you like to override the version?"

Wait for confirmation before continuing.

---

## Step 3: Apply Version Bump

Edit `Cargo.toml` — change the `version` field in `[package]` to the confirmed version:

```toml
version = "X.Y.Z"
```

Then update `Cargo.lock`:

```bash
cargo build 2>&1 | tail -2
```

Expected: compiles without errors (may be instant if nothing changed).

---

## Step 4: Commit and Tag

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(release): bump version to X.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

Verify tag was created:
```bash
git describe --tags
```

Expected: `vX.Y.Z`

---

## Step 5: Push and Publish Confirmation

Show a summary:

```
Ready to release:
  version:  vX.Y.Z
  tag:      vX.Y.Z
  commits:  [list from Step 2]

This will:
  1. git push origin main
  2. git push origin vX.Y.Z
  3. cargo publish

Proceed? (yes / no)
```

**Wait for explicit "yes" before running any of these commands.**

If "no" → tell the user the commit and tag exist locally; they can push and publish manually when ready.

---

## Step 6: Push and Publish

Only run if user said yes in Step 5.

```bash
git push origin main
git push origin vX.Y.Z
cargo publish
```

After `cargo publish` succeeds, confirm:

```
Released kern vX.Y.Z to crates.io.
https://crates.io/crates/kern/X.Y.Z
```

---

## Common Mistakes

**Skipping pre-flight:** Publishes broken code. Pre-flight is mandatory.

**Pushing before user confirmation:** Step 5 exists because push + publish are irreversible. Always confirm.

**Forgetting `cargo build` after version bump:** `Cargo.lock` won't match `Cargo.toml`, causing publish warnings.

**Not pushing the tag:** `git push origin main` alone doesn't push tags. Push the tag separately with `git push origin vX.Y.Z`.

**Publishing before pushing:** If publish succeeds but push fails, crates.io has the release but the repo doesn't have the tag. Always push first.
