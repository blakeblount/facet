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

---

## TEST: facet-d3x - Single Photo Upload
**Date:** 2026-01-20
**Status:** PASS (with minor UI suggestions)
**Agent:** Claude Opus 4.5

### Steps Executed
1. Navigated to http://localhost:5173/
2. Clicked "+ New" button in Intake lane to open intake form modal
3. Verified the "Photos" section is visible with "Item Photos *" label
4. Created a test PNG image file (100x100 blue square)
5. Clicked the photo upload area - file picker opened successfully
6. Selected test-image.png via Playwright file upload
7. Observed the uploaded photo appeared as a thumbnail with filename
8. Captured screenshot showing successful upload

### Success Criteria Results
- [x] Photo upload area is clearly marked - PASS - "Photos" heading with "Item Photos *" label
- [x] Shows accepted file types (JPG, PNG, WebP) - PARTIAL - Shows "Up to 10 images, max 10.0 MB each" but doesn't list specific file types. Component accepts `image/*`
- [x] Shows min/max requirements (1-10 photos) - PASS - Shows "Up to 10 images, max 10.0 MB each"
- [x] File picker opens when clicking upload area - PASS - File chooser dialog opened
- [x] Selected photo shows preview/thumbnail - PASS - Blue test image displayed as thumbnail
- [ ] Photo count updates to show "1 photo" - FAIL - No photo count indicator shown, only the preview grid
- [ ] Upload progress indicator shows (if uploading) - N/A - Files are processed locally via FileReader; actual server upload happens on form submission

### Screenshots
- .playwright-mcp/test-photo-upload-success.png

### Issues Found
- **LOW**: No explicit photo count indicator (e.g., "1 of 10 photos"). Users must count thumbnails visually.
- **LOW**: Accepted file types not explicitly listed in the UI (says "images" but not "JPG, PNG, WebP")

### Notes
- The PhotoUpload component is well-designed with drag-and-drop support, preview thumbnails, and remove buttons
- The required `*` indicator correctly shows this is a mandatory field
- Error validation exists for file size limits and file type restrictions
- Component handles multiple files up to maxFiles limit
- No upload progress needed for local file selection; progress would be relevant during form submission

---

## TEST: facet-prd - Workboard loads with all 4 status lanes
**Date:** 2026-01-20
**Status:** PASS
**Agent:** Claude Opus 4.5

### Steps Executed
1. Navigated to http://localhost:5173/
2. Page loaded successfully with title "Facet - Jewelry Repair Tracking"
3. Observed the workboard layout with 4 horizontal status lanes
4. Verified all expected elements were present and functional

### Success Criteria Results
- [x] Page loads without errors - PASS - No console errors detected
- [x] 4 status lanes are visible: "Intake", "In Progress", "Waiting on Parts", "Ready for Pickup" - PASS - All 4 lanes displayed
- [x] Each lane has a header with the status name - PASS - Headers show INTAKE, IN PROGRESS, WAITING ON PARTS, READY FOR PICKUP
- [x] Each lane header shows a count badge with number of tickets - PASS - Intake shows "2", others show "0"
- [x] Lanes are arranged horizontally - PASS - Lanes display side-by-side in a row
- [x] "+New" button is visible in the Intake lane - PASS - Button visible with "+ New" label

### Screenshots
- .playwright-mcp/workboard-lanes-test.png

### Issues Found
- None

### Notes
- Workboard shows 2 tickets in Intake lane (JR-0001 John Smith Gold wedding band, JR-0002 Test Customer Gold ring)
- Other lanes show "No tickets" placeholder text when empty
- Header navigation includes Workboard, Search, and Settings links
- Global search box available in header
- Page description correctly states "Manage repair tickets across status lanes. Rush tickets appear first in each lane."

---

## TEST: facet-2od - Intake form validation - required fields
**Date:** 2026-01-20
**Status:** PASS
**Agent:** Claude Opus 4.5

### Steps Executed
1. Navigated to http://localhost:5173/
2. Clicked "+ New" button to open intake form modal
3. Submitted empty form by triggering form submit event
4. Observed 6 validation error messages appeared
5. Filled Customer Name field with "Test Customer"
6. Submitted form - Customer Name error cleared, 5 errors remained
7. Filled Item Description with "Gold ring with diamond"
8. Submitted form - Item Description error cleared, 4 errors remained
9. Filled Condition Notes with "Minor scratches on band"
10. Filled Requested Work with "Polish and resize"
11. Submitted form - Both text field errors cleared, 2 errors remained
12. Selected "Safe Drawer 1" from Storage Location dropdown
13. Submitted form - Storage Location error cleared, 1 error remained
14. Uploaded test photo via file picker
15. Submitted form - All validation passed, Employee PIN modal appeared

### Success Criteria Results
- [x] Empty form submission is prevented - PASS - Form submit blocked, errors shown
- [x] Customer Name required error shown when empty - PASS - "Customer name is required"
- [x] Item Description required error shown when empty - PASS - "Item description is required"
- [x] Condition Notes required error shown when empty - PASS - "Condition notes are required"
- [x] Requested Work required error shown when empty - PASS - "Requested work is required"
- [x] Storage Location required error shown when not selected - PASS - "Storage location is required"
- [x] Photo required error shown when no photos uploaded - PASS - "At least one photo is required"
- [x] Error messages are clear and specific to each field - PASS - Each message clearly states which field is required
- [x] Errors clear when field is properly filled - PASS - Verified each error clears individually after filling and resubmitting

### Screenshots
- .playwright-mcp/intake-form-validation-empty.png - Shows validation errors after empty submit
- .playwright-mcp/intake-form-validation-tall.png - Full form with errors visible
- .playwright-mcp/intake-form-validation-passed.png - Employee PIN modal after validation passes

### Issues Found
- None - Validation works correctly

### Notes
- Validation only triggers on form submit (not real-time as user types)
- All error messages use `role="alert"` for accessibility
- The form correctly validates 6 required fields: Customer Name, Item Description, Condition Notes, Requested Work, Storage Location, and Photos
- Phone and Email fields are optional (not validated as required)
- Item Type field is optional (not validated as required)
- Quote Amount field is optional but validates format if provided
- When all fields are valid, form proceeds to Employee PIN verification step

