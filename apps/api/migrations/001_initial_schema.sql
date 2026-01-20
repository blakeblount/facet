-- Facet Initial Database Schema
-- All MVP tables per docs/SCHEMA.md

-- ============================================================
-- ENUMS
-- ============================================================

CREATE TYPE employee_role AS ENUM ('staff', 'admin');

CREATE TYPE ticket_status AS ENUM (
    'intake',
    'in_progress',
    'waiting_on_parts',
    'ready_for_pickup',
    'closed',
    'archived'
);

-- ============================================================
-- TABLES
-- ============================================================

-- customers
-- Stores customer information. Created inline during ticket intake or when explicitly added.
CREATE TABLE customers (
    customer_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL,
    phone           VARCHAR(50),
    email           VARCHAR(255),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_customers_name ON customers (LOWER(name));
CREATE INDEX idx_customers_phone ON customers (phone);
CREATE INDEX idx_customers_email ON customers (LOWER(email));

-- employees
-- Store employees who can perform actions. PIN stored as hash.
CREATE TABLE employees (
    employee_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL,
    pin_hash        VARCHAR(255) NOT NULL,
    role            employee_role NOT NULL DEFAULT 'staff',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_employees_active ON employees (is_active) WHERE is_active = TRUE;

-- storage_locations
-- Managed list of physical storage locations.
CREATE TABLE storage_locations (
    location_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL UNIQUE,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_storage_locations_active ON storage_locations (is_active) WHERE is_active = TRUE;

-- tickets
-- Core entity for repair jobs.
CREATE TABLE tickets (
    ticket_id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    friendly_code           VARCHAR(20) NOT NULL UNIQUE,

    -- Customer reference
    customer_id             UUID NOT NULL REFERENCES customers(customer_id),

    -- Item details
    item_type               VARCHAR(100),
    item_description        TEXT NOT NULL,
    condition_notes         TEXT NOT NULL,
    requested_work          TEXT NOT NULL,

    -- Operational
    status                  ticket_status NOT NULL DEFAULT 'intake',
    is_rush                 BOOLEAN NOT NULL DEFAULT FALSE,
    promise_date            DATE,
    storage_location_id     UUID NOT NULL REFERENCES storage_locations(location_id),

    -- Pricing
    quote_amount            DECIMAL(10,2),
    actual_amount           DECIMAL(10,2),

    -- Employee attribution
    taken_in_by             UUID NOT NULL REFERENCES employees(employee_id),
    worked_by               UUID REFERENCES employees(employee_id),
    closed_by               UUID REFERENCES employees(employee_id),
    last_modified_by        UUID REFERENCES employees(employee_id),

    -- Timestamps
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at               TIMESTAMPTZ,

    -- Queue ordering (for future drag-and-drop)
    queue_position          INTEGER
);

-- Primary query: workboard lanes
CREATE INDEX idx_tickets_status ON tickets (status);
CREATE INDEX idx_tickets_status_rush_created ON tickets (status, is_rush DESC, created_at ASC);

-- Search
CREATE INDEX idx_tickets_friendly_code ON tickets (friendly_code);
CREATE INDEX idx_tickets_customer ON tickets (customer_id);
CREATE INDEX idx_tickets_created ON tickets (created_at);

-- Overdue query
CREATE INDEX idx_tickets_promise_date ON tickets (promise_date) WHERE promise_date IS NOT NULL;

-- ticket_photos
-- Photos attached to tickets. Stored in S3, referenced here.
CREATE TABLE ticket_photos (
    photo_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id           UUID NOT NULL REFERENCES tickets(ticket_id) ON DELETE CASCADE,
    storage_key         VARCHAR(500) NOT NULL,
    content_type        VARCHAR(100) NOT NULL,
    size_bytes          INTEGER NOT NULL,
    uploaded_by         UUID NOT NULL REFERENCES employees(employee_id),
    uploaded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ticket_photos_ticket ON ticket_photos (ticket_id);

-- ticket_notes
-- Internal notes on tickets. Append-only.
CREATE TABLE ticket_notes (
    note_id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id           UUID NOT NULL REFERENCES tickets(ticket_id) ON DELETE CASCADE,
    content             TEXT NOT NULL,
    created_by          UUID NOT NULL REFERENCES employees(employee_id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ticket_notes_ticket ON ticket_notes (ticket_id);
CREATE INDEX idx_ticket_notes_created ON ticket_notes (ticket_id, created_at DESC);

-- ticket_status_history
-- Audit trail for status changes.
CREATE TABLE ticket_status_history (
    history_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id           UUID NOT NULL REFERENCES tickets(ticket_id) ON DELETE CASCADE,
    from_status         ticket_status,
    to_status           ticket_status NOT NULL,
    changed_by          UUID NOT NULL REFERENCES employees(employee_id),
    changed_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ticket_status_history_ticket ON ticket_status_history (ticket_id);
CREATE INDEX idx_ticket_status_history_changed ON ticket_status_history (ticket_id, changed_at DESC);

-- ticket_field_history
-- Audit trail for field changes (pricing, condition, etc.).
CREATE TABLE ticket_field_history (
    history_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id           UUID NOT NULL REFERENCES tickets(ticket_id) ON DELETE CASCADE,
    field_name          VARCHAR(100) NOT NULL,
    old_value           TEXT,
    new_value           TEXT,
    changed_by          UUID NOT NULL REFERENCES employees(employee_id),
    changed_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ticket_field_history_ticket ON ticket_field_history (ticket_id);

-- store_settings
-- Key-value store for configuration. Single-row for MVP (multi-tenant later).
CREATE TABLE store_settings (
    setting_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_name          VARCHAR(255) NOT NULL DEFAULT 'Jewelry Store',
    store_phone         VARCHAR(50),
    store_address       TEXT,
    ticket_prefix       VARCHAR(10) NOT NULL DEFAULT 'JR',
    next_ticket_number  INTEGER NOT NULL DEFAULT 1,
    currency            VARCHAR(3) NOT NULL DEFAULT 'USD',
    max_photos_per_ticket INTEGER NOT NULL DEFAULT 10,
    admin_pin_hash      VARCHAR(255) NOT NULL,
    setup_complete      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure single row for MVP
CREATE UNIQUE INDEX idx_store_settings_singleton ON store_settings ((TRUE));

-- ============================================================
-- FUNCTIONS
-- ============================================================

-- Atomic friendly_code generation
-- Called during ticket creation
CREATE OR REPLACE FUNCTION generate_friendly_code()
RETURNS VARCHAR(20) AS $$
DECLARE
    prefix VARCHAR(10);
    next_num INTEGER;
    code VARCHAR(20);
BEGIN
    UPDATE store_settings
    SET next_ticket_number = next_ticket_number + 1
    RETURNING ticket_prefix, next_ticket_number - 1
    INTO prefix, next_num;

    code := prefix || '-' || LPAD(next_num::TEXT, 4, '0');
    RETURN code;
END;
$$ LANGUAGE plpgsql;
