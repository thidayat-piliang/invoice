-- FlashBill Database Schema v1.0
-- Created: 2025-12-25

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users Table (Core)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(20),
    company_name VARCHAR(255),
    business_type VARCHAR(50),

    password_hash VARCHAR(255),
    email_verified BOOLEAN DEFAULT FALSE,
    verification_token VARCHAR(100),
    reset_token VARCHAR(100),
    reset_token_expires TIMESTAMP,

    subscription_tier VARCHAR(20) DEFAULT 'free',
    subscription_status VARCHAR(20) DEFAULT 'active',
    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    current_period_end TIMESTAMP,

    business_address JSONB,
    tax_settings JSONB,
    currency VARCHAR(3) DEFAULT 'USD',

    notification_settings JSONB DEFAULT '{"email_payment_received": true, "email_invoice_paid": true, "email_payment_reminder": true, "push_payment_received": true, "push_overdue": true}',

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_users_subscription ON users(subscription_tier, subscription_status);
CREATE INDEX idx_users_created ON users(created_at);

-- Clients Table
CREATE TABLE clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    company_name VARCHAR(255),

    billing_address JSONB,

    payment_terms INTEGER DEFAULT 30,
    tax_exempt BOOLEAN DEFAULT FALSE,
    tax_exempt_certificate VARCHAR(255),
    notes TEXT,

    total_invoiced DECIMAL(15,2) DEFAULT 0.00,
    total_paid DECIMAL(15,2) DEFAULT 0.00,
    average_payment_days INTEGER,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_clients_user ON clients(user_id);
CREATE INDEX idx_clients_name ON clients(name);
CREATE INDEX idx_clients_created ON clients(created_at);

-- Invoices Table
CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,

    invoice_number VARCHAR(50) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'draft',

    issue_date DATE NOT NULL,
    due_date DATE NOT NULL,
    sent_at TIMESTAMP WITH TIME ZONE,
    paid_at TIMESTAMP WITH TIME ZONE,

    subtotal DECIMAL(15,2) NOT NULL,
    tax_amount DECIMAL(15,2) DEFAULT 0.00,
    discount_amount DECIMAL(15,2) DEFAULT 0.00,
    total_amount DECIMAL(15,2) NOT NULL,
    amount_paid DECIMAL(15,2) DEFAULT 0.00,

    tax_calculation JSONB,
    tax_included BOOLEAN DEFAULT FALSE,

    payment_method VARCHAR(50),
    payment_reference VARCHAR(255),

    items JSONB NOT NULL,
    notes TEXT,
    terms TEXT,

    receipt_image_url VARCHAR(500),
    pdf_url VARCHAR(500),

    reminder_sent_count INTEGER DEFAULT 0,
    last_reminder_sent TIMESTAMP WITH TIME ZONE,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_invoices_user ON invoices(user_id);
CREATE INDEX idx_invoices_client ON invoices(client_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_date ON invoices(due_date);
CREATE INDEX idx_invoices_number ON invoices(invoice_number);
CREATE INDEX idx_invoices_created ON invoices(created_at);

-- Invoice Items Sub-table
CREATE TABLE invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,

    description TEXT NOT NULL,
    quantity DECIMAL(10,2) NOT NULL,
    unit_price DECIMAL(15,2) NOT NULL,
    tax_rate DECIMAL(5,2) DEFAULT 0.00,
    tax_amount DECIMAL(15,2) DEFAULT 0.00,
    total DECIMAL(15,2) GENERATED ALWAYS AS (quantity * unit_price + tax_amount) STORED,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_invoice_items_invoice ON invoice_items(invoice_id);

-- Payments Table
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    payment_method VARCHAR(50) NOT NULL,

    gateway VARCHAR(50),
    gateway_payment_id VARCHAR(255),
    gateway_fee DECIMAL(15,2) DEFAULT 0.00,

    status VARCHAR(20) DEFAULT 'pending',
    failure_reason TEXT,

    paid_by VARCHAR(255),
    notes TEXT,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_payments_invoice ON payments(invoice_id);
CREATE INDEX idx_payments_user ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);

-- Expenses Table
CREATE TABLE expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    category VARCHAR(50) NOT NULL,

    vendor VARCHAR(255),
    description TEXT,
    receipt_image_url VARCHAR(500),

    date_incurred DATE NOT NULL,
    tax_deductible BOOLEAN DEFAULT TRUE,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_expenses_user ON expenses(user_id);
CREATE INDEX idx_expenses_category ON expenses(category);
CREATE INDEX idx_expenses_date ON expenses(date_incurred);

-- Audit Log
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID,
    changes JSONB,
    ip_address INET,
    user_agent TEXT,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_logs(user_id);
CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);

-- Session Tokens Table
CREATE TABLE session_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    token_type VARCHAR(20) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_session_user ON session_tokens(user_id);
CREATE INDEX idx_session_token ON session_tokens(token_hash);
CREATE INDEX idx_session_expires ON session_tokens(expires_at);

-- Views
CREATE VIEW monthly_metrics AS
SELECT
    user_id,
    DATE_TRUNC('month', issue_date) as month,
    COUNT(*) as invoice_count,
    SUM(total_amount) as total_invoiced,
    SUM(amount_paid) as total_paid,
    AVG(
        CASE WHEN paid_at IS NOT NULL
        THEN EXTRACT(DAY FROM paid_at - issue_date)
        END
    ) as avg_payment_days
FROM invoices
GROUP BY user_id, DATE_TRUNC('month', issue_date);

CREATE VIEW overdue_invoices AS
SELECT
    i.*,
    c.name as client_name,
    c.email as client_email,
    EXTRACT(DAY FROM (NOW() - i.due_date)) as days_overdue,
    (i.total_amount - i.amount_paid) as balance_due
FROM invoices i
JOIN clients c ON i.client_id = c.id
WHERE i.status NOT IN ('paid', 'cancelled')
  AND i.due_date < CURRENT_DATE;

CREATE VIEW revenue_metrics AS
SELECT
    user_id,
    DATE_TRUNC('month', paid_at) as month,
    COUNT(*) as paid_invoice_count,
    SUM(total_amount) as total_revenue,
    SUM(amount_paid) as total_collected,
    AVG(total_amount) as avg_invoice_value
FROM invoices
WHERE paid_at IS NOT NULL
GROUP BY user_id, DATE_TRUNC('month', paid_at);

-- Triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_clients_updated_at BEFORE UPDATE ON clients
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_invoices_updated_at BEFORE UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_payments_updated_at BEFORE UPDATE ON payments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_expenses_updated_at BEFORE UPDATE ON expenses
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Functions
CREATE OR REPLACE FUNCTION calculate_invoice_balance(p_invoice_id UUID)
RETURNS DECIMAL(15,2) AS $$
DECLARE
    v_total DECIMAL(15,2);
    v_paid DECIMAL(15,2);
BEGIN
    SELECT total_amount, amount_paid
    INTO v_total, v_paid
    FROM invoices
    WHERE id = p_invoice_id;

    RETURN v_total - v_paid;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_client_stats(p_client_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE clients
    SET
        total_invoiced = (
            SELECT COALESCE(SUM(total_amount), 0)
            FROM invoices
            WHERE client_id = p_client_id
        ),
        total_paid = (
            SELECT COALESCE(SUM(amount_paid), 0)
            FROM invoices
            WHERE client_id = p_client_id
        ),
        average_payment_days = (
            SELECT AVG(EXTRACT(DAY FROM paid_at - issue_date))
            FROM invoices
            WHERE client_id = p_client_id
            AND paid_at IS NOT NULL
        )
    WHERE id = p_client_id;
END;
$$ LANGUAGE plpgsql;

-- Full-text search indexes
CREATE INDEX idx_invoices_fts ON invoices USING gin (
    to_tsvector('english',
        coalesce(invoice_number, '') || ' ' ||
        coalesce(notes, '') || ' ' ||
        coalesce(terms, '')
    )
);

CREATE INDEX idx_clients_fts ON clients USING gin (
    to_tsvector('english',
        coalesce(name, '') || ' ' ||
        coalesce(company_name, '') || ' ' ||
        coalesce(email, '')
    )
);

-- Tax rates table
CREATE TABLE IF NOT EXISTS tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_code VARCHAR(2) NOT NULL UNIQUE,
    state_name VARCHAR(100) NOT NULL,
    tax_rate DECIMAL(5,2) NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

INSERT INTO tax_rates (state_code, state_name, tax_rate) VALUES
('CA', 'California', 7.25),
('NY', 'New York', 8.875),
('TX', 'Texas', 6.25),
('FL', 'Florida', 6.00),
('IL', 'Illinois', 6.25),
('PA', 'Pennsylvania', 6.00),
('OH', 'Ohio', 5.75),
('GA', 'Georgia', 4.00),
('NC', 'North Carolina', 4.75),
('MI', 'Michigan', 6.00);
