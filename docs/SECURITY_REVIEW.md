# Security Review: Facet API

> **Review Date**: January 2026
> **Last Updated**: January 2026
> **Scope**: Backend API security audit
> **Principle**: Treat the client as hostile; validate everything server-side

---

## Executive Summary

This security review evaluates the Facet API with the core assumption that **all client input is untrusted**. The API serves a jewelry repair shop management system with employee PIN authentication, ticket management, and photo uploads.

### Risk Summary

| Severity | Open | Fixed | Description |
|----------|------|-------|-------------|
| CRITICAL | 0 | 5 | Immediate deployment blockers |
| HIGH | 0 | 3 | Address within current sprint |
| MEDIUM | 2 | 2 | Address within next quarter |
| LOW | 2 | 0 | Long-term improvements |

### Key Findings (All Resolved)

1. ~~**No rate limiting** on authentication endpoints enables brute-force attacks~~ **FIXED**
2. ~~**Missing row-level authorization** allows any employee to modify any ticket~~ **FIXED**
3. ~~**CORS allows all origins** regardless of configuration~~ **FIXED**
4. ~~**Photo uploads lack magic byte validation** (Content-Type header only)~~ **FIXED**
5. ~~**No request body size limits** on API endpoints~~ **FIXED**

---

## Threat Model

### Attacker Profiles

| Profile | Capabilities | Goals |
|---------|--------------|-------|
| **Malicious User** | Valid employee credentials, browser access | Escalate privileges, access unauthorized tickets |
| **Compromised Client** | Modified frontend, arbitrary API calls | Bypass client-side validation, inject malicious data |
| **Network Attacker** | MITM position (if not HTTPS) | Credential theft, session hijacking |
| **Brute Force Bot** | Automated requests | PIN enumeration, credential stuffing |

### Attack Vectors

| Vector | Risk | Current Protection |
|--------|------|--------------------|
| Header manipulation | LOW | ✅ Session-based auth binds employee ID to session token |
| Body tampering | LOW | ✅ serde validates types + RBAC validates business logic |
| File upload exploits | LOW | ✅ Magic byte validation + Content-Type verification |
| Brute force auth | LOW | ✅ Rate limiting (5/min) + exponential backoff |
| SQL injection | LOW | ✅ Parameterized queries via sqlx |

### Assets at Risk

- Customer PII (name, phone, email)
- Employee credentials (PIN hashes)
- Ticket data and repair details
- Photo uploads (item documentation)
- Store configuration and settings

---

## Trust Boundaries

| Boundary | Source | Sink | Trust Level |
|----------|--------|------|-------------|
| **Client → API** | HTTP requests | Axum handlers | UNTRUSTED |
| **API → Database** | sqlx queries | PostgreSQL | TRUSTED (parameterized) |
| **API → S3** | File uploads | Object storage | TRUSTED (generated keys) |
| **Offline Cache → API** | IndexedDB sync | API endpoints | UNTRUSTED |

---

## Critical Issues (Deployment Blockers)

### CRIT-1: No Rate Limiting on PIN Verification ✅ FIXED

**Location**: `POST /api/v1/employees/verify`, `POST /api/v1/admin/verify`

**Original State**: Endpoints accepted unlimited requests with no throttling.

**Risk**: 4-digit PINs can be enumerated in ~10,000 attempts. At 100 req/sec, complete enumeration takes under 2 minutes.

**Resolution**: Implemented in `middleware/rate_limit.rs`:
- Per-IP rate limiting: 5 requests per minute via `governor` crate
- Exponential backoff after failures:
  - 1st failure: immediate retry
  - 2nd failure: 5 second wait
  - 3rd failure: 30 second wait
  - 4th+ failures: 5 minute wait
- Success resets backoff counter
- IP extraction from `X-Real-IP`, `X-Forwarded-For`, or socket address
- Comprehensive test coverage

---

### CRIT-2: Missing Row-Level Authorization ✅ FIXED

**Location**: All ticket handlers (`handlers/tickets.rs`)

**Original State**: `X-Employee-ID` header was validated to ensure the employee exists and is active, but there was no check that this employee has permission to modify the specific ticket.

**Affected Endpoints**:
- `PUT /api/v1/tickets/:ticket_id`
- `POST /api/v1/tickets/:ticket_id/status`
- `POST /api/v1/tickets/:ticket_id/close`
- `POST /api/v1/tickets/:ticket_id/rush`
- `POST /api/v1/tickets/:ticket_id/notes`
- `POST /api/v1/tickets/:ticket_id/photos`

**Risk**: Any authenticated employee could modify any ticket in the system by providing a valid employee ID and guessing/enumerating ticket UUIDs.

**Resolution**: Implemented in `middleware/rbac.rs`:
- Role-based access control (RBAC) with `Permission` enum
- Ownership model: `taken_in_by` or `worked_by` defines ticket ownership
- `require_ticket_access()` checks ownership before modification
- `is_ticket_owner()` utility for ownership checks
- Admin role bypasses ownership checks
- Staff role can only modify tickets they own or are assigned to
- `can_close_ticket()` restricts closure to admins
- `can_delete_photo()` restricts photo deletion to admins
- Comprehensive test coverage for all permission scenarios

---

### CRIT-3: Photo Upload Trusts Content-Type Header Only ✅ FIXED

**Location**: `handlers/tickets.rs:upload_photo`

**Original State**: The upload validated only the `Content-Type` from the multipart header, which is client-controlled.

**Risk**: Attacker could upload any file type (malware, scripts, HTML) by spoofing the Content-Type header.

**Resolution**: Implemented in `utils/file_validation.rs`:
- `detect_image_format()` inspects magic bytes to detect actual file type
- `validate_image_content_type()` ensures declared Content-Type matches actual format
- Supported formats with magic byte signatures:
  - JPEG: `FF D8 FF`
  - PNG: `89 50 4E 47 0D 0A 1A 0A`
  - WebP: `52 49 46 46 xx xx xx xx 57 45 42 50`
- Minimum 12 bytes required for detection (defense against truncated files)
- Rejects mismatched Content-Type (e.g., HTML claiming to be JPEG)
- Filenames generated server-side with UUID (already in place)
- Comprehensive test coverage including malicious file detection

---

### CRIT-4: Default Admin PIN Has No Expiration ✅ FIXED

**Location**: `handlers/admin.rs:admin_setup`

**Original State**: Default admin PIN was "changeme" with no forced expiration. The `setup_complete` flag prevented re-running setup, but if initial setup was skipped, the default PIN remained valid indefinitely.

**Risk**: Forgotten deployments or test instances remain accessible with default credentials.

**Resolution**: Implemented via migration `003_add_setup_deadline.sql`:
- `setup_deadline` column added to `store_settings` table
- Default deadline: 24 hours from deployment (`NOW() + INTERVAL '24 hours'`)
- Admin operations fail if setup not complete within deadline
- `min_pin_length` column added (default: 6 digits)
- Enforces minimum PIN complexity requirements

---

### CRIT-5: CORS Allows All Origins ✅ FIXED

**Location**: `cors.rs:build_cors_layer`

**Original State**: CORS configuration always allowed any origin regardless of config, with a comment "for simplicity in MVP".

**Risk**: Any website could make authenticated requests to the API if the user's browser has valid credentials/cookies.

**Resolution**: Implemented in `cors.rs`:
- Wildcard `*` in `CORS_ORIGINS` correctly allows any origin (dev only)
- Specific origins use `AllowOrigin::list()` for explicit allowlist
- Invalid origins are filtered out with warning logs
- Empty origin list blocks all cross-origin requests
- Whitespace trimming for origin strings
- Comprehensive test coverage:
  - `test_cors_allows_any_origin_when_wildcard`
  - `test_cors_allows_configured_origin`
  - `test_cors_rejects_unconfigured_origin`
  - `test_cors_allows_multiple_origins`
  - `test_cors_handles_empty_origins`
  - `test_cors_preflight_allowed_origin`
  - `test_cors_preflight_disallowed_origin`

---

## High Priority Issues

### HIGH-1: No Request Body Size Limits ✅ FIXED

**Location**: `main.rs` and `middleware/body_limit.rs`

**Original State**: No limits on request body size for JSON endpoints.

**Risk**:
- Memory exhaustion via large payloads
- DoS attacks with oversized requests
- Photo endpoint had 10MB limit, but JSON endpoints had none

**Resolution**: Implemented in `middleware/body_limit.rs`:
- `RequestBodyLimitLayer` applied to API routes
- `json_payload_error` middleware converts 413 responses to JSON format
- Consistent error response: `{ "data": null, "error": { "code": "PAYLOAD_TOO_LARGE", "message": "..." } }`
- Photo uploads retain separate 10MB limit
- Test coverage for boundary conditions

---

### HIGH-2: Employee Role Not Used for Authorization ✅ FIXED

**Location**: Database schema has `employee_role` enum (`staff`, `admin`), now enforced via RBAC.

**Original State**:
- `handlers/employees.rs`: Admin operations checked `X-Admin-PIN` header
- `handlers/tickets.rs`: No role checks, any employee could perform any operation

**Risk**: Role field provided false sense of security; staff and admin had equivalent ticket access.

**Resolution**: Implemented in `middleware/rbac.rs` and `models/employee.rs`:
- `Permission` enum defines granular permissions:
  - `CreateTicket`, `ViewTicket`, `ModifyOwnTicket`, `AddNotes`, `UploadPhotos`
  - `CloseAnyTicket`, `DeletePhotos`, `ManageEmployees`, `ManageSettings`, `ManageLocations`
- `EmployeeRole::has_permission()` maps roles to permissions
- Staff permissions: Create, View, ModifyOwn, AddNotes, UploadPhotos
- Admin permissions: All of the above plus CloseAnyTicket, DeletePhotos, ManageEmployees, ManageSettings, ManageLocations
- `require_permission()` enforces role-based access
- `require_ticket_access()` combines role check with ownership validation

---

### HIGH-3: Admin PIN in Request Headers ✅ FIXED

**Location**: All admin endpoints now use session-based authentication.

**Original State**: Admin PIN was transmitted in HTTP headers on every admin request.

**Risk**:
- Headers logged by proxies/load balancers
- Visible in browser network tools
- No session management (PIN sent repeatedly)

**Resolution**: Implemented via migration `004_add_admin_sessions.sql` and `models/admin_session.rs`:
- Session-based authentication after initial PIN verification
- `/admin/verify` returns session token on success
- `X-Admin-Session` header used for subsequent requests
- Session expiration after 30 minutes of inactivity
- `last_activity_at` timestamp for sliding expiration
- Cryptographically random 256-bit session tokens (base64url encoded)
- Sessions stored in database with proper cleanup

---

## Medium Priority Issues

### MED-1: X-Employee-ID Header Not Bound to Session ✅ FIXED

**Location**: `handlers/tickets.rs` now uses session-based employee identification.

**Original State**: Any valid employee UUID could be used in the header. There was no verification that the current request was authorized to act as that employee.

**Risk**: Employee A could impersonate Employee B by sending B's UUID in the header.

**Resolution**: Implemented via migration `005_add_employee_sessions.sql` and `models/employee_session.rs`:
- Employee ID bound to authenticated session after PIN verification
- `X-Employee-Session` header used for subsequent requests
- Employee ID derived from session, not client-provided header
- Session includes `employee_id` field that cannot be spoofed
- Same session security model as admin sessions:
  - Cryptographically random 256-bit tokens
  - Sliding expiration via `last_activity_at`
  - Database storage with proper cleanup

---

### MED-2: Cascading Deletes Lose Audit History ✅ FIXED

**Location**: Database schema now uses soft-delete for tickets.

**Original State**: `ON DELETE CASCADE` on related tables meant deleting a ticket permanently destroyed all associated audit records.

**Risk**: Deleting a ticket permanently destroyed all associated audit records.

**Resolution**: Implemented via migration `006_soft_delete_tickets.sql`:
- `deleted_at` column added to `tickets` table
- `deleted_by` column tracks who performed the deletion
- Soft-delete model: tickets are marked as deleted, not removed
- Audit history preserved: photos, notes, status history remain intact
- Hard delete still available for admin override if needed
- Queries filter out soft-deleted tickets by default

---

### MED-3: No Input Sanitization for Text Fields (Partial)

**Location**: All handlers accepting text input

**Current State**: Whitespace trimming implemented in most handlers. Full sanitization remains a future improvement.

**Fields Affected**:
- `item_description`, `condition_notes`, `requested_work`
- `customer.name`, `customer.phone`, `customer.email`
- `employee.name`
- Note `content`

**Risk**:
- Stored XSS if rendered without escaping (frontend responsibility, but defense-in-depth)
- Log injection if fields logged directly

**Status**: Acceptable for MVP
- Whitespace trimming in place
- Frontend handles output escaping (Svelte auto-escapes by default)
- Future: Consider length validation and structured logging

---

### MED-4: No Validation of Referenced Entity Existence Before Action ✅ ACCEPTABLE

**Location**: Various update handlers

**Current State**: Critical handlers check entity existence; others rely on foreign key constraints.

**Examples**:
- `update_ticket`: Checks ticket exists ✅
- `toggle_rush`: Checks ticket exists ✅
- `add_note`: Checks ticket exists ✅
- `upload_photo`: Checks ticket exists ✅
- `storage_location_id` changes: Foreign key constraint (returns clear error)

**Status**: Acceptable
- High-traffic operations validate existence
- Foreign key constraints provide reliable fallback
- Error messages are user-friendly

---

## Low Priority Issues

### LOW-1: Queue Endpoint Has No Authentication (Accepted Risk)

**Location**: `GET /api/v1/queue`

**Current State**: Returns all active tickets without authentication.

**Risk**: Information disclosure of all ticket summaries to unauthenticated users.

**Status**: Accepted for MVP
- Single-store deployment with trusted network
- Queue view is the primary workstation interface
- Adding auth would friction the core workflow
- Future: Consider authentication when multi-tenant is introduced

---

### LOW-2: Settings Endpoint Has No Read Authentication (Accepted Risk)

**Location**: `GET /api/v1/settings`

**Current State**: Returns store settings without authentication.

**Risk**: Minor information disclosure (store name, address, ticket prefix).

**Status**: Accepted for MVP
- Store info is public (name, address on receipts)
- No sensitive data exposed
- Future: Separate public vs internal settings if needed

---

## Endpoint-by-Endpoint Audit

### Tickets (`/api/v1/tickets`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/` | GET | None | Accepted (workboard view) |
| `/` | POST | X-Employee-Session | ✅ Session auth + role check |
| `/:id` | GET | None | Accepted (receipt lookup) |
| `/:id` | PUT | X-Employee-Session | ✅ Session auth + ownership check |
| `/:id/status` | POST | X-Employee-Session | ✅ Session auth + RBAC |
| `/:id/close` | POST | X-Employee-Session | ✅ Admin-only via RBAC |
| `/:id/rush` | POST | X-Employee-Session | ✅ Session auth + ownership check |
| `/:id/notes` | POST | X-Employee-Session | ✅ Session auth + role check |
| `/:id/photos` | POST | X-Employee-Session | ✅ Session auth + magic byte validation |
| `/:id/photos/:pid` | DELETE | X-Admin-Session | ✅ Admin-only via RBAC |
| `/:id/receipt.pdf` | GET | None | Accepted (customer access) |
| `/:id/label.pdf` | GET | None | Accepted (customer access) |

### Employees (`/api/v1/employees`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/` | GET | X-Admin-Session | ✅ Session auth + admin role |
| `/` | POST | X-Admin-Session | ✅ Session auth + validation |
| `/:id` | PUT | X-Admin-Session | ✅ Session auth + admin role |
| `/:id` | DELETE | X-Admin-Session | ✅ Soft delete preserves history |
| `/verify` | POST | Rate limited | ✅ Rate limit + exponential backoff |

### Admin (`/api/v1/admin`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/setup` | POST | current_pin | ✅ 24-hour deadline enforced |
| `/verify` | POST | Rate limited | ✅ Rate limit + exponential backoff |
| `/change-pin` | POST | X-Admin-Session | ✅ Session auth |

### Customers (`/api/v1/customers`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/` | GET | None | Accepted (search for repeat customers) |
| `/:id` | GET | None | Accepted (customer lookup) |

### Queue (`/api/v1/queue`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/` | GET | None | Accepted (LOW-1 - primary interface) |

### Settings (`/api/v1/settings`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/` | GET | None | Accepted (LOW-2 - public store info) |
| `/` | PUT | X-Admin-Session | ✅ Session auth + admin role |

### Locations (`/api/v1/locations`)

| Endpoint | Method | Auth | Status |
|----------|--------|------|--------|
| `/` | GET | None | Accepted (dropdown options) |
| `/` | POST | X-Admin-Session | ✅ Session auth + validation |
| `/:id` | PUT | X-Admin-Session | ✅ Session auth + admin role |

---

## Validation Checklist

### Required Validations Per Endpoint Type

**All Endpoints**:
- [x] Request body size limit enforced
- [x] Content-Type header matches expected (application/json or multipart)
- [x] UUID path parameters are valid format

**Authenticated Endpoints**:
- [x] Authentication header present and valid
- [x] Rate limiting applied
- [x] Session/token not expired
- [x] Role authorization checked

**Create Operations**:
- [x] Required fields present and non-empty
- [x] String lengths within acceptable bounds
- [x] Referenced entities exist (foreign keys)
- [x] No duplicate constraint violations (graceful error)

**Update Operations**:
- [x] Entity exists before modification
- [x] User has permission to modify (ownership or role)
- [ ] Optimistic locking if concurrent modifications possible

**File Uploads**:
- [x] File size within limits
- [x] Magic bytes match declared Content-Type
- [x] Storage key generated server-side (not client-provided)
- [x] Content-Disposition handled safely

---

## Recommendations by Priority

### Immediate (Deployment Blockers) ✅ ALL COMPLETE

1. ~~**Add rate limiting to PIN verification endpoints**~~ ✅
   - Block: CRIT-1
   - Implemented: `middleware/rate_limit.rs`

2. ~~**Fix CORS configuration to respect allowed origins**~~ ✅
   - Block: CRIT-5
   - Implemented: `cors.rs`

3. ~~**Add magic byte validation for photo uploads**~~ ✅
   - Block: CRIT-3
   - Implemented: `utils/file_validation.rs`

### Short-Term (Current Sprint) ✅ ALL COMPLETE

4. ~~**Implement row-level authorization for tickets**~~ ✅
   - Block: CRIT-2
   - Implemented: `middleware/rbac.rs`

5. ~~**Add setup deadline for default admin PIN**~~ ✅
   - Block: CRIT-4
   - Implemented: Migration `003_add_setup_deadline.sql`

6. ~~**Add request body size limits**~~ ✅
   - Block: HIGH-1
   - Implemented: `middleware/body_limit.rs`

7. ~~**Implement session-based admin authentication**~~ ✅
   - Block: HIGH-3
   - Implemented: Migration `004_add_admin_sessions.sql`, `models/admin_session.rs`

### Medium-Term (Next Quarter) ✅ MOSTLY COMPLETE

8. ~~**Enforce employee role-based authorization**~~ ✅
   - Block: HIGH-2
   - Implemented: `middleware/rbac.rs`, `models/employee.rs`

9. ~~**Bind employee ID to session instead of header**~~ ✅
   - Block: MED-1
   - Implemented: Migration `005_add_employee_sessions.sql`, `models/employee_session.rs`

10. ~~**Replace cascading deletes with soft-delete**~~ ✅
    - Block: MED-2
    - Implemented: Migration `006_soft_delete_tickets.sql`

11. **Add input sanitization and length validation** (Remaining)
    - Block: MED-3
    - Status: Partial - some handlers trim whitespace, full validation TBD

---

## Appendix: File Reference

| File | Security Relevance |
|------|-------------------|
| `main.rs` | CORS configuration, middleware stack |
| `cors.rs` | CORS origin validation with allowlist |
| `auth.rs` | PIN hashing (Argon2 - appropriate) |
| `routes/mod.rs` | Route definitions, endpoint mapping |
| `handlers/tickets.rs` | Ticket operations, photo upload |
| `handlers/employees.rs` | Employee CRUD, PIN verification |
| `handlers/admin.rs` | Admin setup, PIN change |
| `handlers/customers.rs` | Customer search (public) |
| `handlers/settings.rs` | Store settings (admin write) |
| `handlers/locations.rs` | Storage locations (admin write) |
| `middleware/rate_limit.rs` | Per-IP rate limiting with exponential backoff |
| `middleware/rbac.rs` | Role-based access control, ticket ownership |
| `middleware/body_limit.rs` | Request body size limits |
| `utils/file_validation.rs` | Magic byte validation for uploads |
| `models/admin_session.rs` | Admin session management |
| `models/employee_session.rs` | Employee session management |
| `migrations/001_initial_schema.sql` | Database schema, constraints |
| `migrations/003_add_setup_deadline.sql` | Admin PIN expiration |
| `migrations/004_add_admin_sessions.sql` | Admin session storage |
| `migrations/005_add_employee_sessions.sql` | Employee session storage |
| `migrations/006_soft_delete_tickets.sql` | Soft delete for audit preservation |
