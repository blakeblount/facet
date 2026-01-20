# Facet PRD
**Document type:** Product Requirements Document
**Version:** 0.1 (MVP spec)
**Primary platform:** Desktop-first web app (responsive; expandable to mobile later)

---

## 1. Summary
A desktop-first web application for jewelry stores to intake repair jobs, capture photos, print a customer receipt + matching physical tag, and manage a clear work queue. The queue is FIFO by default, with a Rush flag for priority overrides. The system tracks status, pricing (quoted vs actual), storage location, and employee attribution for intake, work, and closure.

---

## 2. Problem Statement
Jewelry repair intake and tracking is often handled via paper forms, spreadsheets, or informal notes. This leads to:
- Missing intake details (condition, requested work, storage location)
- Disputes due to lack of photo evidence
- Poor visibility into work-in-progress
- Difficulty handling rush jobs while maintaining FIFO context
- No accountability trail (who took in, worked, closed)

---

## 3. Goals
### MVP Goals
- Capture complete repair intake details including **photos**
- Print **receipt + matching physical tag** using Ticket ID
- Provide a clear queue by status with **FIFO + Rush**
- Track key statuses from intake to closure
- Record pricing: **quote** and **actual charged**
- Track storage location (where the item physically is)
- Attribute actions to employees (intake / work / close / edits)
- Support shared front-desk workstation usage

### Non-Goals (MVP)
- POS integration, payments, inventory for retail items
- Customer notification (email/SMS) and customer portal
- Advanced technician time tracking, payroll, or job costing
- Multi-location support / multi-tenant productization (planned later)

---

## 4. Users and Roles
### User Types
- **Front Desk Staff (Kiosk/Shared Mode):**
  - Create tickets, capture photos, print receipt/tag
  - Update statuses, add notes, assign “worked by”
- **Technician / Jeweler:**
  - Update status, add notes, record work completion details
- **Manager/Admin (Protected Actions):**
  - Configure store settings (statuses, printers, employee list)
  - Perform sensitive edits (delete photos, edit closed tickets, hard deletes)

### Access Model (MVP)
- Primary workstation runs in **Shared Mode** (no per-user login required).
- Employee attribution is captured via **Employee ID entry** (PIN or short code).
- Admin functions gated behind **Admin PIN/password**.

---

## 5. Core Workflow
### 5.1 Intake (Create Ticket)
1. Staff begins a new ticket.
2. Staff enters:
   - customer info
   - item description and condition
   - requested work
   - optional promise date
   - optional quote
   - storage location
3. Staff captures **photos (minimum 1 required)**.
4. Staff enters **Employee ID** for “Taken In By”.
5. System generates Ticket ID.
6. System prints:
   - **Customer receipt** (PDF)
   - **Physical tag** (label) with matching Ticket ID

### 5.2 Queue Management (Workboard)
- Tickets appear in status lanes.
- Default ordering: **Rush first**, then **FIFO** within each lane.
- Staff can move tickets between statuses.
- Staff can mark/unmark Rush.
- Staff enters Employee ID when performing key actions (status change, rush toggle, close).

### 5.3 Work Updates
- Staff/technician adds internal notes.
- Ticket can be assigned “Worked By” employee (one or more, MVP supports primary).
- Optional: record “work performed” summary.

### 5.4 Completion and Pickup
- Ticket moved to **Ready for Pickup**.
- At pickup:
  - Staff verifies receipt Ticket ID matches the physical tag.
  - Staff enters Employee ID for “Closed By”.
  - Staff enters **Actual Charged** (required to close; can be 0).
  - Ticket moved to **Closed**.

---

## 6. Status Model (MVP)
Starter statuses:
1. **Intake**
2. **In Progress**
3. **Waiting on Parts**
4. **Ready for Pickup**
5. **Closed**

Notes:
- Status changes should record timestamp and employee attribution.
- Closed tickets are read-only except via Admin override.

---

## 7. Priority and Queue Ordering
### MVP Priority Logic
Within each status lane, sort by:
1. `is_rush` (true first)
2. `created_at` ascending (FIFO)

### Phase 2 (Planned)
- Add **drag-and-drop ordering** within each status lane.
- Persist an explicit `queue_rank` / `sort_index`.
- Maintain FIFO as the initial rank assignment at creation.

---

## 8. Data Capture Requirements
### 8.1 Ticket Fields (MVP)
**Identity & timestamps**
- `ticket_id` (UUID, offline-capable)
- `friendly_code` (human-friendly short code, e.g., JR-9F3K2)
- `created_at`, `updated_at`
- `closed_at` (nullable)

**Customer**
- `customer_name` (required)
- `customer_phone` (optional)
- `customer_email` (optional)

**Item & condition**
- `item_type` (optional select: ring, necklace, bracelet, etc.)
- `item_description` (required)
- `condition_notes` (required)
- `requested_work` (required)

**Operational**
- `status` (required)
- `is_rush` (default false)
- `promise_date` (optional)
- `storage_location` (recommended, required in MVP if practical)

**Pricing**
- `quote_amount` (optional)
- `actual_amount` (required at close; nullable until closure)
- `currency` (default store setting)

**Employee Attribution**
- `taken_in_by_employee_id` (required)
- `worked_by_employee_id` (optional; MVP primary worker)
- `closed_by_employee_id` (required at close)
- `last_modified_by_employee_id` (captured on edits/status changes)

### 8.2 Photos (MVP)
- Minimum: **1 photo required**.
- Allow up to 10 photos per ticket (configurable).
- Photos stored as objects; ticket stores references.
- Capture:
  - `photo_id`, `ticket_id`, `uploaded_at`
  - `uploaded_by_employee_id`
  - `storage_key` / URL reference
  - `content_type`, `size_bytes`

---

## 9. Printing Requirements
### 9.1 Customer Receipt (PDF)
Must include:
- Ticket ID (friendly code + UUID/QR optional)
- Customer name
- Item summary (description)
- Requested work summary
- Quote amount (if present)
- Promise date (if present)
- Date/time created
- Store name/contact (configurable)
- **QR code** that opens ticket details (recommended)

### 9.2 Physical Tag / Label
Must include:
- Ticket friendly code (large)
- Optional QR code
- Short item descriptor (truncate)
- Optional status or created date

Security intent:
- Customer must present receipt to retrieve item.
- Staff verifies receipt ID matches physical tag ID.

---

## 10. Employee Identification Model (Shared Workstation)
### MVP Approach
- Maintain an **Employee list** (name + short Employee ID/PIN).
- Shared workstation actions require entry of Employee ID for:
  - Ticket intake creation completion
  - Status change
  - Rush toggle
  - Closing ticket
  - Editing critical fields (condition, requested work, pricing)

### UX Guidance
- Use a quick modal: “Enter Employee ID” (with fast keypad input).
- Cache the last-used Employee ID for a short window (e.g., 5 minutes) to reduce friction, but allow quick switching.

### Admin vs Staff
- Staff actions: most ticket workflow operations.
- Admin actions: configuration + sensitive operations (delete photos, edit closed tickets, delete tickets).

---

## 11. Views and Screens (MVP)
1. **Queue / Workboard**
   - Lanes by status
   - Ticket cards show: friendly code, customer name, item short desc, Rush badge, promise date badge, thumbnail
   - Quick actions: move status, toggle rush, open detail

2. **New Ticket Form**
   - Structured fields + photo capture upload
   - Print receipt/tag after save
   - Requires Employee ID confirmation

3. **Ticket Detail**
   - Full ticket data + photos gallery
   - Notes timeline
   - Status history (who/when)
   - Pricing: quote + actual
   - Print buttons (receipt/tag reprint)

4. **Search**
   - Search by customer name, phone/email, ticket code, keywords
   - Filter by status and date range

5. **Admin Settings**
   - Employees management
   - Printer settings / label templates
   - Store info for receipt header
   - Optional: status names/order (future)

---

## 12. Audit Trail and History (MVP)
### Required History
- Status change history: `from_status`, `to_status`, timestamp, employee_id
- Rush toggle history: timestamp, employee_id
- Pricing changes (quote/actual): timestamp, employee_id
- Photo uploads: timestamp, employee_id

Purpose:
- Accountability
- Dispute handling
- Operational visibility

---

## 13. Offline Support (Desired for MVP)
### Requirement
If internet goes down, staff can still:
- Create ticket
- Capture photos
- Print receipt/tag
- Queue tickets locally

### Implementation Direction
- Use PWA capabilities + local storage (IndexedDB).
- Tickets created offline use UUID Ticket IDs immediately.
- Sync engine:
  - pushes tickets/photos when connection returns
  - resolves conflicts (MVP: “server wins”, with alerts for collisions)
- Printing works offline using locally generated PDF/label templates.

---

## 14. Technical Architecture (Scalable, DigitalOcean-friendly)
### Recommended Stack
- **Frontend:** SvelteKit (desktop-first UI; PWA support for offline)
- **Backend:** Rust (Axum API)
- **Database:** Postgres
- **Object Storage:** S3-compatible (DigitalOcean Spaces)
- **Deployment (Phase 1):** Single DigitalOcean Droplet with Docker Compose
- **Deployment (Scale):** Separate API + DB + Spaces; optionally DO App Platform later

### Security Notes
- Store PII securely; enforce TLS everywhere.
- Photos served via signed URLs (short-lived) when multi-tenant is introduced.
- Rate limits and basic abuse prevention for public endpoints.
- Admin functions protected by Admin credential and server-side authorization.

---

## 15. MVP Scope Checklist
### Included
- Ticket intake with required photos
- Receipt + tag printing
- FIFO queue with Rush flag
- Status transitions and notes
- Quote + actual charged
- Employee attribution (intake / work / close / edits)
- Search and basic admin settings
- Offline-capable intake (desired)

### Excluded (Later)
- Customer notifications (email/SMS)
- Customer portal
- Payments/POS integrations
- Drag-and-drop reorder (Phase 2)
- Multi-location / multi-tenant productization (Phase 3+)

---

## 16. Success Metrics
- Intake completeness rate (required fields present, photos present)
- Average intake time (target: < 2 minutes)
- Reduction in “lost” items / unclear status
- Reduced disputes due to photo evidence + audit trail
- Visibility: staff can answer “where is it and what’s happening?” quickly

---

## 17. Open Questions (for later phases)
- Drag-and-drop reorder rules and permissions
- Multi-worker assignment and time tracking
- Quotes approval workflow (customer approval capture)
- Multi-tenant store separation, billing, and onboarding
- Integration with POS or accounting systems
