# Facet
**Desktop-first jewelry repair intake + ticketing web app**

## Overview
Facet is a desktop-first web application for jewelry stores to intake repair jobs, capture photos, print a customer receipt plus a matching physical tag, and manage a clear work queue. The queue is FIFO by default, with a Rush flag for priority overrides. The system tracks status, pricing (quoted vs actual), storage location, and employee attribution for intake, work, and closure.

Core principles:
- Desktop-first, fast data entry
- Photo evidence at intake
- FIFO queue with a Rush override
- Employee attribution for accountability
- Printable receipts/tags to bind the physical item to the ticket
- Offline-capable intake (PWA) for internet outages

---

## Key Features
### MVP
- Repair ticket intake with required photos
- Printable receipt (PDF) plus physical label/tag (Ticket ID plus optional QR)
- Status workflow: Intake → In Progress → Waiting on Parts → Ready for Pickup → Closed
- Queue ordering: Rush first, then FIFO
- Quote amount plus actual charged at close
- Storage location tracking (bin, safe drawer, etc.)
- Employee attribution (intake / work / close / key edits)
- Audit trail (status changes, pricing edits, photo uploads)

### Later
- Drag-and-drop lane ordering (explicit rank)
- Customer notifications (email/SMS)
- Customer portal (“track my repair”)
- Multi-tenant (multiple stores)

---

## Architecture
High-level components:
- **Web UI (SvelteKit)**: queue view, intake form, ticket detail, admin
- **API (Rust Axum)**: authentication/authorization, ticket CRUD, print endpoints, audit trail
- **Postgres**: tickets, customers, employees, status history, notes, pricing
- **Object Storage (S3 compatible)**: photos and generated PDFs/labels (optional)
- **PWA Offline Layer**: local ticket creation plus background sync when online returns

Data flow:
1. Intake creates ticket plus uploads photos (or stores locally when offline).
2. Backend persists ticket and photo references.
3. Frontend requests print payload and renders receipt/label templates.
4. Queue pulls tickets by status lanes and sorts Rush plus FIFO.

---

## Tech Stack
- Frontend: **SvelteKit** (TypeScript recommended)
- Backend: **Rust** plus **Axum**
- Database: **Postgres**
- Storage: **S3-compatible** (DigitalOcean Spaces)
- Local dev orchestration: **Docker Compose**
- Migrations: **sqlx** (recommended) or Diesel (choose one and standardize)

---

## Repo Structure
Suggested structure (adjust as needed):

```
/
  apps/
    web/                # SvelteKit frontend
    api/                # Rust Axum backend
  packages/
    shared/             # shared types (optional), schemas, utilities
  infra/
    docker/             # docker compose, nginx, local configs
  docs/
    PRD.md              # detailed product requirements
  VISION.md             # project vision and feature scope
  AGENTS.md             # agent instructions for coding assistants
  README.md             # this file (dev setup)
```

---

## Local Development
### Prerequisites
- Docker plus Docker Compose
- Node.js (LTS)
- Rust toolchain (stable)
- (Optional) sqlx-cli for migrations

### Quick Start (Docker-first)
1. Create environment files:
   - `apps/api/.env`
   - `apps/web/.env`

2. Start dependencies:

```bash
docker compose up -d db
```

3. Run API:

```bash
cd apps/api
cargo run
```

4. Run Web:

```bash
cd apps/web
npm install
npm run dev
```

5. Open:
- Web: http://localhost:5173
- API: http://localhost:8080 (or configured port)

---

## Environment Variables
### API (`apps/api/.env`)
Required:
- `DATABASE_URL=postgres://facet:facet@localhost:5432/facet`
- `APP_ENV=development`
- `API_PORT=8080`

Photos / object storage:
- `S3_ENDPOINT=https://nyc3.digitaloceanspaces.com` (example)
- `S3_BUCKET=facet-photos-dev`
- `S3_ACCESS_KEY=...`
- `S3_SECRET_KEY=...`
- `S3_REGION=us-east-1` (some S3-compatible providers ignore, but keep it)

Security:
- `ADMIN_PIN_HASH=...` (hash, not plain text)
- `EMPLOYEE_PIN_MIN_LENGTH=4`
- `SIGNED_URL_TTL_SECONDS=300`

Printing:
- `RECEIPT_TEMPLATE=default`
- `LABEL_TEMPLATE=default`
- `STORE_NAME=Example Jewelers`
- `STORE_PHONE=...`
- `STORE_ADDRESS_LINE1=...`

### Web (`apps/web/.env`)
- `PUBLIC_API_BASE_URL=http://localhost:8080`
- `PUBLIC_APP_NAME=Facet`
- `PUBLIC_ENABLE_OFFLINE=true`

---

## Database and Migrations
Recommended approach: **sqlx** migrations.

Example workflow:
1. Create migration:

```bash
cd apps/api
sqlx migrate add create_tickets
```

2. Apply migrations:

```bash
sqlx migrate run
```

Key tables (conceptual):
- `employees` (id, name, pin_hash, role)
- `tickets` (uuid, friendly_code, status, rush, promise_date, storage_location, quote, actual, etc.)
- `ticket_photos` (id, ticket_uuid, storage_key, uploaded_by, created_at)
- `ticket_status_history` (ticket_uuid, from_status, to_status, changed_by, changed_at)
- `ticket_notes` (ticket_uuid, note, created_by, created_at)

---

## Object Storage for Photos
Photos are stored in S3-compatible storage and referenced from Postgres.

Best practice:
- Upload via API to validate file type/size and attach `ticket_uuid`.
- Store originals, optionally generate thumbnails later.
- Serve images via signed URLs (especially important in multi-tenant future).

Constraints (suggested):
- Max photo size: 10 MB
- Allowed content types: image/jpeg, image/png, image/webp
- Max photos per ticket: 10 (configurable)

---

## Printing (Receipts and Tags)
### Requirements
- Receipt: PDF with Ticket ID, customer name, item summary, requested work, quote (optional), promise date (optional), created date/time, store header, and optional QR.
- Tag/Label: Ticket ID large plus optional QR plus short item descriptor.

### Implementation Options
1. **Browser-native printing (MVP-friendly)**
   - Web renders receipt/label HTML templates
   - Print via `window.print()` with print CSS
   - Pros: simple, no hardware coupling
   - Cons: printer variations, label printers can be fiddly

2. **Server-generated PDFs (more controlled)**
   - API generates PDFs (receipt/label)
   - Web opens PDF for printing
   - Pros: consistent output
   - Cons: more work

MVP suggestion:
- Use browser printing with strong print CSS.
- Add server PDF generation later if printer chaos becomes a recurring villain.

Label printers:
- If using thermal label printers (Zebra, Brother), expect template tuning.
- Consider supporting a fixed label size in settings (ex: 2x1 inch).

---

## Offline Mode (PWA)
Goal: allow intake creation (including photos) when internet is down.

Approach:
- Use IndexedDB to store:
  - draft tickets
  - photos (as blobs)
  - pending sync queue
- Ticket IDs generated client-side as UUID.
- Sync worker pushes tickets/photos when back online.
- Conflict policy (MVP):
  - server wins for existing ticket fields
  - offline-created tickets are appended
  - display “Sync issues” banner if any uploads fail

---

## Testing
### API
- Unit tests for:
  - queue ordering (rush plus FIFO)
  - status transitions and history creation
  - pricing edits audit
- Integration tests with Postgres (docker)

### Web
- Smoke tests for:
  - intake form required fields plus photo requirement
  - queue sorting behavior
  - printing template render

---

## Deployment (DigitalOcean)
### Phase 1: Single Droplet (simple, scalable enough early)
- Docker Compose services:
  - api
  - web (or static build served by nginx)
  - postgres
- Use DigitalOcean Spaces for photos
- Backups:
  - nightly Postgres dump plus store to Spaces (or DO backups)
  - retention policy (ex: 7 daily, 4 weekly)

### Phase 2: Split services
- Move Postgres to managed database
- Move web to DO App Platform or CDN
- Keep API as container service
- Add centralized logging (Grafana/Loki or a managed solution)

---

## Security Notes
- Never store PINs in plain text. Store salted hashes.
- Use TLS in production.
- Validate and sanitize file uploads.
- Maintain audit trails for critical actions.
- If multi-tenant is introduced:
  - strict tenant scoping in every query
  - signed URLs with short TTL
  - per-tenant buckets or key prefixes

---

## Roadmap
### MVP
- Intake plus photos (required)
- Receipt plus label printing
- FIFO plus rush queue
- Status transitions plus audit trail
- Quote plus actual
- Employee attribution via ID/PIN entry
- Offline intake (PWA)

### Phase 2
- Drag-and-drop ordering within lanes
- Customer notifications (email/SMS)
- Better reporting (throughput, overdue, estimate vs actual)

### Phase 3
- Multi-tenant productization
- Customer portal
- POS integrations (optional)

