# Agent Instructions

## Project Overview

<!-- Brief description of the project -->

## Tech Stack

<!-- Languages, frameworks, key dependencies -->

-

## Quality Gates

Run these before every commit:

```bash
# Example for Rust:
# cargo fmt && cargo clippy && cargo test

# Example for Node:
# npm run lint && npm run test

# Update with your project's commands:

```

## Issue Tracking

This project uses beads for issue tracking.

```bash
bd ready                          # Find available work
bd show <id>                      # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>                     # Complete work
bd create "description"           # Create new issue
```

## Code Conventions

<!-- Project-specific patterns, naming, structure -->

-

## Important Files

<!-- Key files an agent should know about -->

-

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
