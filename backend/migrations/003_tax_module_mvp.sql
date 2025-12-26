-- FlashBill Tax Module MVP
-- Created: 2025-12-26
-- Scope: Simple manual tax calculation, informational only

-- Tax Settings Table
-- Menyimpan tax rate manual per organization
CREATE TABLE tax_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    label VARCHAR(100) NOT NULL,           -- "Sales Tax", "VAT", "GST"
    rate DOUBLE PRECISION NOT NULL,        -- 0.0 - 1.0 (e.g. 0.07 for 7%)
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_tax_settings_org ON tax_settings(organization_id);
CREATE INDEX idx_tax_settings_default ON tax_settings(organization_id, is_default) WHERE is_default = true;

-- Add tax_label to invoices table (for snapshot)
ALTER TABLE invoices ADD COLUMN tax_label VARCHAR(100);

-- Add tax_id field (optional, for display)
ALTER TABLE invoices ADD COLUMN tax_id VARCHAR(32);

-- Update invoice_items to store tax snapshot
-- Note: tax_rate and tax_amount already exist in the table

-- Add legal disclaimer to organization settings
-- This will be stored in users.tax_settings JSONB field
-- Example: {"legal_disclaimer": "Flashbill does not calculate, verify, or file taxes..."}

-- Trigger to ensure only one default tax per organization
CREATE OR REPLACE FUNCTION enforce_single_default_tax()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_default = true THEN
        -- Unset other defaults for this organization
        UPDATE tax_settings
        SET is_default = false
        WHERE organization_id = NEW.organization_id
        AND id != NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_single_default_tax
    BEFORE INSERT OR UPDATE ON tax_settings
    FOR EACH ROW EXECUTE FUNCTION enforce_single_default_tax();

-- Trigger to update updated_at
CREATE TRIGGER update_tax_settings_updated_at
    BEFORE UPDATE ON tax_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
