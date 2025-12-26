-- Fix tax_id column length to accommodate UUID (36 chars) + buffer
ALTER TABLE invoices ALTER COLUMN tax_id TYPE VARCHAR(50);
