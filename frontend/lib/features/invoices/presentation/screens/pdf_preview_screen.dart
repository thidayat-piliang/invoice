import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'dart:typed_data';
import '../providers/invoice_provider.dart';
import '../../../../shared/services/pdf_service.dart';

/// PDF Preview and Share Screen
class PdfPreviewScreen extends ConsumerStatefulWidget {
  final String invoiceId;

  const PdfPreviewScreen({
    super.key,
    required this.invoiceId,
  });

  @override
  ConsumerState<PdfPreviewScreen> createState() => _PdfPreviewScreenState();
}

class _PdfPreviewScreenState extends ConsumerState<PdfPreviewScreen> {
  final PdfService _pdfService = PdfService();
  Uint8List? _pdfBytes;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadInvoiceAndGeneratePdf();
  }

  Future<void> _loadInvoiceAndGeneratePdf() async {
    try {
      // Load invoice details
      await ref.read(invoiceProvider.notifier).loadInvoice(widget.invoiceId);
      final invoice = ref.read(invoiceProvider).selectedInvoice;

      if (invoice == null) {
        setState(() {
          _error = 'Invoice not found';
          _isLoading = false;
        });
        return;
      }

      // Generate PDF
      final pdfBytes = await _pdfService.generateInvoicePdf(invoice);
      setState(() {
        _pdfBytes = pdfBytes;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = 'Failed to generate PDF: $e';
        _isLoading = false;
      });
    }
  }

  Future<void> _sharePdf() async {
    if (_pdfBytes == null) return;

    try {
      final invoice = ref.read(invoiceProvider).selectedInvoice;
      final fileName = invoice?.invoiceNumber ?? 'invoice';
      await _pdfService.sharePdf(_pdfBytes!, fileName);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('PDF shared successfully'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to share: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  Future<void> _printPdf() async {
    if (_pdfBytes == null) return;

    try {
      await _pdfService.printPdf(_pdfBytes!);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Printing started'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to print: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  Future<void> _downloadPdf() async {
    if (_pdfBytes == null) return;

    try {
      final invoice = ref.read(invoiceProvider).selectedInvoice;
      final fileName = invoice?.invoiceNumber ?? 'invoice';
      final path = await _pdfService.downloadPdf(_pdfBytes!, fileName);

      if (mounted) {
        if (path != null) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('PDF saved to: $path'),
              backgroundColor: Colors.green,
            ),
          );
        } else {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Failed to save PDF'),
              backgroundColor: Colors.red,
            ),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to download: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final invoiceState = ref.watch(invoiceProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('PDF Preview'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/invoices/${widget.invoiceId}'),
        ),
        actions: [
          if (_pdfBytes != null)
            IconButton(
              icon: const Icon(Icons.share),
              onPressed: _sharePdf,
              tooltip: 'Share PDF',
            ),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
                      const SizedBox(height: 16),
                      Text(
                        _error!,
                        style: const TextStyle(color: Colors.red),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: _loadInvoiceAndGeneratePdf,
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                )
              : Column(
                  children: [
                    // Preview Area
                    Expanded(
                      child: Container(
                        color: Colors.grey.shade200,
                        child: Center(
                          child: Card(
                            margin: const EdgeInsets.all(16),
                            elevation: 4,
                            child: Padding(
                              padding: const EdgeInsets.all(16),
                              child: Column(
                                mainAxisSize: MainAxisSize.min,
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Row(
                                    children: [
                                      const Icon(Icons.description, color: Colors.blue),
                                      const SizedBox(width: 8),
                                      Text(
                                        invoiceState.selectedInvoice?.invoiceNumber ?? 'Invoice',
                                        style: const TextStyle(
                                          fontSize: 18,
                                          fontWeight: FontWeight.bold,
                                        ),
                                      ),
                                    ],
                                  ),
                                  const SizedBox(height: 8),
                                  Text(
                                    'Client: ${invoiceState.selectedInvoice?.clientName ?? 'N/A'}',
                                    style: TextStyle(color: Colors.grey.shade700),
                                  ),
                                  Text(
                                    'Amount: \$${invoiceState.selectedInvoice?.totalAmount.toStringAsFixed(2) ?? '0.00'}',
                                    style: TextStyle(color: Colors.grey.shade700),
                                  ),
                                  const SizedBox(height: 16),
                                  const Text(
                                    'PDF Preview Available',
                                    style: TextStyle(
                                      fontSize: 14,
                                      color: Colors.green,
                                      fontWeight: FontWeight.w600,
                                    ),
                                  ),
                                  const SizedBox(height: 8),
                                  Text(
                                    'Size: ${(_pdfBytes!.length / 1024).toStringAsFixed(2)} KB',
                                    style: TextStyle(
                                      fontSize: 12,
                                      color: Colors.grey.shade600,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),

                    // Action Buttons
                    Container(
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        border: Border(
                          top: BorderSide(color: Colors.grey.shade300),
                        ),
                      ),
                      child: Column(
                        children: [
                          Row(
                            children: [
                              Expanded(
                                child: _buildActionButton(
                                  icon: Icons.share,
                                  label: 'Share',
                                  onPressed: _sharePdf,
                                  color: Colors.blue,
                                ),
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: _buildActionButton(
                                  icon: Icons.print,
                                  label: 'Print',
                                  onPressed: _printPdf,
                                  color: Colors.green,
                                ),
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: _buildActionButton(
                                  icon: Icons.download,
                                  label: 'Save',
                                  onPressed: _downloadPdf,
                                  color: Colors.orange,
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 12),
                          SizedBox(
                            width: double.infinity,
                            child: OutlinedButton.icon(
                              onPressed: () => context.go('/invoices/${widget.invoiceId}'),
                              icon: const Icon(Icons.arrow_back),
                              label: const Text('Back to Invoice'),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
    );
  }

  Widget _buildActionButton({
    required IconData icon,
    required String label,
    required VoidCallback onPressed,
    required Color color,
  }) {
    return ElevatedButton.icon(
      onPressed: onPressed,
      icon: Icon(icon, size: 18),
      label: Text(label),
      style: ElevatedButton.styleFrom(
        backgroundColor: color,
        foregroundColor: Colors.white,
        padding: const EdgeInsets.symmetric(vertical: 12),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
        ),
      ),
    );
  }
}
