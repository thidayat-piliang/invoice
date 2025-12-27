-- Add tracking fields for invoice notifications and read receipts
-- Migration for guest checkout and notification features

-- Add viewed_at timestamp for read receipt tracking
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS viewed_at TIMESTAMP WITH TIME ZONE;

-- Add notification_sent_at for email notification tracking
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS notification_sent_at TIMESTAMP WITH TIME ZONE;

-- Add whatsapp_sent_at for WhatsApp notification tracking
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS whatsapp_sent_at TIMESTAMP WITH TIME ZONE;

-- Add guest payment token for secure guest checkout access
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS guest_payment_token VARCHAR(255);

-- Add index for guest token lookup
CREATE INDEX IF NOT EXISTS idx_invoices_guest_token ON invoices(guest_payment_token);

-- Add index for viewed status queries
CREATE INDEX IF NOT EXISTS idx_invoices_viewed ON invoices(viewed_at);

-- Add index for notification tracking queries
CREATE INDEX IF NOT EXISTS idx_invoices_notifications ON invoices(notification_sent_at, whatsapp_sent_at);

-- Update existing invoices to have default values
UPDATE invoices
SET viewed_at = NULL,
    notification_sent_at = NULL,
    whatsapp_sent_at = NULL,
    guest_payment_token = NULL
WHERE viewed_at IS NULL
  AND notification_sent_at IS NULL
  AND whatsapp_sent_at IS NULL;

-- Create a function to generate guest payment token
CREATE OR REPLACE FUNCTION generate_guest_token(invoice_id UUID)
RETURNS VARCHAR AS $$
DECLARE
    token VARCHAR;
BEGIN
    token := 'guest_' || invoice_id::text || '_' || substr(md5(random()::text), 1, 16);
    RETURN token;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to auto-generate guest token on invoice creation
CREATE OR REPLACE FUNCTION auto_generate_guest_token()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.guest_payment_token IS NULL THEN
        NEW.guest_payment_token := 'guest_' || NEW.id::text || '_' || substr(md5(random()::text), 1, 16);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_generate_guest_token
    BEFORE INSERT ON invoices
    FOR EACH ROW
    EXECUTE FUNCTION auto_generate_guest_token();

-- Add view to get invoices with read status
CREATE OR REPLACE VIEW invoice_read_status AS
SELECT
    i.id,
    i.invoice_number,
    i.status,
    i.user_id,
    i.client_id,
    i.sent_at,
    i.viewed_at,
    i.notification_sent_at,
    i.whatsapp_sent_at,
    i.guest_payment_token,
    CASE
        WHEN i.viewed_at IS NOT NULL THEN 'read'
        WHEN i.sent_at IS NOT NULL THEN 'sent'
        ELSE 'draft'
    END as read_status,
    CASE
        WHEN i.viewed_at IS NOT NULL THEN
            EXTRACT(DAY FROM (i.viewed_at - i.sent_at))
        ELSE NULL
    END as days_to_read,
    c.name as client_name,
    c.email as client_email,
    c.phone as client_phone
FROM invoices i
JOIN clients c ON i.client_id = c.id;

-- Add function to mark invoice as viewed
CREATE OR REPLACE FUNCTION mark_invoice_viewed(p_invoice_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE invoices
    SET viewed_at = NOW(),
        status = CASE
            WHEN status = 'sent' THEN 'viewed'
            ELSE status
        END,
        updated_at = NOW()
    WHERE id = p_invoice_id;
END;
$$ LANGUAGE plpgsql;

-- Add function to update notification sent timestamp
CREATE OR REPLACE FUNCTION update_notification_sent(p_invoice_id UUID, p_is_whatsapp BOOLEAN)
RETURNS VOID AS $$
BEGIN
    IF p_is_whatsapp THEN
        UPDATE invoices
        SET whatsapp_sent_at = NOW(),
            updated_at = NOW()
        WHERE id = p_invoice_id;
    ELSE
        UPDATE invoices
        SET notification_sent_at = NOW(),
            updated_at = NOW()
        WHERE id = p_invoice_id;
    END IF;
END;
$$ LANGUAGE plpgsql;
