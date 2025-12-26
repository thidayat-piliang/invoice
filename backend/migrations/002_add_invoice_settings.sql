-- Add invoice settings to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS invoice_settings JSONB DEFAULT '{"template": "default", "logo_url": null, "terms": "Payment due within 30 days", "notes": "Thank you for your business!"}';
