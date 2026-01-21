-- Add setup deadline to enforce PIN change within 24 hours of deployment
-- This prevents forgotten deployments from remaining accessible with default credentials

-- Add setup_deadline column
ALTER TABLE store_settings
ADD COLUMN setup_deadline TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours');

-- Add minimum PIN length requirement tracking
ALTER TABLE store_settings
ADD COLUMN min_pin_length INTEGER NOT NULL DEFAULT 6;
