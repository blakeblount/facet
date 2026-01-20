# UI Test Results

This file tracks the results of UI/UX testing performed by coding agents using Playwright.

## Instructions for Agents

### Server Setup (IMPORTANT)
**DO NOT start or stop servers for individual tests.**

The setup task `facet-fo1` starts servers once and leaves them running. If servers are already running, skip setup and proceed with tests.

- **Web URL:** http://localhost:5173
- **API URL:** http://localhost:3001

If servers are NOT running:
1. Run `bd show facet-fo1` and follow the setup instructions
2. Start servers in BACKGROUND mode so they persist
3. **NEVER shut down servers** - leave them for other tests

### Before Running Any Test
1. Verify servers are accessible (navigate to http://localhost:5173)
2. If servers are down, follow setup task first
3. Ensure the database has seed data

### Running a Test
1. Pick a test issue from `bd list --label=ui-test`
2. Run `bd update <id> --status=in_progress` to claim it
3. Use Playwright to execute each step
4. Document results below using the template
5. Run `bd close <id>` when complete (pass or fail - we need the data)

### CRITICAL: Do Not Close Servers
**NEVER run commands that stop, kill, or restart the dev servers.**
- Do not use `pkill`, `kill`, or Ctrl+C on server processes
- Do not close terminal sessions running servers
- If you accidentally stop a server, restart it immediately
- Servers should remain running across ALL test sessions

### Recording Results
After completing each test, append results to this file using this format:

```markdown
---

## TEST: [Issue ID] - [Test Name]
**Date:** YYYY-MM-DD
**Status:** PASS | FAIL | BLOCKED
**Agent:** [agent identifier if available]

### Steps Executed
1. [Step taken and what happened]
2. [Step taken and what happened]
...

### Success Criteria Results
- [ ] Criterion 1: PASS/FAIL - [notes]
- [ ] Criterion 2: PASS/FAIL - [notes]
...

### Screenshots
- [filename if captured]

### Issues Found
- [Issue description, severity, suggested fix]

### Notes
[Any additional observations, edge cases discovered, etc.]
```

### Severity Levels for Issues
- **CRITICAL**: Blocks core functionality, data loss, security issue
- **HIGH**: Major feature broken, poor UX, workflow blocked
- **MEDIUM**: Feature works but has problems, minor UX issues
- **LOW**: Cosmetic, minor inconvenience, edge case

---

# Test Results

<!-- Append test results below this line -->

---

## TEST: facet-fo1 - SETUP - Start dev servers
**Date:** 2026-01-20
**Status:** PASS
**Agent:** Claude Opus 4.5

### Steps Executed
1. Checked if servers were already running - port 3000 had beads-ui, not our API
2. Started Docker Desktop (was not running)
3. Started postgres database with `docker compose up -d db`
4. Created .env file for API with correct database credentials
5. Verified database schema and seed data already present
6. Started Rust API server with `cargo run --bin api` in background (port 3001)
7. Started web server with `npm run dev` in background (port 5173)
8. Verified API responds at http://localhost:3001/api/v1/queue with 2 test tickets
9. Verified web server responds at http://localhost:5173
10. Used Playwright to navigate to workboard - confirmed tickets display correctly

### Success Criteria Results
- [x] API server is running and responding - PASS (port 3001)
- [x] Web server is running and accessible - PASS (port 5173)
- [x] Can load the workboard page in browser - PASS
- [x] Servers are running in background (not blocking) - PASS

### Screenshots
- None captured

### Issues Found
- None

### Notes
- API runs on port 3001 (not 3000 as may be documented elsewhere)
- Port 3000 is used by beads-ui which is a separate tool
- The vite.config.ts correctly proxies /api to localhost:3001
- Initial page load showed "Offline" indicator and cached data, but clicking "Reload" fetched fresh data
- Database has 2 test tickets already seeded: JR-0001 and JR-0002

