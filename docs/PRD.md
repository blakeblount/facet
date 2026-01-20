# Facet PRD
**Document type:** Product Requirements Document
**Version:** 0.2 (MVP spec, clarifications added)
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
1. **Intake** - Item received, work not yet started
2. **In Progress** - Actively being worked on
3. **Waiting on Parts** - Blocked on external dependency
4. **Ready for Pickup** - Work complete, awaiting customer
5. **Closed** - Customer picked up, ticket complete
6. **Archived** - Closed tickets auto-archive for long-term storage

**Status Rules:**
- Status changes record timestamp and employee attribution
- Closed tickets are **read-only** except via Admin override
- **Closed tickets cannot be reopened** - if a customer returns with the same item for new work, create a new ticket
- Closed tickets automatically transition to **Archived** status (timing TBD, e.g., after 30 days)
- Archived tickets remain searchable but are hidden from the main queue view

**Editing Rules:**
- Most fields can be edited while ticket is open (Intake through Ready for Pickup)
- Once Closed, only Admin can edit
- Editing records `last_modified_by_employee_id` and timestamp in audit trail

---

## 7. Priority and Queue Ordering
### MVP Priority Logic
Within each status lane, sort by:
1. `is_rush` (true first)
2. `created_at` ascending (FIFO)

### Overdue Visual Indicator
Tickets past their `promise_date` display a **red visual indicator**:
- Ticket card turns red or has red border/flag
- Overdue badge visible on card
- Overdue tickets sort normally (no automatic priority bump) but are visually prominent

### Phase 2 (Planned)
- Add **drag-and-drop ordering** within each status lane.
- Persist an explicit `queue_rank` / `sort_index`.
- Maintain FIFO as the initial rank assignment at creation.

---

## 8. Data Capture Requirements

### 8.1 Customer Entity
Customers are stored as a **separate entity** from tickets to support repeat customers.

**Customer Fields**
- `customer_id` (UUID)
- `name` (required)
- `phone` (optional)
- `email` (optional)
- `created_at`

**Behavior**
- During ticket intake, staff can search for existing customers by name/phone/email
- If a match is found, auto-fill customer info and link ticket to existing customer
- If no match, a new customer record is created inline during ticket creation
- Customer creation should be seamless - no separate "create customer" step required

### 8.2 Ticket Fields (MVP)
**Identity & timestamps**
- `ticket_id` (UUID, offline-capable)
- `friendly_code` (human-friendly short code, e.g., JR-0001)
- `created_at`, `updated_at`
- `closed_at` (nullable)

**friendly_code generation:**
- Format: `{store_prefix}-{sequential_number}` (e.g., JR-0001, JR-0002)
- Store prefix is configurable in admin settings (default: "JR")
- Sequential number assigned server-side on ticket creation
- Offline-created tickets get friendly_code assigned on sync (use UUID until then)

**Customer Reference**
- `customer_id` (required, references customer entity)

**Item & condition**
- `item_type` (free text, optional)
- `item_description` (required)
- `condition_notes` (required)
- `requested_work` (required)

**Operational**
- `status` (required)
- `is_rush` (default false)
- `promise_date` (optional)
- `storage_location_id` (required, references managed storage locations list)

**Pricing**
- `quote_amount` (optional)
- `actual_amount` (required at close; nullable until closure)
- `currency` (default store setting)

**Employee Attribution**
- `taken_in_by_employee_id` (required)
- `worked_by_employee_id` (optional; MVP primary worker)
- `closed_by_employee_id` (required at close)
- `last_modified_by_employee_id` (captured on edits/status changes)

### 8.3 Storage Locations (Managed List)
Storage locations are a **managed list** maintained by admins.

- Admins can add new locations on-the-fly during ticket creation
- Regular staff can only select from existing locations
- Examples: "Safe Drawer 1", "Workbench A", "Display Case 3"
- Locations can be deactivated but not deleted (preserve history)

**Fields:**
- `location_id` (UUID)
- `name` (required, unique)
- `is_active` (default true)
- `created_at`

### 8.4 Photos (MVP)
- Minimum: **1 photo required**.
- Allow up to 10 photos per ticket (configurable).
- **File upload only** for MVP (camera API integration later).
- Photos stored as objects; ticket stores references.
- Capture:
  - `photo_id`, `ticket_id`, `uploaded_at`
  - `uploaded_by_employee_id`
  - `storage_key` / URL reference
  - `content_type`, `size_bytes`

### 8.5 Notes (Internal Only)
Notes are **internal only** - not visible to customers.

**Fields:**
- `note_id` (UUID)
- `ticket_id` (required)
- `content` (text, required)
- `created_by_employee_id` (required)
- `created_at`

Notes are append-only; existing notes cannot be edited or deleted (audit trail).

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

**Print Failure Handling:**
- Ticket **cannot be saved** until print succeeds (receipt + tag are the physical claim check)
- If print fails, display error with retry option
- Staff must resolve print issue before completing intake
- For reprints (from ticket detail), failure does not affect ticket state

**Security intent:**
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
- Use a quick modal: "Enter Employee ID" (with fast keypad input).
- **After completing an action**, return to "Enter Employee ID" state (no persistent session).
- This ensures each action is attributed to whoever is at the workstation at that moment.

### Employee Lifecycle
- Employees can be **deactivated** (not deleted) to preserve attribution history
- Deactivated employees cannot perform actions but their past actions remain visible
- Hard delete option available but buried in admin UI (requires confirmation, warns about history loss)

**Employee Fields:**
- `employee_id` (UUID)
- `name` (required)
- `pin` (short code, hashed)
- `role` (staff | admin)
- `is_active` (default true)
- `created_at`

### Admin vs Staff
- Staff actions: most ticket workflow operations.
- Admin actions: configuration + sensitive operations (delete photos, edit closed tickets, delete tickets).

### Admin Bootstrap
- System ships with a **default admin account** (admin / changeme or similar)
- On first login, admin is **forced to change password**
- This allows initial store setup without external provisioning

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
   - **Full-text search** across all relevant fields:
     - Customer name, phone, email
     - Ticket friendly_code
     - Item description, condition notes, requested work
     - Notes content
   - Filter by status (including Archived) and date range
   - Search should be thorough - partial matches, case-insensitive
   - Archived tickets included in search results (marked as archived)

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
- `friendly_code` assigned server-side on sync (display UUID or "Pending" until then).
- Printing works offline using locally generated PDF/label templates.

### Sync and Conflict Resolution
**New tickets created offline:**
- Sync automatically when connection returns
- Assigned `friendly_code` on server
- No conflict possible (new records)

**Edits to existing tickets while offline:**
- If the same ticket was edited online by someone else, **server wins**
- User sees toast notification: "This ticket was updated while you were offline. Your changes were not saved."
- User can view current ticket state and re-enter changes if needed
- At ~30 tickets/day on a single workstation, this scenario is rare

**Expected Volume:**
- ~30 tickets per day
- Single workstation (shared mode)
- Offline conflicts expected to be rare

---

## 14. Technical Architecture (Scalable, DigitalOcean-friendly)
### Recommended Stack
- **Frontend:** SvelteKit (desktop-first UI; PWA support for offline)
- **Backend:** Rust (Axum API)
- **Database:** Postgres
- **Object Storage:** S3-compatible (DigitalOcean Spaces)
- **Deployment (Phase 1):** Single DigitalOcean Droplet with Docker Compose
- **Deployment (Scale):** Separate API + DB + Spaces; optionally DO App Platform later

### API Design
- **REST API** with JSON payloads
- Standard HTTP methods (GET, POST, PUT, DELETE)
- Consistent response format: `{ "data": {...}, "error": null }` or `{ "data": null, "error": {...} }`
- Employee ID passed in request header or body for attribution
- Endpoints to be defined in separate API spec document

### Security Notes
- Store PII securely; enforce TLS everywhere.
- Photos served via signed URLs (short-lived) when multi-tenant is introduced.
- Rate limits and basic abuse prevention for public endpoints.
- Admin functions protected by Admin credential and server-side authorization.
- Employee PINs stored as salted hashes, never plain text.

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
- Camera API integration for photo capture
- Label printer hardware compatibility
- Customer notification preferences (when Phase 2)
- Archive retention policy (how long to keep archived tickets)
