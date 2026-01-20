# Facet API Specification

**Version:** 0.1
**Base URL:** `/api/v1`

---

## Overview

REST API with JSON payloads. All endpoints return consistent response format:

```json
// Success
{ "data": { ... }, "error": null }

// Error
{ "data": null, "error": { "code": "NOT_FOUND", "message": "Ticket not found" } }
```

### Authentication

MVP uses **Employee PIN** for attribution, not session-based auth:
- Most endpoints require `X-Employee-ID` header (employee UUID)
- Admin endpoints require `X-Admin-PIN` header
- PIN verification happens via `/employees/verify` before actions

### Common Parameters

- Pagination: `?limit=50&offset=0`
- Sorting: `?sort=created_at&order=desc`

---

## Endpoints

### Tickets

#### List Tickets
```
GET /tickets
```

Query parameters:
| Param | Type | Description |
|-------|------|-------------|
| `status` | string | Filter by status (intake, in_progress, waiting_on_parts, ready_for_pickup, closed, archived) |
| `is_rush` | boolean | Filter rush tickets only |
| `search` | string | Full-text search across ticket fields, customer, notes |
| `customer_id` | uuid | Filter by customer |
| `from_date` | date | Created after this date |
| `to_date` | date | Created before this date |
| `include_archived` | boolean | Include archived tickets (default: false) |

Response:
```json
{
  "data": {
    "tickets": [
      {
        "ticket_id": "uuid",
        "friendly_code": "JR-0001",
        "status": "in_progress",
        "is_rush": false,
        "is_overdue": true,
        "customer": {
          "customer_id": "uuid",
          "name": "Jane Doe",
          "phone": "555-1234"
        },
        "item_type": "ring",
        "item_description": "Gold band with diamond",
        "promise_date": "2026-01-25",
        "quote_amount": 150.00,
        "storage_location": {
          "location_id": "uuid",
          "name": "Safe Drawer 1"
        },
        "photo_count": 3,
        "thumbnail_url": "https://...",
        "created_at": "2026-01-19T10:30:00Z",
        "taken_in_by": {
          "employee_id": "uuid",
          "name": "Alice"
        }
      }
    ],
    "total": 42,
    "limit": 50,
    "offset": 0
  }
}
```

#### Get Ticket
```
GET /tickets/:ticket_id
```

Response includes full ticket details:
```json
{
  "data": {
    "ticket_id": "uuid",
    "friendly_code": "JR-0001",
    "status": "in_progress",
    "is_rush": false,
    "customer": {
      "customer_id": "uuid",
      "name": "Jane Doe",
      "phone": "555-1234",
      "email": "jane@example.com"
    },
    "item_type": "ring",
    "item_description": "Gold band with diamond",
    "condition_notes": "Minor scratches on band",
    "requested_work": "Resize from 7 to 6, polish",
    "promise_date": "2026-01-25",
    "storage_location": {
      "location_id": "uuid",
      "name": "Safe Drawer 1"
    },
    "quote_amount": 150.00,
    "actual_amount": null,
    "photos": [
      {
        "photo_id": "uuid",
        "url": "https://signed-url...",
        "uploaded_at": "2026-01-19T10:30:00Z",
        "uploaded_by": { "employee_id": "uuid", "name": "Alice" }
      }
    ],
    "notes": [
      {
        "note_id": "uuid",
        "content": "Customer mentioned ring has sentimental value",
        "created_at": "2026-01-19T10:35:00Z",
        "created_by": { "employee_id": "uuid", "name": "Alice" }
      }
    ],
    "status_history": [
      {
        "from_status": "intake",
        "to_status": "in_progress",
        "changed_at": "2026-01-19T14:00:00Z",
        "changed_by": { "employee_id": "uuid", "name": "Bob" }
      }
    ],
    "taken_in_by": { "employee_id": "uuid", "name": "Alice" },
    "worked_by": { "employee_id": "uuid", "name": "Bob" },
    "closed_by": null,
    "created_at": "2026-01-19T10:30:00Z",
    "updated_at": "2026-01-19T14:00:00Z",
    "closed_at": null
  }
}
```

#### Create Ticket
```
POST /tickets
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)

Request:
```json
{
  "customer": {
    "customer_id": "uuid",       // existing customer, OR
    "name": "Jane Doe",          // new customer (inline creation)
    "phone": "555-1234",
    "email": "jane@example.com"
  },
  "item_type": "ring",
  "item_description": "Gold band with diamond",
  "condition_notes": "Minor scratches on band",
  "requested_work": "Resize from 7 to 6, polish",
  "promise_date": "2026-01-25",
  "storage_location_id": "uuid",
  "quote_amount": 150.00,
  "is_rush": false
}
```

Response:
```json
{
  "data": {
    "ticket_id": "uuid",
    "friendly_code": "JR-0001",
    "status": "intake",
    "print_data": {
      "receipt_url": "/tickets/uuid/receipt.pdf",
      "label_url": "/tickets/uuid/label.pdf"
    }
  }
}
```

Notes:
- Returns print URLs for receipt and label
- Client must successfully print before considering intake complete
- If `customer.customer_id` provided, links to existing customer
- If customer fields provided without ID, creates new customer inline

#### Update Ticket
```
PUT /tickets/:ticket_id
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)

Request (partial update):
```json
{
  "item_description": "Updated description",
  "quote_amount": 175.00,
  "storage_location_id": "uuid",
  "worked_by_employee_id": "uuid"
}
```

Restrictions:
- Cannot update closed/archived tickets (returns 403)
- Admin override: include `X-Admin-PIN` header to edit closed tickets

#### Update Ticket Status
```
POST /tickets/:ticket_id/status
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)

Request:
```json
{
  "status": "in_progress"
}
```

Notes:
- Creates status history entry automatically
- Validates status transitions (e.g., cannot go from closed to in_progress)

#### Toggle Rush
```
POST /tickets/:ticket_id/rush
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)

Request:
```json
{
  "is_rush": true
}
```

#### Close Ticket
```
POST /tickets/:ticket_id/close
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)

Request:
```json
{
  "actual_amount": 145.00
}
```

Notes:
- `actual_amount` required (can be 0)
- Sets status to "closed" and records `closed_at`, `closed_by`

#### Get Receipt PDF
```
GET /tickets/:ticket_id/receipt.pdf
```

Returns PDF binary with appropriate content-type.

#### Get Label PDF
```
GET /tickets/:ticket_id/label.pdf
```

Returns PDF binary for physical tag printing.

---

### Photos

#### Upload Photo
```
POST /tickets/:ticket_id/photos
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)
- `Content-Type: multipart/form-data`

Request: multipart form with `file` field

Response:
```json
{
  "data": {
    "photo_id": "uuid",
    "url": "https://signed-url...",
    "uploaded_at": "2026-01-19T10:30:00Z"
  }
}
```

Constraints:
- Max file size: 10 MB
- Allowed types: image/jpeg, image/png, image/webp
- Max 10 photos per ticket

#### Delete Photo
```
DELETE /tickets/:ticket_id/photos/:photo_id
```

Headers:
- `X-Admin-PIN: <pin>` (required - admin only)

---

### Notes

#### Add Note
```
POST /tickets/:ticket_id/notes
```

Headers:
- `X-Employee-ID: <employee_uuid>` (required)

Request:
```json
{
  "content": "Customer called, requested expedited completion"
}
```

Notes are append-only; no edit or delete endpoints.

---

### Customers

#### Search Customers
```
GET /customers
```

Query parameters:
| Param | Type | Description |
|-------|------|-------------|
| `search` | string | Search by name, phone, or email |

Response:
```json
{
  "data": {
    "customers": [
      {
        "customer_id": "uuid",
        "name": "Jane Doe",
        "phone": "555-1234",
        "email": "jane@example.com",
        "ticket_count": 5
      }
    ]
  }
}
```

Used for autocomplete during ticket creation.

#### Get Customer
```
GET /customers/:customer_id
```

Response includes customer details and ticket history:
```json
{
  "data": {
    "customer_id": "uuid",
    "name": "Jane Doe",
    "phone": "555-1234",
    "email": "jane@example.com",
    "created_at": "2025-06-15T09:00:00Z",
    "tickets": [
      {
        "ticket_id": "uuid",
        "friendly_code": "JR-0001",
        "status": "closed",
        "item_description": "Gold band",
        "created_at": "2026-01-10T10:00:00Z"
      }
    ]
  }
}
```

---

### Employees

#### Verify Employee PIN
```
POST /employees/verify
```

Request:
```json
{
  "pin": "1234"
}
```

Response:
```json
{
  "data": {
    "employee_id": "uuid",
    "name": "Alice",
    "role": "staff"
  }
}
```

Error if PIN invalid:
```json
{
  "data": null,
  "error": { "code": "INVALID_PIN", "message": "Invalid employee PIN" }
}
```

#### List Employees
```
GET /employees
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Query parameters:
| Param | Type | Description |
|-------|------|-------------|
| `include_inactive` | boolean | Include deactivated employees (default: false) |

Response:
```json
{
  "data": {
    "employees": [
      {
        "employee_id": "uuid",
        "name": "Alice",
        "role": "staff",
        "is_active": true,
        "created_at": "2025-01-01T00:00:00Z"
      }
    ]
  }
}
```

#### Create Employee
```
POST /employees
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Request:
```json
{
  "name": "Charlie",
  "pin": "5678",
  "role": "staff"
}
```

#### Update Employee
```
PUT /employees/:employee_id
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Request:
```json
{
  "name": "Charlie Updated",
  "is_active": false
}
```

#### Delete Employee
```
DELETE /employees/:employee_id
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Notes:
- Warns if employee has attribution history
- Consider deactivation instead to preserve history

---

### Storage Locations

#### List Locations
```
GET /locations
```

Query parameters:
| Param | Type | Description |
|-------|------|-------------|
| `include_inactive` | boolean | Include deactivated locations (default: false) |

Response:
```json
{
  "data": {
    "locations": [
      {
        "location_id": "uuid",
        "name": "Safe Drawer 1",
        "is_active": true
      }
    ]
  }
}
```

#### Create Location
```
POST /locations
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Request:
```json
{
  "name": "Workbench B"
}
```

#### Update Location
```
PUT /locations/:location_id
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Request:
```json
{
  "name": "Workbench B (Updated)",
  "is_active": false
}
```

---

### Store Settings

#### Get Settings
```
GET /settings
```

Response:
```json
{
  "data": {
    "store_name": "Example Jewelers",
    "store_phone": "555-0000",
    "store_address": "123 Main St",
    "ticket_prefix": "JR",
    "currency": "USD",
    "max_photos_per_ticket": 10
  }
}
```

#### Update Settings
```
PUT /settings
```

Headers:
- `X-Admin-PIN: <pin>` (required)

Request:
```json
{
  "store_name": "Updated Jewelers",
  "ticket_prefix": "UJ"
}
```

---

### Admin

#### Verify Admin PIN
```
POST /admin/verify
```

Request:
```json
{
  "pin": "admin-pin"
}
```

Used to unlock admin functions in UI.

#### Change Admin PIN
```
POST /admin/change-pin
```

Headers:
- `X-Admin-PIN: <current_pin>` (required)

Request:
```json
{
  "new_pin": "new-secure-pin"
}
```

#### Force Password Change (First Login)
```
POST /admin/setup
```

Request:
```json
{
  "current_pin": "changeme",
  "new_pin": "secure-new-pin"
}
```

Returns error if setup already completed.

---

### Queue (Workboard)

#### Get Queue
```
GET /queue
```

Returns tickets grouped by status for workboard display:

```json
{
  "data": {
    "lanes": {
      "intake": {
        "count": 3,
        "tickets": [...]
      },
      "in_progress": {
        "count": 5,
        "tickets": [...]
      },
      "waiting_on_parts": {
        "count": 2,
        "tickets": [...]
      },
      "ready_for_pickup": {
        "count": 4,
        "tickets": [...]
      }
    }
  }
}
```

Notes:
- Excludes closed/archived tickets
- Each lane sorted by: rush first, then FIFO
- Tickets include `is_overdue` flag for visual indicator

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid request data |
| `INVALID_PIN` | 401 | Employee or admin PIN incorrect |
| `FORBIDDEN` | 403 | Action not allowed (e.g., edit closed ticket) |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Conflict (e.g., duplicate friendly_code) |
| `PHOTO_LIMIT` | 422 | Max photos per ticket reached |
| `PRINT_REQUIRED` | 422 | Cannot complete action until print succeeds |
| `SERVER_ERROR` | 500 | Internal server error |

---

## Offline Sync (Future Detail)

For PWA offline support, the following sync endpoints will be added:

```
POST /sync/push    # Push locally created/modified tickets
POST /sync/pull    # Pull updates since last sync timestamp
```

Conflict resolution: server wins, client notified of overwrites.
