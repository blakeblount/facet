-- Employee sessions table for secure session-based authentication
-- Similar to admin_sessions but includes employee_id to bind session to specific employee
-- This prevents employee impersonation via X-Employee-ID header spoofing

CREATE TABLE employee_sessions (
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(employee_id) ON DELETE CASCADE,
    session_token VARCHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast session token lookups
CREATE INDEX idx_employee_sessions_token ON employee_sessions (session_token);

-- Index for finding sessions by employee (for invalidation when employee is deactivated)
CREATE INDEX idx_employee_sessions_employee ON employee_sessions (employee_id);

-- Index for cleanup of expired sessions
CREATE INDEX idx_employee_sessions_expires ON employee_sessions (expires_at);

COMMENT ON TABLE employee_sessions IS 'Stores employee session tokens to prevent header-based employee impersonation';
COMMENT ON COLUMN employee_sessions.employee_id IS 'The employee this session belongs to';
COMMENT ON COLUMN employee_sessions.session_token IS '256-bit cryptographically random token, base64url encoded';
COMMENT ON COLUMN employee_sessions.expires_at IS 'Session expiration time (8 hours by default, sliding window from last activity)';
COMMENT ON COLUMN employee_sessions.last_activity_at IS 'Updated on each authenticated request for sliding expiration';
