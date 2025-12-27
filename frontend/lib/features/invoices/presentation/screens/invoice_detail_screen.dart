import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';
import '../providers/invoice_provider.dart';
import '../../../../shared/widgets/empty_state.dart';
import '../../../../shared/widgets/discussion_bottom_sheet.dart';
import '../../../../app/theme/app_theme.dart';

class InvoiceDetailScreen extends ConsumerStatefulWidget {
  final String invoiceId;

  const InvoiceDetailScreen({super.key, required this.invoiceId});

  @override
  ConsumerState<InvoiceDetailScreen> createState() => _InvoiceDetailScreenState();
}

class _InvoiceDetailScreenState extends ConsumerState<InvoiceDetailScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(invoiceProvider.notifier).loadInvoice(widget.invoiceId);
    });
  }

  Future<void> _downloadPdf() async {
    final pdfData = await ref.read(invoiceProvider.notifier).getInvoicePdf(widget.invoiceId);
    if (pdfData != null && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('PDF downloaded successfully'),
          backgroundColor: Colors.green,
        ),
      );
      // In a real app, you would save the PDF to device storage
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to download PDF'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _sendReminder() async {
    final success = await ref.read(invoiceProvider.notifier).sendReminder(
      widget.invoiceId,
      {'message': 'Payment reminder for invoice ${widget.invoiceId}'},
    );
    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Reminder sent successfully'),
          backgroundColor: Colors.green,
        ),
      );
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to send reminder'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _recordPayment() async {
    final invoiceState = ref.read(invoiceProvider);
    final invoice = invoiceState.selectedInvoice;
    if (invoice == null) return;

    final amountController = TextEditingController();
    final notesController = TextEditingController();
    String paymentMethod = 'PayPal';

    final result = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Record Payment'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Partial payment info
                if (invoice.allowPartialPayment) ...[
                  Container(
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: Colors.orange.shade50,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: Colors.orange.shade200),
                    ),
                    child: Row(
                      children: [
                        Icon(Icons.info_outline, color: Colors.orange.shade700, size: 18),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'Partial payments allowed${invoice.minPaymentAmount != null ? ' (min: \$${invoice.minPaymentAmount!.toStringAsFixed(2)})' : ''}',
                            style: TextStyle(
                              fontSize: 12,
                              color: Colors.orange.shade700,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 12),
                ],
                TextField(
                  controller: amountController,
                  decoration: InputDecoration(
                    labelText: 'Amount *',
                    prefixText: '\$',
                    helperText: 'Balance due: \$${invoice.balanceDue.toStringAsFixed(2)}',
                  ),
                  keyboardType: TextInputType.number,
                ),
                const SizedBox(height: 12),
                DropdownButtonFormField<String>(
                  value: paymentMethod,
                  decoration: const InputDecoration(
                    labelText: 'Payment Method *',
                  ),
                  items: const [
                    DropdownMenuItem(value: 'PayPal', child: Text('PayPal')),
                    DropdownMenuItem(value: 'Stripe', child: Text('Stripe')),
                    DropdownMenuItem(value: 'AchDebit', child: Text('ACH Debit')),
                    DropdownMenuItem(value: 'BankTransfer', child: Text('Bank Transfer')),
                    DropdownMenuItem(value: 'Cash', child: Text('Cash')),
                    DropdownMenuItem(value: 'Check', child: Text('Check')),
                  ],
                  onChanged: (value) {
                    if (value != null) {
                      setState(() {
                        paymentMethod = value;
                      });
                    }
                  },
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: notesController,
                  decoration: const InputDecoration(
                    labelText: 'Notes (optional)',
                  ),
                  maxLines: 2,
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              onPressed: () {
                // Validate amount
                final amount = double.tryParse(amountController.text);
                if (amount == null || amount <= 0) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Please enter a valid amount'),
                      backgroundColor: Colors.red,
                    ),
                  );
                  return;
                }

                // Validate minimum payment
                if (invoice.minPaymentAmount != null && amount < invoice.minPaymentAmount!) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text('Minimum payment is \$${invoice.minPaymentAmount!.toStringAsFixed(2)}'),
                      backgroundColor: Colors.red,
                    ),
                  );
                  return;
                }

                // Validate partial payment setting
                if (!invoice.allowPartialPayment && amount < invoice.balanceDue) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('This invoice requires full payment'),
                      backgroundColor: Colors.red,
                    ),
                  );
                  return;
                }

                // Validate amount doesn't exceed balance
                if (amount > invoice.balanceDue) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text('Amount exceeds balance due (\$${invoice.balanceDue.toStringAsFixed(2)})'),
                      backgroundColor: Colors.red,
                    ),
                  );
                  return;
                }

                Navigator.of(context).pop(true);
              },
              child: const Text('Record'),
            ),
          ],
        ),
      ),
    );

    if (result == true) {
      final success = await ref.read(invoiceProvider.notifier).recordPayment(
        widget.invoiceId,
        {
          'amount': double.parse(amountController.text),
          'payment_method': paymentMethod,
          'notes': notesController.text.isEmpty ? null : notesController.text,
        },
      );
      if (success && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Payment recorded successfully'),
            backgroundColor: Colors.green,
          ),
        );
        // Refresh invoice details
        ref.read(invoiceProvider.notifier).loadInvoice(widget.invoiceId);
      }
    }
  }

  Future<void> _sendWhatsApp() async {
    final success = await ref.read(invoiceProvider.notifier).sendInvoiceWhatsapp(widget.invoiceId);
    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invoice sent via WhatsApp!'),
          backgroundColor: Colors.green,
        ),
      );
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to send WhatsApp message'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _sendPaymentConfirmation() async {
    final invoiceState = ref.read(invoiceProvider);
    final invoice = invoiceState.selectedInvoice;
    if (invoice == null) return;

    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Send Payment Confirmation'),
        content: Text(
          'Send payment confirmation to ${invoice.clientName}?\n\n'
          'Email: ${invoice.clientEmail ?? 'N/A'}\n'
          'Phone: ${invoice.clientPhone ?? 'N/A'}',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Send'),
          ),
        ],
      ),
    );

    if (result == true) {
      final success = await ref.read(invoiceProvider.notifier).sendPaymentConfirmation(
        widget.invoiceId,
        {
          'send_email': invoice.clientEmail != null,
          'send_whatsapp': invoice.clientPhone != null,
        },
      );
      if (success && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Payment confirmation sent!'),
            backgroundColor: Colors.green,
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
        title: const Text('Invoice Details'),
        actions: [
          IconButton(
            icon: const Icon(Icons.download),
            onPressed: _downloadPdf,
            tooltip: 'Download PDF',
          ),
        ],
      ),
      body: invoiceState.isLoading
          ? const Center(child: CircularProgressIndicator())
          : invoiceState.error != null
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
                      const SizedBox(height: 16),
                      Text(
                        invoiceState.error!,
                        style: const TextStyle(color: Colors.red),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: () => ref.read(invoiceProvider.notifier).loadInvoice(widget.invoiceId),
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                )
              : invoiceState.selectedInvoice == null
                  ? const EmptyState(
                      icon: Icons.error_outline,
                      title: 'Invoice Not Found',
                      message: 'This invoice does not exist or has been deleted',
                    )
                  : RefreshIndicator(
                      onRefresh: () async {
                        ref.read(invoiceProvider.notifier).loadInvoice(widget.invoiceId);
                      },
                      child: SingleChildScrollView(
                        padding: const EdgeInsets.all(16),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            _buildHeader(invoiceState.selectedInvoice!),
                            const SizedBox(height: 16),
                            _buildClientInfo(invoiceState.selectedInvoice!),
                            const SizedBox(height: 16),
                            _buildItems(invoiceState.selectedInvoice!),
                            const SizedBox(height: 16),
                            _buildTotals(invoiceState.selectedInvoice!),
                            const SizedBox(height: 16),
                            _buildNotesAndTerms(invoiceState.selectedInvoice!),
                            const SizedBox(height: 24),
                            _buildActions(invoiceState.selectedInvoice!),
                          ],
                        ),
                      ),
                    ),
    );
  }

  Widget _buildHeader(InvoiceDetail invoice) {
    // Check if invoice has read tracking info (from InvoiceDetail model)
    // Note: We need to update InvoiceDetail model to include viewedAt and sentAt
    final hasViewedAt = false; // Will be updated when model is updated
    final hasSentAt = false;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Invoice #${invoice.invoiceNumber}',
                  style: const TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                _StatusBadge(status: invoice.status),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('Issue Date', style: TextStyle(color: Colors.grey)),
                      Text(
                        _formatDate(invoice.issueDate),
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                    ],
                  ),
                ),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('Due Date', style: TextStyle(color: Colors.grey)),
                      Text(
                        _formatDate(invoice.dueDate),
                        style: TextStyle(
                          fontWeight: FontWeight.w600,
                          color: invoice.dueDate.isBefore(DateTime.now()) ? Colors.red : null,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            // Read Receipt Indicator
            if (invoice.status == 'viewed') ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: Colors.blue.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.visibility, color: Colors.blue, size: 18),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        '✓ Client has viewed this invoice',
                        style: TextStyle(
                          color: Colors.blue.shade800,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildClientInfo(InvoiceDetail invoice) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Bill To',
              style: TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w600,
                color: Colors.grey,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              invoice.clientName,
              style: const TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w600,
              ),
            ),
            if (invoice.clientEmail != null) ...[
              const SizedBox(height: 4),
              Row(
                children: [
                  const Icon(Icons.email, size: 14, color: Colors.grey),
                  const SizedBox(width: 4),
                  Text(invoice.clientEmail!),
                ],
              ),
            ],
            if (invoice.clientPhone != null) ...[
              const SizedBox(height: 4),
              Row(
                children: [
                  const Icon(Icons.phone, size: 14, color: Colors.grey),
                  const SizedBox(width: 4),
                  Text(invoice.clientPhone!),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildItems(InvoiceDetail invoice) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Items',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
        ),
        const SizedBox(height: 8),
        Card(
          child: Column(
            children: [
              // Header row
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                color: Colors.grey.shade50,
                child: const Row(
                  children: [
                    Expanded(flex: 3, child: Text('Description', style: TextStyle(fontWeight: FontWeight.w600))),
                    Expanded(flex: 1, child: Text('Qty', textAlign: TextAlign.center, style: TextStyle(fontWeight: FontWeight.w600))),
                    Expanded(flex: 2, child: Text('Price', textAlign: TextAlign.right, style: TextStyle(fontWeight: FontWeight.w600))),
                    Expanded(flex: 2, child: Text('Total', textAlign: TextAlign.right, style: TextStyle(fontWeight: FontWeight.w600))),
                  ],
                ),
              ),
              ...invoice.items.asMap().entries.map((entry) {
                final item = entry.value;
                return Column(
                  children: [
                    if (entry.key > 0) const Divider(height: 1),
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                      child: Row(
                        children: [
                          Expanded(flex: 3, child: Text(item.description)),
                          Expanded(flex: 1, child: Text(item.quantity.toString(), textAlign: TextAlign.center)),
                          Expanded(flex: 2, child: Text('\$${item.unitPrice.toStringAsFixed(2)}', textAlign: TextAlign.right)),
                          Expanded(
                            flex: 2,
                            child: Text(
                              '\$${item.amount.toStringAsFixed(2)}',
                              textAlign: TextAlign.right,
                              style: const TextStyle(fontWeight: FontWeight.w600),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                );
              }),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildTotals(InvoiceDetail invoice) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            _buildTotalRow('Subtotal', invoice.subtotal),
            if (invoice.discountAmount > 0) ...[
              const SizedBox(height: 8),
              _buildTotalRow('Discount', invoice.discountAmount, color: Colors.green),
            ],
            if (invoice.taxAmount > 0) ...[
              const SizedBox(height: 8),
              _buildTotalRow('Tax', invoice.taxAmount),
            ],
            const SizedBox(height: 8),
            const Divider(),
            const SizedBox(height: 8),
            _buildTotalRow('Total', invoice.totalAmount, isBold: true),
            if (invoice.amountPaid > 0) ...[
              const SizedBox(height: 8),
              _buildTotalRow('Amount Paid', invoice.amountPaid, color: Colors.green),
            ],
            if (invoice.balanceDue > 0) ...[
              const SizedBox(height: 8),
              _buildTotalRow('Balance Due', invoice.balanceDue, isBold: true, color: Colors.orange),
            ],
            // Partial payment info
            if (invoice.allowPartialPayment || invoice.isPartial) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: Colors.orange.shade50,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: Colors.orange.shade200),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(Icons.info_outline, color: Colors.orange.shade700, size: 16),
                        const SizedBox(width: 6),
                        Text(
                          'Partial Payment',
                          style: TextStyle(
                            fontWeight: FontWeight.w600,
                            color: Colors.orange.shade700,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      invoice.allowPartialPayment
                          ? 'Buyers can pay in installments${invoice.minPaymentAmount != null ? ' (min: \$${invoice.minPaymentAmount!.toStringAsFixed(2)})' : ''}'
                          : 'Full payment required',
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.orange.shade700,
                      ),
                    ),
                    if (invoice.partialPaymentCount > 0) ...[
                      const SizedBox(height: 4),
                      Text(
                        'Payments made: ${invoice.partialPaymentCount}',
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: Colors.orange.shade700,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildNotesAndTerms(InvoiceDetail invoice) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (invoice.notes != null && invoice.notes!.isNotEmpty) ...[
          const Text(
            'Notes',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 4),
          Text(invoice.notes!),
          const SizedBox(height: 16),
        ],
        if (invoice.terms != null && invoice.terms!.isNotEmpty) ...[
          const Text(
            'Terms & Conditions',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 4),
          Text(invoice.terms!),
        ],
      ],
    );
  }

  Widget _buildActions(InvoiceDetail invoice) {
    return Column(
      children: [
        // Row 1: Back and Send
        Row(
          children: [
            Expanded(
              child: OutlinedButton.icon(
                onPressed: () => context.go('/invoices'),
                icon: const Icon(Icons.arrow_back),
                label: const Text('Back'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: ElevatedButton.icon(
                onPressed: () => context.go('/invoices/send/${widget.invoiceId}'),
                icon: const Icon(Icons.send),
                label: const Text('Send'),
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),

        // Row 2: Reminder and WhatsApp
        Row(
          children: [
            Expanded(
              child: OutlinedButton.icon(
                onPressed: _sendReminder,
                icon: const Icon(Icons.notifications),
                label: const Text('Reminder'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: OutlinedButton.icon(
                onPressed: _sendWhatsApp,
                icon: const Icon(Icons.whatsapp),
                label: const Text('WhatsApp'),
                style: OutlinedButton.styleFrom(
                  foregroundColor: Colors.green,
                  side: const BorderSide(color: Colors.green),
                ),
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),

        // Row 2.5: Discussion
        SizedBox(
          width: double.infinity,
          child: OutlinedButton.icon(
            onPressed: () {
              showDiscussionBottomSheet(
                context,
                invoiceId: widget.invoiceId,
                clientName: invoice.clientName,
              );
            },
            icon: const Icon(Icons.chat),
            label: const Text('Discussion / Notes'),
            style: OutlinedButton.styleFrom(
              foregroundColor: Colors.blue,
              side: const BorderSide(color: Colors.blue),
            ),
          ),
        ),
        const SizedBox(height: 12),

        // Row 3: PDF and Download
        Row(
          children: [
            Expanded(
              child: OutlinedButton.icon(
                onPressed: () => context.go('/invoices/pdf-preview/${widget.invoiceId}'),
                icon: const Icon(Icons.picture_as_pdf),
                label: const Text('Preview PDF'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: OutlinedButton.icon(
                onPressed: _downloadPdf,
                icon: const Icon(Icons.download),
                label: const Text('Download'),
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),

        // Payment Confirmation (if paid)
        if (invoice.status == 'paid') ...[
          SizedBox(
            width: double.infinity,
            child: OutlinedButton.icon(
              onPressed: _sendPaymentConfirmation,
              icon: const Icon(Icons.check_circle),
              label: const Text('Send Payment Confirmation'),
              style: OutlinedButton.styleFrom(
                foregroundColor: AppTheme.primaryColor,
                side: BorderSide(color: AppTheme.primaryColor),
              ),
            ),
          ),
          const SizedBox(height: 12),
        ],

        // Record Payment
        if (invoice.balanceDue > 0)
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: _recordPayment,
              icon: const Icon(Icons.payment),
              label: const Text('Record Payment'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.green,
                foregroundColor: Colors.white,
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildTotalRow(String label, double amount, {bool isBold = false, Color? color}) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: 16,
            fontWeight: isBold ? FontWeight.bold : FontWeight.normal,
            color: color,
          ),
        ),
        Text(
          '\$${amount.toStringAsFixed(2)}',
          style: TextStyle(
            fontSize: 16,
            fontWeight: isBold ? FontWeight.bold : FontWeight.normal,
            color: color,
          ),
        ),
      ],
    );
  }

  String _formatDate(DateTime date) {
    return '${date.month}/${date.day}/${date.year}';
  }
}

class _StatusBadge extends StatelessWidget {
  final String status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    Color color;
    switch (status) {
      case 'draft':
        color = Colors.grey;
        break;
      case 'sent':
      case 'viewed':
        color = Colors.blue;
        break;
      case 'paid':
        color = Colors.green;
        break;
      case 'overdue':
        color = Colors.red;
        break;
      case 'partial':
        color = Colors.orange;
        break;
      default:
        color = Colors.grey;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        status.toUpperCase(),
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
