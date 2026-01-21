-- Add soft-delete support for tickets
-- This preserves audit history (status_history, field_history) when tickets are deleted

-- Add soft-delete columns to tickets
ALTER TABLE tickets ADD COLUMN deleted_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN deleted_by UUID REFERENCES employees(employee_id);

-- Create index for efficiently filtering out deleted tickets
CREATE INDEX idx_tickets_deleted ON tickets (deleted_at) WHERE deleted_at IS NOT NULL;

-- Create index for efficiently finding non-deleted tickets (most common query)
CREATE INDEX idx_tickets_not_deleted ON tickets (deleted_at) WHERE deleted_at IS NULL;

-- Change ON DELETE CASCADE to ON DELETE RESTRICT for audit tables
-- This prevents accidental deletion of audit history
-- Note: We keep CASCADE on photos and notes since they are attachments, not audit records

-- ticket_status_history: RESTRICT to preserve audit trail
ALTER TABLE ticket_status_history
    DROP CONSTRAINT ticket_status_history_ticket_id_fkey,
    ADD CONSTRAINT ticket_status_history_ticket_id_fkey
        FOREIGN KEY (ticket_id) REFERENCES tickets(ticket_id) ON DELETE RESTRICT;

-- ticket_field_history: RESTRICT to preserve audit trail
ALTER TABLE ticket_field_history
    DROP CONSTRAINT ticket_field_history_ticket_id_fkey,
    ADD CONSTRAINT ticket_field_history_ticket_id_fkey
        FOREIGN KEY (ticket_id) REFERENCES tickets(ticket_id) ON DELETE RESTRICT;
