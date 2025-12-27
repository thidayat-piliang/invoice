-- Add partial payment settings to invoices
-- Up migration

ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS allow_partial_payment BOOLEAN DEFAULT true;

ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS min_payment_amount DECIMAL(10,2);

-- Update existing invoices to allow partial (backward compatible)
UPDATE invoices
SET allow_partial_payment = true
WHERE allow_partial_payment IS NULL;

-- Create payment history view
CREATE OR REPLACE VIEW payment_history AS
SELECT
    p.id,
    p.invoice_id,
    p.amount,
    p.payment_method,
    p.notes,
    p.status,
    p.created_at,
    i.invoice_number,
    i.client_id,
    i.total_amount
FROM payments p
JOIN invoices i ON p.invoice_id = i.id
ORDER BY p.created_at DESC;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_invoices_partial_status
ON invoices(status, allow_partial_payment)
WHERE status IN ('partial', 'overdue');

CREATE INDEX IF NOT EXISTS idx_payments_invoice_id
ON payments(invoice_id);

-- Add status tracking for partial payments
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS partial_payment_count INTEGER DEFAULT 0;

-- Update trigger to auto-increment partial count
CREATE OR REPLACE FUNCTION update_partial_count()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'partial' THEN
        NEW.partial_payment_count := COALESCE(OLD.partial_payment_count, 0) + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_partial_count
    BEFORE UPDATE ON invoices
    FOR EACH ROW
    WHEN (NEW.status = 'partial' AND OLD.status != 'partial')
    EXECUTE FUNCTION update_partial_count();
