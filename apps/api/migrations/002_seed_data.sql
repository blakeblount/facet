-- Seed data for Facet jewelry repair system
-- Creates default admin, storage locations, and store settings

-- ============================================================
-- DEFAULT ADMIN EMPLOYEE
-- ============================================================
-- Default admin with PIN "changeme" (argon2 hashed)
-- IMPORTANT: Change this PIN immediately after initial setup!
INSERT INTO employees (name, pin_hash, role, is_active)
VALUES (
    'Admin',
    '$argon2id$v=19$m=19456,t=2,p=1$D/DKaAnOyTZlFVp73kqgzg$2chDrK0UX8r3hPNbl42ZgFjJVlTvaqIhzFj5/FomjMY',
    'admin',
    TRUE
);

-- ============================================================
-- DEFAULT STORAGE LOCATIONS
-- ============================================================
INSERT INTO storage_locations (name, is_active) VALUES
    ('Safe Drawer 1', TRUE),
    ('Safe Drawer 2', TRUE),
    ('Workbench A', TRUE),
    ('Workbench B', TRUE),
    ('Display Case', TRUE);

-- ============================================================
-- INITIAL STORE SETTINGS
-- ============================================================
-- Uses same admin PIN hash for store admin_pin_hash
-- setup_complete = FALSE indicates first-time setup needed
INSERT INTO store_settings (
    store_name,
    ticket_prefix,
    next_ticket_number,
    currency,
    max_photos_per_ticket,
    admin_pin_hash,
    setup_complete
)
VALUES (
    'Jewelry Store',
    'JR',
    1,
    'USD',
    10,
    '$argon2id$v=19$m=19456,t=2,p=1$D/DKaAnOyTZlFVp73kqgzg$2chDrK0UX8r3hPNbl42ZgFjJVlTvaqIhzFj5/FomjMY',
    FALSE
);
