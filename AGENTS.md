# Agent Instructions

## Project Overview

Facet is a jewelry repair intake and ticketing web app. Desktop-first, photo-required intake, FIFO queue with Rush override, employee attribution for accountability. See `VISION.md` for feature scope and `docs/PRD.md` for detailed requirements.

## Tech Stack

- **Frontend:** SvelteKit (TypeScript)
- **Backend:** Rust + Axum
- **Database:** Postgres
- **Storage:** S3-compatible (DigitalOcean Spaces)
- **Dev:** Docker Compose
- **Migrations:** sqlx

## Repo Structure

```
apps/
  web/              # SvelteKit frontend
  api/              # Rust Axum backend
packages/
  shared/           # Shared types, schemas, utilities
infra/
  docker/           # Docker compose, nginx, local configs
docs/
  PRD.md            # Detailed product requirements
```

## Quality Gates

Run these before every commit:

```bash
# Frontend (apps/web)
cd apps/web && npm run lint && npm run check && npm test

# Backend (apps/api)
cd apps/api && cargo fmt --check && cargo clippy && cargo test

# Full check
docker compose up -d db && cargo test && npm test
```

## Issue Tracking

This project uses beads for issue tracking.

```bash
bd ready                              # Find available work
bd show <id>                          # View issue details
bd update <id> --status in_progress   # Claim work
bd close <id>                         # Complete work
bd create --title="..." --type=task   # Create new issue
```

## Code Conventions

- Use TypeScript strict mode in frontend
- Rust: follow clippy lints, use `?` for error propagation
- API responses: consistent JSON structure with `data`, `error` fields
- Database: snake_case for columns, use UUIDs for primary keys
- Tickets use `friendly_code` (e.g., JR-9F3K2) for human display

## Key Domain Concepts

- **Ticket:** A repair job with customer info, item details, photos, status
- **Status flow:** Intake → In Progress → Waiting on Parts → Ready for Pickup → Closed
- **Queue order:** Rush tickets first, then FIFO by created_at
- **Employee attribution:** Every key action records who did it (via Employee ID/PIN)

## Important Files

- `VISION.md` - Project vision and feature scope
- `docs/PRD.md` - Full product requirements
- `apps/api/src/main.rs` - API entry point
- `apps/web/src/routes/` - SvelteKit routes
- `docker-compose.yml` - Local dev services

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
