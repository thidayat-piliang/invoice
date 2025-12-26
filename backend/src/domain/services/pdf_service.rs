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

    /// Generate a professional invoice PDF with full details
    /// NOTE: Tax information is displayed for informational purposes only.
    /// FlashBill does not calculate, verify, or file taxes on your behalf.
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
        tax_label: Option<&str>,
    ) -> Result<Vec<u8>, PdfError> {
        // Create PDF document
        let mut doc = PdfDocument::new(&format!("Invoice {}", invoice_number));

        // Create operations for the page
        let mut ops: Vec<Op> = Vec::new();

        // Start text section
        ops.push(Op::StartTextSection);

        // === HEADER ===
        // Company Name (Bold, 18pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(18.0),
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(20.0).into(),
                y: Mm(270.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(company_name.unwrap_or("FlashBill").to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // Company Address (Regular, 9pt)
        if let Some(addr) = company_address {
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(9.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(260.0).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(addr.to_string())],
                font: BuiltinFont::Helvetica,
            });
        }

        // Invoice Title (Bold, 20pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(20.0),
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(130.0).into(),
                y: Mm(270.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("INVOICE".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // Invoice Number (Regular, 11pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(11.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(130.0).into(),
                y: Mm(260.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!("Invoice #: {}", invoice_number))],
            font: BuiltinFont::Helvetica,
        });

        // Issue Date (Regular, 11pt)
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(130.0).into(),
                y: Mm(250.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!("Issue Date: {}", issue_date))],
            font: BuiltinFont::Helvetica,
        });

        // Due Date (Regular, 11pt)
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(130.0).into(),
                y: Mm(240.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!("Due Date: {}", due_date))],
            font: BuiltinFont::Helvetica,
        });

        // === BILL TO ===
        // "BILL TO:" (Bold, 12pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(12.0),
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(20.0).into(),
                y: Mm(240.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("BILL TO:".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // Client Name (Regular, 11pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(11.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(20.0).into(),
                y: Mm(230.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(client_name.to_string())],
            font: BuiltinFont::Helvetica,
        });

        // Client Email (Regular, 9pt)
        if let Some(email) = client_email {
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(9.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(220.0).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(email.to_string())],
                font: BuiltinFont::Helvetica,
            });
        }

        // Client Address (Regular, 9pt)
        if let Some(addr) = client_address {
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(9.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(210.0).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(addr.to_string())],
                font: BuiltinFont::Helvetica,
            });
        }

        // === LINE ITEMS HEADER ===
        let mut y_pos = 180.0;

        // Description header (Bold, 10pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(10.0),
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(20.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("Description".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // Qty header
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(110.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("Qty".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // Unit Price header
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(135.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("Unit Price".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // Total header
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(165.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("Total".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        // === LINE ITEMS ===
        y_pos -= 8.0;
        // Set regular font for line items (9pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(9.0),
            font: BuiltinFont::Helvetica,
        });

        for item in items {
            // Description
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(item.description.clone())],
                font: BuiltinFont::Helvetica,
            });

            // Qty
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(110.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(format!("{:.2}", item.quantity))],
                font: BuiltinFont::Helvetica,
            });

            // Unit Price
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(135.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(format!("{:.2}", item.unit_price))],
                font: BuiltinFont::Helvetica,
            });

            // Total
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(165.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(format!("{:.2}", item.total))],
                font: BuiltinFont::Helvetica,
            });

            y_pos -= 8.0;
        }

        // === TOTALS ===
        y_pos -= 10.0;

        // Subtotal (Regular, 10pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(10.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(135.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("Subtotal:".to_string())],
            font: BuiltinFont::Helvetica,
        });

        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(165.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!("{:.2}", subtotal))],
            font: BuiltinFont::Helvetica,
        });

        y_pos -= 8.0;

        // Tax
        if tax_amount > 0.0 {
            // Tax label (if provided)
            if let Some(label) = tax_label {
                ops.push(Op::SetTextCursor {
                    pos: Point {
                        x: Mm(135.0).into(),
                        y: Mm(y_pos).into(),
                    },
                });
                ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text(format!("{}:", label))],
                    font: BuiltinFont::Helvetica,
                });
            } else {
                ops.push(Op::SetTextCursor {
                    pos: Point {
                        x: Mm(135.0).into(),
                        y: Mm(y_pos).into(),
                    },
                });
                ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text("Tax:".to_string())],
                    font: BuiltinFont::Helvetica,
                });
            }

            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(165.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(format!("{:.2}", tax_amount))],
                font: BuiltinFont::Helvetica,
            });

            y_pos -= 8.0;
        }

        // Discount
        if discount > 0.0 {
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(135.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text("Discount:".to_string())],
                font: BuiltinFont::Helvetica,
            });

            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(165.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(format!("-{:.2}", discount))],
                font: BuiltinFont::Helvetica,
            });

            y_pos -= 8.0;
        }

        // Total (Bold, 12pt)
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(12.0),
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(135.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("TOTAL:".to_string())],
            font: BuiltinFont::HelveticaBold,
        });

        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(165.0).into(),
                y: Mm(y_pos).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!("{:.2}", total))],
            font: BuiltinFont::HelveticaBold,
        });

        // === NOTES & TERMS ===
        y_pos -= 15.0;

        if let Some(notes_text) = notes {
            // Notes header (Bold, 10pt)
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(10.0),
                font: BuiltinFont::HelveticaBold,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text("Notes:".to_string())],
                font: BuiltinFont::HelveticaBold,
            });

            y_pos -= 8.0;
            // Notes text (Regular, 9pt)
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(9.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(notes_text.to_string())],
                font: BuiltinFont::Helvetica,
            });

            y_pos -= 10.0;
        }

        if let Some(terms_text) = terms {
            // Terms header (Bold, 10pt)
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(10.0),
                font: BuiltinFont::HelveticaBold,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text("Terms:".to_string())],
                font: BuiltinFont::HelveticaBold,
            });

            y_pos -= 8.0;
            // Terms text (Regular, 9pt)
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(9.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(y_pos).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(terms_text.to_string())],
                font: BuiltinFont::Helvetica,
            });
        }

        // End text section
        ops.push(Op::EndTextSection);

        // === FOOTER ===
        // Add footer text
        ops.push(Op::StartTextSection);
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(8.0),
            font: BuiltinFont::Helvetica,
        });

        // Legal disclaimer about tax (only if tax is present)
        if tax_amount > 0.0 {
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(25.0).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(
                    "Tax information is for informational purposes only. FlashBill does not".to_string(),
                )],
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::SetTextCursor {
                pos: Point {
                    x: Mm(20.0).into(),
                    y: Mm(20.0).into(),
                },
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(
                    "calculate, verify, or file taxes on your behalf.".to_string(),
                )],
                font: BuiltinFont::Helvetica,
            });
        }

        // Main footer text
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(20.0).into(),
                y: Mm(15.0).into(),
            },
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(
                "Generated by FlashBill - Thank you for your business!".to_string(),
            )],
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::EndTextSection);

        // Create the page with A4 dimensions
        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
        doc.pages.push(page);

        // Serialize to bytes with default options
        let opts = PdfSaveOptions::default();
        let mut warnings = Vec::new();
        let output = doc.save(&opts, &mut warnings);

        Ok(output)
    }
}

pub struct InvoiceItemPdf {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
}
