# Security Review: Facet API

> **Review Date**: January 2026
> **Scope**: Backend API security audit
> **Principle**: Treat the client as hostile; validate everything server-side

---

## Executive Summary

This security review evaluates the Facet API with the core assumption that **all client input is untrusted**. The API serves a jewelry repair shop management system with employee PIN authentication, ticket management, and photo uploads.

### Risk Summary

| Severity | Count | Description |
|----------|-------|-------------|
| CRITICAL | 5 | Immediate deployment blockers |
| HIGH | 3 | Address within current sprint |
| MEDIUM | 4 | Address within next quarter |
| LOW | 2 | Long-term improvements |

### Key Findings

1. **No rate limiting** on authentication endpoints enables brute-force attacks
2. **Missing row-level authorization** allows any employee to modify any ticket
3. **CORS allows all origins** regardless of configuration
4. **Photo uploads lack magic byte validation** (Content-Type header only)
5. **No request body size limits** on API endpoints

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
| Header manipulation | HIGH | Minimal - UUIDs parsed but not validated against session |
| Body tampering | HIGH | Partial - serde validates types but not business logic |
| File upload exploits | CRITICAL | Weak - Content-Type header trusted |
| Brute force auth | CRITICAL | None - No rate limiting |
| SQL injection | LOW | Protected - parameterized queries via sqlx |

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

### CRIT-1: No Rate Limiting on PIN Verification

**Location**: `POST /api/v1/employees/verify`, `POST /api/v1/admin/verify`

**Current State**: Endpoints accept unlimited requests with no throttling.

**Risk**: 4-digit PINs can be enumerated in ~10,000 attempts. At 100 req/sec, complete enumeration takes under 2 minutes.

**Recommendation**:
- Add rate limiting middleware (e.g., `tower::limit::RateLimitLayer`)
- Implement per-IP throttling: 5 attempts per minute
- Add exponential backoff after failures
- Consider account lockout after 10 failed attempts

---

### CRIT-2: Missing Row-Level Authorization

**Location**: All ticket handlers (`handlers/tickets.rs`)

**Current State**: `X-Employee-ID` header is validated to ensure the employee exists and is active, but there is no check that this employee has permission to modify the specific ticket.

**Affected Endpoints**:
- `PUT /api/v1/tickets/:ticket_id`
- `POST /api/v1/tickets/:ticket_id/status`
- `POST /api/v1/tickets/:ticket_id/close`
- `POST /api/v1/tickets/:ticket_id/rush`
- `POST /api/v1/tickets/:ticket_id/notes`
- `POST /api/v1/tickets/:ticket_id/photos`

**Risk**: Any authenticated employee can modify any ticket in the system by providing a valid employee ID and guessing/enumerating ticket UUIDs.

**Recommendation**:
- Implement role-based access control (RBAC) using the existing `employee_role` enum
- Define ownership model (e.g., `taken_in_by` or `worked_by` owns the ticket)
- Admin role should bypass ownership checks
- Staff role should only modify tickets they own or are assigned to

---

### CRIT-3: Photo Upload Trusts Content-Type Header Only

**Location**: `handlers/tickets.rs:upload_photo` (lines 1486-1497)

**Current State**:
```
let content_type = field.content_type()...
if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
    return Err(...)
}
```

The upload validates the `Content-Type` from the multipart header, which is client-controlled.

**Risk**: Attacker can upload any file type (malware, scripts, HTML) by spoofing the Content-Type header.

**Recommendation**:
- Validate file magic bytes against expected signatures
- JPEG: `FF D8 FF`
- PNG: `89 50 4E 47 0D 0A 1A 0A`
- WebP: `52 49 46 46 xx xx xx xx 57 45 42 50`
- Consider re-encoding images server-side to strip metadata/exploits
- Generate random filenames (already done with UUID)

---

### CRIT-4: Default Admin PIN Has No Expiration

**Location**: `handlers/admin.rs:admin_setup`

**Current State**: Default admin PIN is "changeme" with no forced expiration. The `setup_complete` flag prevents re-running setup, but if initial setup is skipped, the default PIN remains valid indefinitely.

**Risk**: Forgotten deployments or test instances remain accessible with default credentials.

**Recommendation**:
- Add `setup_deadline` timestamp column
- Fail all admin operations if setup not complete within 24 hours
- Require minimum PIN complexity (length, not common patterns)
- Log/alert on default PIN usage

---

### CRIT-5: CORS Allows All Origins

**Location**: `main.rs:build_cors_layer` (lines 63-77)

**Current State**:
```rust
if config.cors_origins.len() == 1 && config.cors_origins[0] == "*" {
    cors.allow_origin(Any)
} else {
    // For specific origins, we still use Any for simplicity in MVP
    cors.allow_origin(Any)  // <-- Always allows any origin
}
```

**Risk**: Any website can make authenticated requests to the API if the user's browser has valid credentials/cookies.

**Recommendation**:
- Parse and validate specific origins from configuration
- Use `tower_http::cors::AllowOrigin::list()` for explicit origins
- Remove the "simplicity" fallback that ignores configuration

---

## High Priority Issues

### HIGH-1: No Request Body Size Limits

**Location**: `main.rs` - missing `RequestBodyLimit` middleware

**Current State**: No limits on request body size for JSON endpoints.

**Risk**:
- Memory exhaustion via large payloads
- DoS attacks with oversized requests
- Photo endpoint has 10MB limit, but JSON endpoints have none

**Recommendation**:
- Add `tower_http::limit::RequestBodyLimitLayer`
- Reasonable default: 1MB for JSON endpoints
- Keep existing 10MB limit for photo uploads

---

### HIGH-2: Employee Role Not Used for Authorization

**Location**: Database schema has `employee_role` enum (`staff`, `admin`), but role checks are inconsistent.

**Current State**:
- `handlers/employees.rs`: Admin operations check `X-Admin-PIN` header
- `handlers/tickets.rs`: No role checks, any employee can perform any operation

**Risk**: Role field provides false sense of security; staff and admin have equivalent ticket access.

**Recommendation**:
- Define clear permission matrix per role
- Staff: Create tickets, modify own tickets, add notes/photos
- Admin: All ticket operations, employee management, settings
- Enforce role checks in middleware or handler layer

---

### HIGH-3: Admin PIN in Request Headers

**Location**: All admin endpoints using `X-Admin-PIN` header

**Current State**: Admin PIN is transmitted in HTTP headers on every admin request.

**Risk**:
- Headers logged by proxies/load balancers
- Visible in browser network tools
- No session management (PIN sent repeatedly)

**Recommendation**:
- Implement session-based authentication after initial PIN verification
- Return session token on successful `/admin/verify`
- Use `X-Admin-Session` header for subsequent requests
- Session expiration after 30 minutes of inactivity

---

## Medium Priority Issues

### MED-1: X-Employee-ID Header Not Bound to Session

**Location**: `handlers/tickets.rs:extract_employee_id`

**Current State**: Any valid employee UUID can be used in the header. There's no verification that the current request is authorized to act as that employee.

**Risk**: Employee A can impersonate Employee B by sending B's UUID in the header.

**Recommendation**:
- Bind employee ID to authenticated session
- After PIN verification, store employee_id in session
- Derive employee_id from session, not client-provided header

---

### MED-2: Cascading Deletes Lose Audit History

**Location**: Database schema (`ON DELETE CASCADE`)

**Current State**:
```sql
ticket_photos ... ON DELETE CASCADE
ticket_notes ... ON DELETE CASCADE
ticket_status_history ... ON DELETE CASCADE
ticket_field_history ... ON DELETE CASCADE
```

**Risk**: Deleting a ticket permanently destroys all associated audit records.

**Recommendation**:
- Remove `ON DELETE CASCADE` from history tables
- Implement soft-delete for tickets (add `deleted_at` column)
- Archive instead of delete for compliance requirements

---

### MED-3: No Input Sanitization for Text Fields

**Location**: All handlers accepting text input

**Current State**: Text fields are stored as-is without sanitization.

**Fields Affected**:
- `item_description`, `condition_notes`, `requested_work`
- `customer.name`, `customer.phone`, `customer.email`
- `employee.name`
- Note `content`

**Risk**:
- Stored XSS if rendered without escaping (frontend responsibility, but defense-in-depth)
- Log injection if fields logged directly

**Recommendation**:
- Trim whitespace (partially done in some handlers)
- Validate maximum lengths
- Consider HTML entity encoding for display-bound fields

---

### MED-4: No Validation of Referenced Entity Existence Before Action

**Location**: Various update handlers

**Current State**: Some handlers check entity existence, others rely on foreign key constraints.

**Examples**:
- `update_ticket`: Checks ticket exists
- `toggle_rush`: Checks ticket exists
- `add_note`: Checks ticket exists
- `upload_photo`: Checks ticket exists
- `storage_location_id` changes: Foreign key constraint only

**Recommendation**:
- Standardize existence checks before operations
- Return clear 404 errors vs database constraint errors

---

## Low Priority Issues

### LOW-1: Queue Endpoint Has No Authentication

**Location**: `GET /api/v1/queue`

**Current State**: Returns all active tickets without authentication.

**Risk**: Information disclosure of all ticket summaries to unauthenticated users.

**Recommendation**:
- Require X-Employee-ID header for queue access
- Consider if public queue view is a valid use case

---

### LOW-2: Settings Endpoint Has No Read Authentication

**Location**: `GET /api/v1/settings`

**Current State**: Returns store settings without authentication.

**Risk**: Minor information disclosure (store name, address, ticket prefix).

**Recommendation**:
- Acceptable for MVP if store info is public
- Consider separating public vs internal settings

---

## Endpoint-by-Endpoint Audit

### Tickets (`/api/v1/tickets`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/` | GET | None | No auth for listing tickets |
| `/` | POST | X-Employee-ID | No role check, validates required fields |
| `/:id` | GET | None | No auth for ticket detail |
| `/:id` | PUT | X-Employee-ID | No ownership check (CRIT-2) |
| `/:id/status` | POST | X-Employee-ID | Validates status transitions, no ownership |
| `/:id/close` | POST | X-Employee-ID | Requires ReadyForPickup status |
| `/:id/rush` | POST | X-Employee-ID | Checks ticket is open |
| `/:id/notes` | POST | X-Employee-ID | Validates content non-empty |
| `/:id/photos` | POST | X-Employee-ID | Content-Type only (CRIT-3), 10MB limit |
| `/:id/photos/:pid` | DELETE | X-Admin-PIN | Admin only, proper auth |
| `/:id/receipt.pdf` | GET | None | No auth |
| `/:id/label.pdf` | GET | None | No auth |

### Employees (`/api/v1/employees`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/` | GET | X-Admin-PIN | Proper admin auth |
| `/` | POST | X-Admin-PIN | Validates name/PIN required |
| `/:id` | PUT | X-Admin-PIN | Proper admin auth |
| `/:id` | DELETE | X-Admin-PIN | Hard delete, warns about history |
| `/verify` | POST | None | No rate limiting (CRIT-1) |

### Admin (`/api/v1/admin`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/setup` | POST | current_pin | One-time use, no expiration (CRIT-4) |
| `/verify` | POST | None | No rate limiting (CRIT-1) |
| `/change-pin` | POST | X-Admin-PIN | Proper auth |

### Customers (`/api/v1/customers`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/` | GET | None | Public search, no auth |
| `/:id` | GET | None | No auth for customer detail |

### Queue (`/api/v1/queue`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/` | GET | None | No auth (LOW-1) |

### Settings (`/api/v1/settings`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/` | GET | None | No auth (LOW-2) |
| `/` | PUT | X-Admin-PIN | Proper admin auth |

### Locations (`/api/v1/locations`)

| Endpoint | Method | Auth | Issues |
|----------|--------|------|--------|
| `/` | GET | None | Public list, acceptable |
| `/` | POST | X-Admin-PIN | Validates name uniqueness |
| `/:id` | PUT | X-Admin-PIN | Proper admin auth |

---

## Validation Checklist

### Required Validations Per Endpoint Type

**All Endpoints**:
- [ ] Request body size limit enforced
- [ ] Content-Type header matches expected (application/json or multipart)
- [ ] UUID path parameters are valid format

**Authenticated Endpoints**:
- [ ] Authentication header present and valid
- [ ] Rate limiting applied
- [ ] Session/token not expired
- [ ] Role authorization checked

**Create Operations**:
- [ ] Required fields present and non-empty
- [ ] String lengths within acceptable bounds
- [ ] Referenced entities exist (foreign keys)
- [ ] No duplicate constraint violations (graceful error)

**Update Operations**:
- [ ] Entity exists before modification
- [ ] User has permission to modify (ownership or role)
- [ ] Optimistic locking if concurrent modifications possible

**File Uploads**:
- [ ] File size within limits
- [ ] Magic bytes match declared Content-Type
- [ ] Storage key generated server-side (not client-provided)
- [ ] Content-Disposition handled safely

---

## Recommendations by Priority

### Immediate (Deployment Blockers)

1. **Add rate limiting to PIN verification endpoints**
   - Block: CRIT-1
   - Effort: Low (add middleware)

2. **Fix CORS configuration to respect allowed origins**
   - Block: CRIT-5
   - Effort: Low (fix existing code)

3. **Add magic byte validation for photo uploads**
   - Block: CRIT-3
   - Effort: Medium (add validation logic)

### Short-Term (Current Sprint)

4. **Implement row-level authorization for tickets**
   - Block: CRIT-2
   - Effort: Medium (define ownership model)

5. **Add setup deadline for default admin PIN**
   - Block: CRIT-4
   - Effort: Low (add timestamp check)

6. **Add request body size limits**
   - Block: HIGH-1
   - Effort: Low (add middleware)

7. **Implement session-based admin authentication**
   - Block: HIGH-3
   - Effort: Medium (add session management)

### Medium-Term (Next Quarter)

8. **Enforce employee role-based authorization**
   - Block: HIGH-2
   - Effort: Medium (define permission matrix)

9. **Bind employee ID to session instead of header**
   - Block: MED-1
   - Effort: Medium (refactor auth flow)

10. **Replace cascading deletes with soft-delete**
    - Block: MED-2
    - Effort: Medium (schema migration)

11. **Add input sanitization and length validation**
    - Block: MED-3
    - Effort: Low (add validators)

---

## Appendix: File Reference

| File | Security Relevance |
|------|-------------------|
| `main.rs` | CORS configuration, middleware stack |
| `auth.rs` | PIN hashing (Argon2 - appropriate) |
| `routes/mod.rs` | Route definitions, endpoint mapping |
| `handlers/tickets.rs` | Ticket operations, photo upload |
| `handlers/employees.rs` | Employee CRUD, PIN verification |
| `handlers/admin.rs` | Admin setup, PIN change |
| `handlers/customers.rs` | Customer search (public) |
| `handlers/settings.rs` | Store settings (admin write) |
| `handlers/locations.rs` | Storage locations (admin write) |
| `migrations/001_initial_schema.sql` | Database schema, constraints |
