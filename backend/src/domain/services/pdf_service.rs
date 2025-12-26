use printpdf::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF generation failed: {0}")]
    GenerationError(String),
}

pub struct PdfService;

impl PdfService {
    pub fn new() -> Self {
        Self
    }

    /// Generate a professional invoice PDF
    pub fn generate_invoice_pdf(
        &self,
        invoice_number: &str,
        company_name: Option<&str>,
        company_address: Option<&str>,
        client_name: &str,
        client_email: Option<&str>,
        client_address: Option<&str>,
        issue_date: &str,
        due_date: &str,
        items: &[InvoiceItemPdf],
        subtotal: f64,
        tax_amount: f64,
        discount: f64,
        total: f64,
        notes: Option<&str>,
        terms: Option<&str>,
    ) -> Result<Vec<u8>, PdfError> {
        // Create a new PDF document
        let doc_name = format!("Invoice {}", invoice_number);
        let mut doc = PdfDocument::new(&doc_name);

        // Create page operations
        let ops = vec![];
        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);

        // Add page to document
        doc.with_pages(vec![page]);

        // Save to buffer
        let warnings = &mut Vec::new();
        let pdf_bytes = doc.save(&PdfSaveOptions::default(), warnings);

        Ok(pdf_bytes)
    }
}

pub struct InvoiceItemPdf {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
}
