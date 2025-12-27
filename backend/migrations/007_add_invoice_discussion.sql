-- Add invoice discussion table for 2-way communication between seller and buyer
CREATE TABLE invoice_discussions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    sender_type VARCHAR(10) NOT NULL CHECK (sender_type IN ('seller', 'buyer')),
    message TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Index for faster queries
    CONSTRAINT fk_invoice FOREIGN KEY(invoice_id) REFERENCES invoices(id)
);

-- Indexes for performance
CREATE INDEX idx_invoice_discussions_invoice_id ON invoice_discussions(invoice_id);
CREATE INDEX idx_invoice_discussions_created_at ON invoice_discussions(created_at);

-- Add comment for documentation
COMMENT ON TABLE invoice_discussions IS 'Stores discussion messages between seller and buyer for each invoice';
COMMENT ON COLUMN invoice_discussions.sender_type IS 'Who sent the message: seller or buyer';
COMMENT ON COLUMN invoice_discussions.message IS 'The message content';
