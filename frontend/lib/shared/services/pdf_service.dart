import 'dart:io';
import 'dart:typed_data';
import 'package:pdf/pdf.dart';
import 'package:pdf/widgets.dart' as pw;
import 'package:printing/printing.dart';
import 'package:share_plus/share_plus.dart';
import 'package:path_provider/path_provider.dart';
import 'package:flutter/material.dart' as material;
import '../../../features/invoices/presentation/providers/invoice_provider.dart';

/// PDF Service for generating and sharing invoices
class PdfService {
  static final PdfService _instance = PdfService._internal();
  factory PdfService() => _instance;

  PdfService._internal();

  /// Generate PDF from invoice data
  Future<Uint8List> generateInvoicePdf(InvoiceDetail invoice) async {
    final pdf = pw.Document();

    pdf.addPage(
      pw.Page(
        build: (pw.Context context) {
          return pw.Column(
            crossAxisAlignment: pw.CrossAxisAlignment.start,
            children: [
              // Header
              pw.Row(
                mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                children: [
                  pw.Text(
                    'INVOICE',
                    style: pw.TextStyle(
                      fontSize: 24,
                      fontWeight: pw.FontWeight.bold,
                      color: PdfColors.blue700,
                    ),
                  ),
                  pw.Text(
                    invoice.invoiceNumber,
                    style: pw.TextStyle(
                      fontSize: 16,
                      fontWeight: pw.FontWeight.bold,
                    ),
                  ),
                ],
              ),
              pw.SizedBox(height: 20),

              // Business and Client Info
              pw.Row(
                crossAxisAlignment: pw.CrossAxisAlignment.start,
                mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                children: [
                  // Client Info
                  pw.Column(
                    crossAxisAlignment: pw.CrossAxisAlignment.start,
                    children: [
                      pw.Text('Bill To:', style: pw.TextStyle(fontWeight: pw.FontWeight.bold)),
                      pw.Text(invoice.clientName),
                      if (invoice.clientEmail != null) pw.Text(invoice.clientEmail!),
                      if (invoice.clientPhone != null) pw.Text(invoice.clientPhone!),
                    ],
                  ),
                  // Invoice Info
                  pw.Column(
                    crossAxisAlignment: pw.CrossAxisAlignment.end,
                    children: [
                      pw.Text('Issue Date: ${_formatDate(invoice.issueDate)}'),
                      pw.Text('Due Date: ${_formatDate(invoice.dueDate)}'),
                      pw.Text(
                        'Status: ${invoice.status.toUpperCase()}',
                        style: pw.TextStyle(
                          color: _getStatusColor(invoice.status),
                          fontWeight: pw.FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
              pw.SizedBox(height: 20),

              // Items Table
              pw.Table.fromTextArray(
                context: context,
                data: [
                  ['Description', 'Qty', 'Unit Price', 'Amount'],
                  ...invoice.items.map((item) => [
                        item.description,
                        item.quantity.toString(),
                        '\$${item.unitPrice.toStringAsFixed(2)}',
                        '\$${item.amount.toStringAsFixed(2)}',
                      ]),
                ],
                headerStyle: pw.TextStyle(fontWeight: pw.FontWeight.bold),
                cellAlignment: pw.Alignment.centerRight,
                cellAlignments: {0: pw.Alignment.centerLeft},
              ),

              pw.SizedBox(height: 20),

              // Totals
              pw.Align(
                alignment: pw.Alignment.centerRight,
                child: pw.Container(
                  width: 250,
                  child: pw.Column(
                    children: [
                      pw.Row(
                        mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                        children: [
                          pw.Text('Subtotal:'),
                          pw.Text('\$${invoice.subtotal.toStringAsFixed(2)}'),
                        ],
                      ),
                      if (invoice.taxAmount > 0)
                        pw.Row(
                          mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                          children: [
                            pw.Text('Tax:'),
                            pw.Text('\$${invoice.taxAmount.toStringAsFixed(2)}'),
                          ],
                        ),
                      if (invoice.discountAmount > 0)
                        pw.Row(
                          mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                          children: [
                            pw.Text('Discount:'),
                            pw.Text('- \$${invoice.discountAmount.toStringAsFixed(2)}'),
                          ],
                        ),
                      pw.Divider(),
                      pw.Row(
                        mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                        children: [
                          pw.Text(
                            'Total:',
                            style: pw.TextStyle(
                              fontWeight: pw.FontWeight.bold,
                              fontSize: 16,
                            ),
                          ),
                          pw.Text(
                            '\$${invoice.totalAmount.toStringAsFixed(2)}',
                            style: pw.TextStyle(
                              fontWeight: pw.FontWeight.bold,
                              fontSize: 16,
                              color: PdfColors.blue700,
                            ),
                          ),
                        ],
                      ),
                      if (invoice.amountPaid > 0) ...[
                        pw.SizedBox(height: 8),
                        pw.Row(
                          mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                          children: [
                            pw.Text('Amount Paid:'),
                            pw.Text('\$${invoice.amountPaid.toStringAsFixed(2)}'),
                          ],
                        ),
                        pw.Row(
                          mainAxisAlignment: pw.MainAxisAlignment.spaceBetween,
                          children: [
                            pw.Text(
                              'Balance Due:',
                              style: pw.TextStyle(fontWeight: pw.FontWeight.bold),
                            ),
                            pw.Text(
                              '\$${invoice.balanceDue.toStringAsFixed(2)}',
                              style: pw.TextStyle(
                                fontWeight: pw.FontWeight.bold,
                                color: PdfColors.red700,
                              ),
                            ),
                          ],
                        ),
                      ],
                    ],
                  ),
                ),
              ),

              if (invoice.notes != null || invoice.terms != null) ...[
                pw.SizedBox(height: 20),
                pw.Divider(),
                pw.SizedBox(height: 10),
                if (invoice.notes != null) ...[
                  pw.Text('Notes:', style: pw.TextStyle(fontWeight: pw.FontWeight.bold)),
                  pw.Text(invoice.notes!),
                  pw.SizedBox(height: 8),
                ],
                if (invoice.terms != null) ...[
                  pw.Text('Terms & Conditions:', style: pw.TextStyle(fontWeight: pw.FontWeight.bold)),
                  pw.Text(invoice.terms!),
                ],
              ],
            ],
          );
        },
      ),
    );

    return pdf.save();
  }

  /// Generate simple PDF from bytes
  Future<Uint8List> generateSimplePdf({
    required String title,
    required String content,
  }) async {
    final pdf = pw.Document();

    pdf.addPage(
      pw.Page(
        build: (pw.Context context) {
          return pw.Column(
            children: [
              pw.Text(title, style: pw.TextStyle(fontSize: 24, fontWeight: pw.FontWeight.bold)),
              pw.SizedBox(height: 20),
              pw.Text(content),
            ],
          );
        },
      ),
    );

    return pdf.save();
  }

  /// Preview PDF
  Future<void> previewPdf(Uint8List pdfBytes, material.BuildContext context) async {
    await material.showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => material.AlertDialog(
        title: const material.Text('PDF Preview'),
        content: material.SizedBox(
          width: double.maxFinite,
          height: 500,
          child: PdfPreview(
            build: (format) => pdfBytes,
            canChangePageFormat: false,
            canChangeOrientation: false,
            allowPrinting: false,
            allowSharing: false,
            useActions: false,
          ),
        ),
        actions: [
          material.TextButton(
            onPressed: () => material.Navigator.of(context).pop(),
            child: const material.Text('Close'),
          ),
        ],
      ),
    );
  }

  /// Share PDF
  Future<void> sharePdf(Uint8List pdfBytes, String fileName) async {
    try {
      final directory = await getTemporaryDirectory();
      final file = File('${directory.path}/$fileName.pdf');
      await file.writeAsBytes(pdfBytes);

      final xFile = XFile(file.path, mimeType: 'application/pdf');
      await Share.shareXFiles(
        [xFile],
        subject: 'Invoice PDF',
        text: 'Here is the invoice PDF',
      );
    } catch (e) {
      throw Exception('Failed to share PDF: $e');
    }
  }

  /// Print PDF
  Future<void> printPdf(Uint8List pdfBytes) async {
    await Printing.layoutPdf(
      onLayout: (format) => pdfBytes,
    );
  }

  /// Download PDF to device
  Future<String?> downloadPdf(Uint8List pdfBytes, String fileName) async {
    try {
      final directory = await getApplicationDocumentsDirectory();
      final file = File('${directory.path}/$fileName.pdf');
      await file.writeAsBytes(pdfBytes);
      return file.path;
    } catch (e) {
      return null;
    }
  }

  /// Get status color for PDF
  PdfColor _getStatusColor(String status) {
    switch (status.toLowerCase()) {
      case 'paid':
        return PdfColors.green700;
      case 'overdue':
        return PdfColors.red700;
      case 'pending':
        return PdfColors.orange700;
      default:
        return PdfColors.blue700;
    }
  }

  /// Format date
  String _formatDate(DateTime date) {
    return '${date.day.toString().padLeft(2, '0')}/${date.month.toString().padLeft(2, '0')}/${date.year}';
  }
}
