-- Admin sessions table for secure session-based authentication
-- Replaces sending PIN on every request with session tokens

CREATE TABLE admin_sessions (
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_token VARCHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast session token lookups
CREATE INDEX idx_admin_sessions_token ON admin_sessions (session_token);

-- Index for cleanup of expired sessions
CREATE INDEX idx_admin_sessions_expires ON admin_sessions (expires_at);

COMMENT ON TABLE admin_sessions IS 'Stores admin session tokens to avoid sending PIN on every request';
COMMENT ON COLUMN admin_sessions.session_token IS '256-bit cryptographically random token, base64url encoded';
COMMENT ON COLUMN admin_sessions.expires_at IS 'Session expiration time (30 min sliding window from last activity)';
COMMENT ON COLUMN admin_sessions.last_activity_at IS 'Updated on each authenticated request for sliding expiration';
